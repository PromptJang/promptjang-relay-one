use anyhow::Result;
use clap::{Parser, Subcommand};
use promptjang_relay_one::{config::Config, mcp, migration, runtime};
use std::path::PathBuf;
use std::sync::Arc;

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

#[tokio::main]
async fn main() -> Result<()> {
    runtime::init_logging();
    let cli = Cli::parse();
    if matches!(cli.command, Some(Command::Mcp)) {
        return mcp::run().await;
    }
    let config = Arc::new(Config::load(cli.data_dir, cli.port)?);
    match cli.command {
        Some(Command::Export { output }) => {
            let pool = runtime::open_pool(&config).await?;
            migration::export(&pool, &output).await
        }
        Some(Command::Import { input }) => {
            let pool = runtime::open_pool(&config).await?;
            migration::import(&pool, &input).await
        }
        command => {
            runtime::serve_cli(
                config,
                !matches!(command, Some(Command::Serve { no_open: true })),
            )
            .await
        }
    }
}
