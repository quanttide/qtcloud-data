use clap::{Args, Subcommand};

use crate::providers;

#[derive(Args)]
pub struct TransferArgs {
    /// 网盘提供商: dropbox（默认）| baidu | google | onedrive | quark
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
    /// 接收文件：从共享链接下载并保存到本地
    Receive {
        /// 共享链接
        url: String,
        /// 本地保存路径，不指定则自动取名
        #[arg(long)]
        output: Option<String>,
    },
}

pub fn run(args: &TransferArgs) {
    // 选择提供商：优先 --provider 参数，receive 时也可从 URL 自动识别
    let provider: Box<dyn providers::StorageProvider> = if args.action.is_receive() {
        // receive 时尝试从 URL 自动识别
        let url = args.action.url();
        providers::detect(url).unwrap_or_else(|| {
            providers::from_name(&args.provider)
                .expect(&format!("不支持的提供商: {}", args.provider))
        })
    } else {
        providers::from_name(&args.provider).expect(&format!("不支持的提供商: {}", args.provider))
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    match &args.action {
        TransferAction::Send {
            file,
            remote,
            output,
        } => {
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
        TransferAction::Receive { url, output } => {
            let local_path = output
                .clone()
                .unwrap_or_else(|| url.rsplit('/').next().unwrap_or("received").to_string());

            if let Err(e) = rt.block_on(provider.receive(url, &local_path)) {
                eprintln!("接收失败: {e}");
            }
        }
    }
}

impl TransferAction {
    fn is_receive(&self) -> bool {
        matches!(self, TransferAction::Receive { .. })
    }

    fn url(&self) -> &str {
        match self {
            TransferAction::Receive { url, .. } => url,
            _ => "",
        }
    }
}
