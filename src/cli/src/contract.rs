use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub action: ContractAction,
}

#[derive(Subcommand)]
pub enum ContractAction {
    /// 查看 blueprint 的输入契约
    Input {
        /// blueprint 名称
        name: String,
    },
    /// 查看 blueprint 的输出契约（含验收规则）
    Output {
        /// blueprint 名称
        name: String,
    },
    /// 查看 blueprint 的完整契约
    Show {
        /// blueprint 名称
        name: String,
    },
}

pub fn run(args: &ContractArgs) {
    let dir = std::env::var("BLUEPRINTS_DIR").unwrap_or_else(|_| "./blueprints".to_string());

    match &args.action {
        ContractAction::Input { name } => {
            show_contract(&dir, name, "contract.input");
        }
        ContractAction::Output { name } => {
            show_contract(&dir, name, "contract.output");
        }
        ContractAction::Show { name } => {
            show_contract(&dir, name, "contract");
        }
    }
}

fn show_contract(dir: &str, name: &str, expression: &str) {
    let key = super::process::to_camel(name);
    let output = Command::new("cue")
        .args([
            "export",
            "--out",
            "yaml",
            "--expression",
            &format!("{key}.{expression}"),
            dir,
        ])
        .output()
        .expect("需要 cue (v0.16+)");
    if !output.status.success() {
        eprintln!("找不到 Blueprint: {name}");
        std::process::exit(1);
    }
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
