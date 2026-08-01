use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use crate::{blueprint_core, spec};

#[derive(Args)]
pub struct DesignArgs {
    #[command(subcommand)]
    pub action: DesignAction,
}

#[derive(Subcommand)]
pub enum DesignAction {
    /// 从 DRD 生成数据契约（Contract: .yaml + .md）
    Contract {
        /// DRD .md 文件路径
        input: String,
    },
    /// 从 DRD 生成处理蓝图（Blueprint: .yaml + .md + .html）
    Blueprint {
        /// DRD .md 文件路径
        input: String,
    },
    /// 将 Markdown 形式化为 YAML 结构化定义
    Formalize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 从 YAML 生成可视化 HTML 页面
    Preview {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub struct DesignHandler {
    llm: quanttide_agent::LLM,
}

impl DesignHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self {
        Self { llm }
    }

    pub fn run(&self, args: &DesignArgs) {
        match &args.action {
            DesignAction::Contract { input } => self.cmd_contract(input),
            DesignAction::Blueprint { input } => self.cmd_blueprint(input),
            DesignAction::Formalize { input, output } => self.cmd_formalize(input, output),
            DesignAction::Preview { input, output } => self.cmd_preview(input, output),
        }
    }

    // ── Contract: LLM outputs Markdown table, code generates YAML ──

    fn cmd_contract(&self, input: &str) {
        let drd = read_drd(input);
        let prompt = blueprint_core::design_contract_prompt(&drd);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        let stem = Path::new(input)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("正在从 DRD 生成 Contract: {stem} ...");

        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let (yaml_content, md_content) =
                    blueprint_core::contract_tables_to_yaml(&resp.content);
                write_spec_files(&stem, "contract", &yaml_content, &md_content);
            }
            Err(e) => {
                eprintln!("LLM 调用失败: {e}");
                std::process::exit(1);
            }
        }
    }

    // ── Blueprint: LLM outputs Markdown table, code generates YAML ──

    fn cmd_blueprint(&self, input: &str) {
        let drd = read_drd(input);
        let prompt = blueprint_core::design_blueprint_prompt(&drd);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        let stem = Path::new(input)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("正在从 DRD 生成 Blueprint: {stem} ...");

        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let (yaml_content, md_content) =
                    blueprint_core::blueprint_table_to_yaml(&resp.content, &stem);
                write_spec_files(&stem, "blueprint", &yaml_content, &md_content);

                // Generate HTML preview from YAML
                let bp: quanttide_data::Blueprint = serde_yaml::from_str(&yaml_content)
                    .unwrap_or_else(|e| {
                        eprintln!("解析 YAML 失败: {e}");
                        std::process::exit(1);
                    });
                let step_refs: Vec<(&str, &str, &str, &str)> = bp
                    .pipeline
                    .steps
                    .iter()
                    .map(|s| {
                        (
                            s.name.as_str(),
                            s.from.as_str(),
                            s.to.as_str(),
                            s.desc.as_str(),
                        )
                    })
                    .collect();
                let html = blueprint_core::render_html(
                    &bp.name,
                    bp.description.as_deref(),
                    bp.status.as_str(),
                    &bp.created_at,
                    &bp.updated_at,
                    "",
                    "",
                    &step_refs,
                );
                let spec_dir = blueprint_core::spec_dir();
                let html_path = Path::new(&spec_dir).join(format!("{stem}-blueprint.html"));
                std::fs::write(&html_path, &html).unwrap_or_else(|e| {
                    eprintln!("写入 .html 失败: {e}");
                    std::process::exit(1);
                });
                println!("已生成: {}", html_path.display());
            }
            Err(e) => {
                eprintln!("LLM 调用失败: {e}");
                std::process::exit(1);
            }
        }
    }

    // ── Formalize ──

    fn cmd_formalize(&self, input: &str, output: &Option<String>) {
        let md_path = Path::new(input);
        let md_content = std::fs::read_to_string(md_path).unwrap_or_else(|e| {
            eprintln!("无法读取 .md 文件: {e}");
            std::process::exit(1);
        });

        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = md_path.file_stem().unwrap_or_default().to_string_lossy();
                Path::new(&blueprint_core::spec_dir()).join(format!("{stem}.yaml"))
            }
        };

        let prompt = blueprint_core::design_formalize_prompt(&md_content);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        println!("正在形式化 {} ...", md_path.display());
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let yaml_code = blueprint_core::extract_cue(&resp.content);
                std::fs::write(&output_path, &yaml_code).unwrap_or_else(|e| {
                    eprintln!("写入 .yaml 失败: {e}");
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

    // ── Preview ──

    fn cmd_preview(&self, input: &str, output: &Option<String>) {
        let yaml_path = Path::new(input);
        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = yaml_path.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{stem}.html"))
            }
        };

        let yaml_content = std::fs::read_to_string(yaml_path).unwrap_or_else(|e| {
            eprintln!("无法读取 .yaml: {e}");
            std::process::exit(1);
        });

        let bp = spec::load_blueprint_from_yaml(&yaml_content).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

        let step_refs: Vec<(&str, &str, &str, &str)> = bp
            .pipeline
            .steps
            .iter()
            .map(|s| {
                (
                    s.name.as_str(),
                    s.from.as_str(),
                    s.to.as_str(),
                    s.desc.as_str(),
                )
            })
            .collect();
        let html = blueprint_core::render_html(
            &bp.name,
            bp.description.as_deref(),
            bp.status.as_str(),
            &bp.created_at,
            &bp.updated_at,
            "",
            "",
            &step_refs,
        );
        std::fs::write(&output_path, &html).unwrap_or_else(|e| {
            eprintln!("写入 .html 失败: {e}");
            std::process::exit(1);
        });
        println!("已生成: {}", output_path.display());
    }
}

