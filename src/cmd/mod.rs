use crate::{conf::settings, pkg::server, prelude::Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "lets you run liteginx proxy")]
struct Cmd {
    #[command(subcommand)]
    command: Option<SubCommandType>,
}

#[derive(Subcommand)]
enum SubCommandType {
    Listen,
}

pub async fn run() -> Result<()> {
    let args = Cmd::parse();
    match args.command {
        Some(SubCommandType::Listen) => {}
        None => {
            tracing::error!("no subcommand passed")
        }
    }
    Ok(())
}
