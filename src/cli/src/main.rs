use clap::{Parser, Subcommand};
use qtcloud_data_cli::{
    blueprint, catalog, clarify, contract, design, pipeline, process, review, transfer, version,
};

#[derive(Parser)]
#[command(name = "qtcloud-data", about = "量潮数据云 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 从客户上下文澄清需求 → 生成 DRD（数据需求文档）
    Clarify(clarify::ClarifyArgs),
    /// 设计 Specification（Contract + Blueprint）← 从 DRD
    Design(design::DesignArgs),
    /// 审计 Specification 完整性和一致性
    Review(review::ReviewArgs),
    /// 版本管理（list / show / diff）
    Version(version::VersionArgs),
    /// 蓝图管理（list / show）
    Blueprint(blueprint::BlueprintArgs),
    /// 契约查看
    Contract(contract::ContractArgs),
    /// 管道管理
    Pipeline(pipeline::PipelineArgs),
    /// 数据目录
    Catalog(catalog::CatalogArgs),
    /// 编排流程（receive → pipeline → send）
    Process(process::ProcessArgs),
    /// 数据传输（send / receive）
    Transfer(transfer::TransferArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clarify(args) => clarify::run(args),
        Commands::Design(args) => design::run(args),
        Commands::Review(args) => review::run(args),
        Commands::Version(args) => version::run(args),
        Commands::Blueprint(args) => blueprint::run(args),
        Commands::Contract(args) => contract::run(args),
        Commands::Pipeline(args) => pipeline::run(args),
        Commands::Catalog(args) => catalog::run(args),
        Commands::Process(args) => process::run(args),
        Commands::Transfer(args) => transfer::run(args),
    }
}
