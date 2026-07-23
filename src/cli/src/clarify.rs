use clap::{Args, Subcommand};
use std::path::Path;

use crate::blueprint_core;

#[derive(Args)]
pub struct ClarifyArgs {
    #[command(subcommand)]
    pub action: ClarifyAction,
}

#[derive(Subcommand)]
pub enum ClarifyAction {
    /// 从聊天记录/上下文生成数据需求文档（DRD）
    FromChat {
        /// 输入的聊天记录文件路径（.txt 或 .md）
        input: String,
    },
}

pub fn run(args: &ClarifyArgs) {
    match &args.action {
        ClarifyAction::FromChat { input } => cmd_from_chat(input),
    }
}

fn cmd_from_chat(input: &str) {
    let chat_path = Path::new(input);
    let chat_content = std::fs::read_to_string(chat_path).unwrap_or_else(|e| {
        eprintln!("无法读取文件 {}: {e}", input);
        std::process::exit(1);
    });

    let prompt = blueprint_core::clarify_prompt(&chat_content);
    let llm = quanttide_agent::LLM::default();
    let messages = vec![quanttide_agent::Message::new("user", &prompt)];

    println!("正在分析聊天记录，生成 DRD ...");
    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
        Ok(resp) => {
            let drd_content = resp.content;
            let drd_dir = blueprint_core::drd_dir();
            std::fs::create_dir_all(&drd_dir).unwrap_or_else(|e| {
                eprintln!("无法创建目录 {drd_dir}: {e}");
                std::process::exit(1);
            });

            // Use the chat filename stem as DRD name
            let stem = chat_path.file_stem().unwrap_or_default().to_string_lossy();
            let output_path = Path::new(&drd_dir).join(format!("{stem}.md"));
            std::fs::write(&output_path, &drd_content).unwrap_or_else(|e| {
                eprintln!("写入 DRD 失败: {e}");
                std::process::exit(1);
            });
            println!("已生成: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("LLM 调用失败: {e}");
            std::process::exit(1);
        }
    }
}