// ── Helpers ──

fn read_drd(input: &str) -> String {
    std::fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("无法读取 DRD 文件 {input}: {e}");
        std::process::exit(1);
    })
}

fn write_spec_files(stem: &str, kind: &str, yaml: &str, md: &str) {
    let spec_dir = blueprint_core::spec_dir();
    std::fs::create_dir_all(&spec_dir).unwrap_or_else(|e| {
        eprintln!("无法创建目录 {spec_dir}: {e}");
        std::process::exit(1);
    });
    let yaml_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.yaml"));
    let md_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.md"));
    std::fs::write(&yaml_path, yaml).unwrap_or_else(|e| {
        eprintln!("写入 .yaml 失败: {e}");
        std::process::exit(1);
    });
    std::fs::write(&md_path, md).unwrap_or_else(|e| {
        eprintln!("写入 .md 失败: {e}");
        std::process::exit(1);
    });
    println!("已生成: {}", yaml_path.display());
    println!("已生成: {}", md_path.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::fake_llm;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    const CONTRACT_TABLES: &str = "## 输入契约\n\n| 字段名 | 类型 | 说明 |\n|--------|------|------|\n| user_id | string | 用户 ID |\n\n## 输出契约\n\n| 字段名 | 类型 | 说明 |\n|--------|------|------|\n| repo | string | 仓库名 |\n| stars | int | 星数 |\n";

    #[test]
    fn design_contract_writes_spec_files_from_llm_tables() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_root("qtcloud-design-contract");
        let drd = root.join("abc.md");
        std::fs::write(&drd, "# DRD ABC\n").unwrap();
        let spec_dir = root.join("spec");

        unsafe {
            std::env::set_var("SPEC_DIR", &spec_dir);
        }
        let handler = DesignHandler::new(fake_llm(CONTRACT_TABLES));
        handler.run(&DesignArgs {
            action: DesignAction::Contract {
                input: drd.to_string_lossy().to_string(),
            },
        });
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        let yaml = std::fs::read_to_string(spec_dir.join("abc-contract.yaml")).unwrap();
        assert!(yaml.contains("contract:"), "{yaml}");
        assert!(yaml.contains("user_id"), "{yaml}");
        let md = std::fs::read_to_string(spec_dir.join("abc-contract.md")).unwrap();
        assert!(md.contains("输入契约"), "{md}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn design_formalize_writes_yaml_from_cue_block() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_root("qtcloud-design-formalize");
        let md = root.join("note.md");
        std::fs::write(&md, "# 需求\n").unwrap();
        let output = root.join("out.yaml");

        let handler = DesignHandler::new(fake_llm("```cue\nname: \"demo\"\nversion: 1\n```\n"));
        handler.run(&DesignArgs {
            action: DesignAction::Formalize {
                input: md.to_string_lossy().to_string(),
                output: Some(output.to_string_lossy().to_string()),
            },
        });

        let yaml = std::fs::read_to_string(&output).unwrap();
        assert!(yaml.contains("demo"), "{yaml}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn design_preview_renders_html_from_blueprint_yaml() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_root("qtcloud-design-preview");
        let yaml_in = root.join("bp.yaml");
        std::fs::write(
            &yaml_in,
            "name: demo\nstatus: draft\ndescription: 示例\ncreated_at: \"2026-01-01\"\nupdated_at: \"2026-01-01\"\ncontract:\n  input:\n    schema: a\n    format: CSV\n  output:\n    schema: b\n    format: CSV\npipeline:\n  name: demo-pipeline\n  steps:\n    - name: step1\n      from: \"[]\"\n      to: \"[]\"\n      desc: 第一步\n",
        )
        .unwrap();
        let output = root.join("out.html");

        let handler = DesignHandler::new(fake_llm("unused"));
        handler.run(&DesignArgs {
            action: DesignAction::Preview {
                input: yaml_in.to_string_lossy().to_string(),
                output: Some(output.to_string_lossy().to_string()),
            },
        });

        let html = std::fs::read_to_string(&output).unwrap();
        assert!(html.contains("demo"), "{html}");

        std::fs::remove_dir_all(&root).ok();
    }
}
