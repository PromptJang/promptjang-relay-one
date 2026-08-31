#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use promptjang_relay_one_core::{config::Config, mcp, migration, runtime};
use tauri::{Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tokio_util::sync::CancellationToken;

#[derive(Parser)]
#[command(version, about = "A durable local mailbox for CLI agents")]
struct Cli {
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(long, global = true, default_value_t = 8081)]
    port: u16,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long)]
        no_open: bool,
    },
    Mcp,
    Export {
        #[arg(long)]
        output: PathBuf,
    },
    Import {
        #[arg(long)]
        input: PathBuf,
    },
}

struct DesktopRuntime {
    shutdown: CancellationToken,
    server: Mutex<Option<tauri::async_runtime::JoinHandle<Result<()>>>>,
    exiting: AtomicBool,
}

struct DesktopLinks {
    docs: String,
}

async fn open_target(url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || open::that(url))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_docs(app: tauri::AppHandle, links: State<'_, DesktopLinks>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("documentation") {
        window.show().map_err(|error| error.to_string())?;
        window.unminimize().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let url = links.docs.parse().map_err(|error| format!("{error}"))?;
    WebviewWindowBuilder::new(&app, "documentation", WebviewUrl::External(url))
        .title("Documentation · PromptJang Relay One")
        .inner_size(980.0, 760.0)
        .min_inner_size(680.0, 520.0)
        .center()
        .build()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_release() -> Result<(), String> {
    open_target("https://github.com/PromptJang/promptjang-relay-one/releases/latest".to_string())
        .await
}

#[tauri::command]
async fn open_skill() -> Result<(), String> {
    open_target("https://github.com/PromptJang/promptjang-relay-skill".to_string()).await
}

fn main() -> Result<()> {
    runtime::init_logging();
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        return run_command(cli.data_dir, cli.port, command);
    }
    run_desktop(cli.data_dir, cli.port)
}

fn run_command(data_dir: Option<PathBuf>, port: u16, command: Command) -> Result<()> {
    let runtime_handle = tokio::runtime::Runtime::new().context("create Relay One runtime")?;
    runtime_handle.block_on(async move {
        if matches!(command, Command::Mcp) {
            return mcp::run().await;
        }
        let config = Arc::new(Config::load(data_dir, port)?);
        match command {
            Command::Serve { no_open } => runtime::serve_cli(config, !no_open).await,
            Command::Export { output } => {
                let pool = runtime::open_pool(&config).await?;
                migration::export(&pool, &output).await
            }
            Command::Import { input } => {
                let pool = runtime::open_pool(&config).await?;
                migration::import(&pool, &input).await
            }
            Command::Mcp => unreachable!("MCP was handled before configuration"),
        }
    })
}

fn run_desktop(data_dir: Option<PathBuf>, port: u16) -> Result<()> {
    let config = Arc::new(Config::load(data_dir, port)?.with_desktop_mode());
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_single_instance::init(
        |app, _arguments, _working_directory| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        },
    ));
    builder = builder.invoke_handler(tauri::generate_handler![
        open_docs,
        open_release,
        open_skill
    ]);
    builder = builder.setup(move |app| {
        let prepared = tauri::async_runtime::block_on(runtime::prepare(config.clone()))
            .context("start Relay One local service")?;
        let docs = format!("{}/docs", prepared.url());
        let url = prepared
            .url()
            .parse()
            .context("build Relay One desktop URL")?;
        let shutdown = CancellationToken::new();
        let shutdown_signal = shutdown.clone();
        let server = tauri::async_runtime::spawn(runtime::serve(prepared, async move {
            shutdown_signal.cancelled().await;
        }));
        app.manage(DesktopRuntime {
            shutdown,
            server: Mutex::new(Some(server)),
            exiting: AtomicBool::new(false),
        });
        app.manage(DesktopLinks { docs });
        WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
            .title("PromptJang Relay One")
            .inner_size(1180.0, 780.0)
            .min_inner_size(720.0, 560.0)
            .center()
            .build()
            .context("create Relay One desktop window")?;
        Ok(())
    });
    let app = builder
        .build(tauri::generate_context!())
        .context("build Relay One desktop application")?;
    app.run(|app_handle, event| match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            api.prevent_close();
            request_exit(app_handle);
        }
        RunEvent::ExitRequested { api, .. } => {
            let state = app_handle.state::<DesktopRuntime>();
            if !state.exiting.load(Ordering::Acquire) {
                api.prevent_exit();
                request_exit(app_handle);
            }
        }
        _ => {}
    });
    Ok(())
}

fn request_exit(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<DesktopRuntime>();
    if state.exiting.swap(true, Ordering::AcqRel) {
        return;
    }
    state.shutdown.cancel();
    let server = state
        .server
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(server) = server {
            match server.await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => tracing::error!(%error, "Relay One server stopped with an error"),
                Err(error) => tracing::error!(%error, "Relay One server task failed"),
            }
        }
        app_handle.exit(0);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn desktop_links_are_allowed_only_from_the_loopback_ui() {
        let capability = include_str!("../capabilities/default.json");
        let permission = include_str!("../permissions/open-links.toml");

        assert!(capability.contains("http://127.0.0.1:*"));
        assert!(capability.contains("open-links"));
        assert!(permission.contains("open_docs"));
        assert!(permission.contains("open_release"));
        assert!(permission.contains("open_skill"));
    }
}
