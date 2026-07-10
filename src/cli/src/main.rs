use clap::{Parser, Subcommand};
use qtcloud_data_cli::{blueprint, pipeline, process, transfer};

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
    /// 编排流程（receive → pipeline → send）
    Process(process::ProcessArgs),
    /// 管道管理
    Pipeline(pipeline::PipelineArgs),
    /// 蓝图管理
    Blueprint(blueprint::BlueprintArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Transfer(args) => transfer::run(args),
        Commands::Process(args) => process::run(args),
        Commands::Pipeline(args) => pipeline::run(args),
        Commands::Blueprint(args) => blueprint::run(args),
    }
}
