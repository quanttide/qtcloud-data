use clap::{Args, Subcommand};

#[derive(Args)]
pub struct TransferArgs {
    #[command(subcommand)]
    pub action: TransferAction,
}

#[derive(Subcommand)]
pub enum TransferAction {
    /// 发送文件：上传到网盘并生成分享链接，把链接给对方
    Send {
        /// 本地文件路径
        file: String,
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
    let token =
        std::env::var("DROPBOX_ACCESS_TOKEN").expect("请设置 DROPBOX_ACCESS_TOKEN 环境变量");

    match &args.action {
        TransferAction::Send { file, output } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(send(&token, file, output));
        }
        TransferAction::Receive { url, output } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(receive(&token, url, output));
        }
    }
}

async fn send(token: &str, file: &str, output: &Option<String>) {
    let remote = format!(
        "/Customers/send/{}",
        file.rsplit('/').next().unwrap_or("result")
    );
    super::dropbox::upload(token, file, &remote).await;

    match super::dropbox::create_shared_link(token, &remote).await {
        Ok(url) => {
            if let Some(out) = output {
                std::fs::write(out, &url).expect("写入链接文件失败");
                println!("✓ 链接已写入: {out}");
            } else {
                println!("{url}");
            }
        }
        Err(e) => eprintln!("⚠ 上传成功，但生成分享链接失败: {e}"),
    }
}

async fn receive(token: &str, url: &str, output: &Option<String>) {
    let path = output
        .clone()
        .unwrap_or_else(|| url.rsplit('/').next().unwrap_or("received").to_string());
    super::dropbox::download_and_save(token, url, &path).await;
}
