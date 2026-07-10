use clap::{Parser, Subcommand};
use qtcloud_data_cli::{blueprint, contract, pipeline, process, transfer};

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
    /// 契约查看
    Contract(contract::ContractArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Transfer(args) => transfer::run(args),
        Commands::Process(args) => process::run(args),
        Commands::Pipeline(args) => pipeline::run(args),
        Commands::Blueprint(args) => blueprint::run(args),
        Commands::Contract(args) => contract::run(args),
    }
}
