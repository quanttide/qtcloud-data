//! Specification 工具命令：wrap / validate（envelope 契约）。

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::CliError;

pub const SPEC_API_VERSION: &str = "qtcloud.quanttide.com/v1alpha1";
pub const SPEC_KIND: &str = "Specification";
pub const SPEC_GENERATED_BY: &str = "qtcloud-data-cli";

#[derive(Args)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub action: SpecAction,
}

#[derive(Subcommand)]
pub enum SpecAction {
    /// 将已有 Blueprint YAML 包装成稳定 Specification YAML
    Wrap {
        /// Blueprint YAML 或 Specification YAML
        input: String,
        /// 输出路径；不指定则写到同目录的 *-spec.yaml
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 校验 Blueprint/Specification YAML 结构
    Validate {
        /// Blueprint YAML 或 Specification YAML
        input: String,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
// ── 数据模型（Specification 系列） ──
pub struct Specification {
    pub api_version: String,
    pub kind: String,
    pub metadata: SpecificationMetadata,
    pub spec: SpecificationBody,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpecificationMetadata {
    pub name: String,
    pub generated_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpecificationBody {
    pub blueprint: quanttide_data::Blueprint,
}

impl Specification {
    pub fn from_blueprint(blueprint: quanttide_data::Blueprint, source_path: Option<&str>) -> Self {
        Self {
            api_version: SPEC_API_VERSION.to_string(),
            kind: SPEC_KIND.to_string(),
            metadata: SpecificationMetadata {
                name: blueprint.name.clone(),
                generated_by: SPEC_GENERATED_BY.to_string(),
                source_path: source_path.map(|path| path.to_string()),
            },
            spec: SpecificationBody { blueprint },
        }
    }
}

// ── 命令（wrap / validate） ──
/// Specification 工具命令入口（wrap / validate）。
pub fn run(args: &SpecArgs) -> Result<(), CliError> {
    match &args.action {
        SpecAction::Wrap { input, output } => wrap_file(input, output),
        SpecAction::Validate { input } => validate_file(input),
    }
}

// ── 解析与包装（load / parse / wrap） ──
/// 把 Blueprint YAML 包装为 Specification envelope。
pub fn wrap_blueprint_yaml(yaml: &str, source_path: Option<&str>) -> Result<String, CliError> {
    let blueprint = load_blueprint_from_yaml(yaml)?;
    let spec = Specification::from_blueprint(blueprint, source_path);
    serde_yaml::to_string(&spec)
        .map_err(|err| CliError::new(format!("序列化 Specification 失败: {err}")))
}

/// 从 YAML 加载 Blueprint（兼容 envelope 与裸 blueprint 两种格式）。
pub fn load_blueprint_from_yaml(yaml: &str) -> Result<quanttide_data::Blueprint, CliError> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .map_err(|err| CliError::new(format!("解析 YAML 失败: {err}")))?;

    if is_specification_envelope(&value) {
        return parse_specification_yaml(yaml).map(|spec| spec.spec.blueprint);
    }

    serde_yaml::from_value(value)
        .map_err(|err| CliError::new(format!("解析 Blueprint YAML 失败: {err}")))
}

/// 解析并校验 Specification envelope（api_version / kind）。
pub fn parse_specification_yaml(yaml: &str) -> Result<Specification, CliError> {
    let spec: Specification = serde_yaml::from_str(yaml)
        .map_err(|err| CliError::new(format!("解析 Specification YAML 失败: {err}")))?;

    if spec.api_version != SPEC_API_VERSION {
        return Err(CliError::new(format!(
            "不支持的 api_version: {}，期望 {}",
            spec.api_version, SPEC_API_VERSION
        )));
    }
    if spec.kind != SPEC_KIND {
        return Err(CliError::new(format!(
            "不支持的 kind: {}，期望 {}",
            spec.kind, SPEC_KIND
        )));
    }

    Ok(spec)
}

fn wrap_file(input: &str, output: &Option<String>) -> Result<(), CliError> {
    let content = std::fs::read_to_string(input)
        .map_err(|err| CliError::new(format!("无法读取 YAML: {err}")))?;

    let wrapped = wrap_blueprint_yaml(&content, Some(input))?;

    let output_path = output
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_spec_output_path(input));

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|err| CliError::new(format!("无法创建输出目录: {err}")))?;
    }

    std::fs::write(&output_path, wrapped)
        .map_err(|err| CliError::new(format!("写入 Specification YAML 失败: {err}")))?;
    println!("已生成: {}", output_path.display());
    Ok(())
}

fn validate_file(input: &str) -> Result<(), CliError> {
    let content = std::fs::read_to_string(input)
        .map_err(|err| CliError::new(format!("无法读取 YAML: {err}")))?;

    let blueprint = load_blueprint_from_yaml(&content)?;

    if let Err(errors) = quanttide_data::validate(&blueprint) {
        let details = errors
            .iter()
            .map(|err| format!("  - {}: {}", err.field, err.message))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(CliError::new(format!(
            "Specification 结构校验失败:\n{details}"
        )));
    }

    println!("Specification OK: {}", blueprint.name);
    Ok(())
}

fn is_specification_envelope(value: &serde_yaml::Value) -> bool {
    let serde_yaml::Value::Mapping(map) = value else {
        return false;
    };

    map.contains_key(serde_yaml::Value::String("api_version".to_string()))
        || map.contains_key(serde_yaml::Value::String("kind".to_string()))
        || map.contains_key(serde_yaml::Value::String("spec".to_string()))
}

fn default_spec_output_path(input: &str) -> PathBuf {
    let input_path = Path::new(input);
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("specification");
    let base = stem.strip_suffix("-blueprint").unwrap_or(stem);
    let filename = format!("{base}-spec.yaml");

    input_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join(&filename))
        .unwrap_or_else(|| PathBuf::from(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blueprint_yaml() -> &'static str {
        r#"name: "sample"
description: "稳定规格测试"
contract:
  input:
    schema: "raw: string"
    format: "CSV"
  output:
    schema: "clean: string"
    format: "CSV"
    rules:
      - 字段非空
pipeline:
  name: "sample-pipeline"
  steps:
    - name: "clean"
      from: "raw"
      to: "clean"
      desc: "trim whitespace"
status: draft
created_at: "2026-07-30T00:00:00+00:00"
updated_at: "2026-07-30T00:00:00+00:00"
"#
    }

    #[test]
    fn wraps_blueprint_yaml_in_stable_specification_envelope() {
        let yaml = wrap_blueprint_yaml(blueprint_yaml(), Some("sample-blueprint.yaml")).unwrap();
        let spec: Specification = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(spec.api_version, SPEC_API_VERSION);
        assert_eq!(spec.kind, SPEC_KIND);
        assert_eq!(spec.metadata.name, "sample");
        assert_eq!(
            spec.metadata.source_path.as_deref(),
            Some("sample-blueprint.yaml")
        );
        assert_eq!(spec.metadata.generated_by, "qtcloud-data-cli");
        assert_eq!(spec.spec.blueprint.pipeline.name, "sample-pipeline");
    }

    #[test]
    fn load_blueprint_accepts_enveloped_and_legacy_yaml() {
        let enveloped =
            wrap_blueprint_yaml(blueprint_yaml(), Some("sample-blueprint.yaml")).unwrap();

        let from_enveloped = load_blueprint_from_yaml(&enveloped).unwrap();
        let from_legacy = load_blueprint_from_yaml(blueprint_yaml()).unwrap();

        assert_eq!(from_enveloped.name, "sample");
        assert_eq!(from_enveloped, from_legacy);
    }

    #[test]
    fn wrap_file_writes_spec_yaml_to_output() {
        let root = crate::test_support::temp_dir("qtcloud-spec-wrap-file");
        let input = root.join("abc-blueprint.yaml");
        std::fs::write(&input, blueprint_yaml()).unwrap();
        let output = root.join("out-spec.yaml");

        run(&SpecArgs {
            action: SpecAction::Wrap {
                input: input.to_string_lossy().to_string(),
                output: Some(output.to_string_lossy().to_string()),
            },
        })
        .unwrap();

        let content = std::fs::read_to_string(&output).unwrap();
        assert!(
            content.contains("api_version: qtcloud.quanttide.com/v1alpha1"),
            "{content}"
        );
        assert!(content.contains("kind: Specification"), "{content}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn wrap_file_reports_unreadable_input() {
        let root = crate::test_support::temp_dir("qtcloud-spec-wrap-missing");
        let err = run(&SpecArgs {
            action: SpecAction::Wrap {
                input: root.join("ghost.yaml").to_string_lossy().to_string(),
                output: None,
            },
        })
        .unwrap_err();
        assert!(err.to_string().contains("无法读取 YAML"), "{err}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn validate_file_accepts_valid_blueprint() {
        let root = crate::test_support::temp_dir("qtcloud-spec-validate-ok");
        let input = root.join("ok.yaml");
        std::fs::write(&input, blueprint_yaml()).unwrap();

        run(&SpecArgs {
            action: SpecAction::Validate {
                input: input.to_string_lossy().to_string(),
            },
        })
        .unwrap();

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn validate_file_rejects_invalid_blueprint() {
        let root = crate::test_support::temp_dir("qtcloud-spec-validate-bad");
        let input = root.join("bad.yaml");
        // 缺 pipeline.steps 等字段，结构校验失败
        std::fs::write(&input, "name: x\nstatus: draft\n").unwrap();

        let err = run(&SpecArgs {
            action: SpecAction::Validate {
                input: input.to_string_lossy().to_string(),
            },
        })
        .unwrap_err();
        assert!(err.to_string().contains("失败"), "{err}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn default_spec_output_path_replaces_blueprint_suffix() {
        assert_eq!(
            default_spec_output_path("spec/abc-blueprint.yaml"),
            PathBuf::from("spec/abc-spec.yaml")
        );
        assert_eq!(
            default_spec_output_path("abc.yaml"),
            PathBuf::from("abc-spec.yaml")
        );
        // 无扩展名输入回退 specification
        assert_eq!(
            default_spec_output_path("spec/abc"),
            PathBuf::from("spec/abc-spec.yaml")
        );
    }

    #[test]
    fn parse_specification_rejects_unknown_api_version() {
        let yaml = wrap_blueprint_yaml(blueprint_yaml(), None)
            .unwrap()
            .replace(
                "api_version: qtcloud.quanttide.com/v1alpha1",
                "api_version: example.com/v1",
            );
        let err = parse_specification_yaml(&yaml).unwrap_err();
        assert!(err.to_string().contains("api_version"), "{err}");
    }

    #[test]
    fn rejects_unknown_specification_kind() {
        let yaml = wrap_blueprint_yaml(blueprint_yaml(), None)
            .unwrap()
            .replace("kind: Specification", "kind: Unknown");

        let err = load_blueprint_from_yaml(&yaml).unwrap_err();

        assert!(err.to_string().contains("kind"));
    }
}
