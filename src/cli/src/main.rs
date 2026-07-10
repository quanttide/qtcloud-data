mod dropbox;
mod transfer;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qtcloud-data", about = "量潮数据云 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 数据传输（send / receive）
    Transfer(transfer::TransferArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Transfer(args) => transfer::run(args),
    }
}
