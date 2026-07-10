use clap::{Args, Subcommand};

use crate::providers;

#[derive(Args)]
pub struct TransferArgs {
    /// 网盘提供商: dropbox（默认）| baidu | google | onedrive | s3
    #[arg(long, default_value = "dropbox")]
    pub provider: String,

    #[command(subcommand)]
    pub action: TransferAction,
}

#[derive(Subcommand)]
pub enum TransferAction {
    /// 发送文件：上传到网盘并生成分享链接，把链接给对方
    Send {
        /// 本地文件路径
        file: String,
        /// 远程路径，不指定则使用文件名
        remote: Option<String>,
        /// 将链接写入文件（不指定则直接打印到终端）
        #[arg(long)]
        output: Option<String>,
    },
    /// 接收文件：从共享链接下载（手动）或直接拉取（自动）
    ///
    /// 手动模式：传入分享链接（http/https），自动识别提供商
    /// 自动模式：传入远程路径，配合 --provider 使用
    Receive {
        /// 分享链接（http/https）或远程路径
        source: String,
        /// 本地保存路径，不指定则自动取名
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn run(args: &TransferArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match &args.action {
        TransferAction::Send {
            file,
            remote,
            output,
        } => {
            let provider = providers::from_name(&args.provider)
                .expect(&format!("不支持的提供商: {}", args.provider));

            let remote_path = remote.clone().unwrap_or_else(|| {
                format!("/send/{}", file.rsplit('/').next().unwrap_or("result"))
            });

            match rt.block_on(provider.send(file, &remote_path)) {
                Ok(link) => {
                    if let Some(out) = output {
                        std::fs::write(out, &link).expect("写入链接文件失败");
                        println!("✓ 链接已写入: {out}");
                    } else {
                        println!("{link}");
                    }
                }
                Err(e) => eprintln!("发送失败: {e}"),
            }
        }
        TransferAction::Receive { source, output } => {
            let local_path = output
                .clone()
                .unwrap_or_else(|| source.rsplit('/').next().unwrap_or("received").to_string());

            let is_url = source.starts_with("http://") || source.starts_with("https://");

            if is_url {
                // 手动模式：从 URL 自动识别提供商
                let provider = providers::detect(source).unwrap_or_else(|| {
                    providers::from_name(&args.provider)
                        .expect(&format!("不支持的提供商: {}", args.provider))
                });
                if let Err(e) = rt.block_on(provider.receive(source, &local_path)) {
                    eprintln!("接收失败: {e}");
                }
            } else {
                // 自动模式：使用 --provider 指定的提供商直接拉取
                let provider = providers::from_name(&args.provider)
                    .expect(&format!("不支持的提供商: {}", args.provider));
                if let Err(e) = rt.block_on(provider.receive_path(source, &local_path)) {
                    eprintln!("自动接收失败: {e}");
                }
            }
        }
    }
}
