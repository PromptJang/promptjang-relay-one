use anyhow::{Context, Result, bail};
use clap::Subcommand;
use serde::Deserialize;

const RELEASES_URL: &str = "https://api.github.com/repos/PromptJang/promptjang-relay-one/releases/latest";

#[derive(Subcommand)]
pub enum UpdateCommand {
    Check,
    Apply,
    Rollback,
}

#[derive(Deserialize)]
struct Release { tag_name: String, html_url: String, body: Option<String> }

pub async fn run(command: UpdateCommand) -> Result<()> {
    match command {
        UpdateCommand::Check => {
            let release = reqwest::Client::new().get(RELEASES_URL).header("User-Agent", "promptjang-relay-one").send().await?.error_for_status()?.json::<Release>().await?;
            println!("{}\n{}\n{}", release.tag_name, release.html_url, release.body.unwrap_or_default());
            Ok(())
        }
        UpdateCommand::Apply => bail!("self-update is unavailable until the signed release manifest is published"),
        UpdateCommand::Rollback => {
            let executable = std::env::current_exe().context("resolve current executable")?;
            let previous = executable.with_extension("previous");
            if !previous.exists() { bail!("no previous executable is available"); }
            bail!("rollback staging is unavailable until the first signed update is installed")
        }
    }
}
