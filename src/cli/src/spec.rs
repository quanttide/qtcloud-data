use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

pub fn run(args: &SpecArgs) {
    match &args.action {
        SpecAction::Wrap { input, output } => wrap_file(input, output),
        SpecAction::Validate { input } => validate_file(input),
    }
}

pub fn wrap_blueprint_yaml(yaml: &str, source_path: Option<&str>) -> Result<String, String> {
    let blueprint = load_blueprint_from_yaml(yaml)?;
    let spec = Specification::from_blueprint(blueprint, source_path);
    serde_yaml::to_string(&spec).map_err(|err| format!("序列化 Specification 失败: {err}"))
}

pub fn load_blueprint_from_yaml(yaml: &str) -> Result<quanttide_data::Blueprint, String> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|err| format!("解析 YAML 失败: {err}"))?;

    if is_specification_envelope(&value) {
        return parse_specification_yaml(yaml).map(|spec| spec.spec.blueprint);
    }

    serde_yaml::from_value(value).map_err(|err| format!("解析 Blueprint YAML 失败: {err}"))
}

pub fn parse_specification_yaml(yaml: &str) -> Result<Specification, String> {
    let spec: Specification =
        serde_yaml::from_str(yaml).map_err(|err| format!("解析 Specification YAML 失败: {err}"))?;

    if spec.api_version != SPEC_API_VERSION {
        return Err(format!(
            "不支持的 api_version: {}，期望 {}",
            spec.api_version, SPEC_API_VERSION
        ));
    }
    if spec.kind != SPEC_KIND {
        return Err(format!("不支持的 kind: {}，期望 {}", spec.kind, SPEC_KIND));
    }

    Ok(spec)
}

fn wrap_file(input: &str, output: &Option<String>) {
    let content = std::fs::read_to_string(input).unwrap_or_else(|err| {
        eprintln!("无法读取 YAML: {err}");
        std::process::exit(1);
    });

    let wrapped = wrap_blueprint_yaml(&content, Some(input)).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });

    let output_path = output
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_spec_output_path(input));

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).unwrap_or_else(|err| {
            eprintln!("无法创建输出目录: {err}");
            std::process::exit(1);
        });
    }

    std::fs::write(&output_path, wrapped).unwrap_or_else(|err| {
        eprintln!("写入 Specification YAML 失败: {err}");
        std::process::exit(1);
    });
    println!("已生成: {}", output_path.display());
}

fn validate_file(input: &str) {
    let content = std::fs::read_to_string(input).unwrap_or_else(|err| {
        eprintln!("无法读取 YAML: {err}");
        std::process::exit(1);
    });

    let blueprint = load_blueprint_from_yaml(&content).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });

    if let Err(errors) = quanttide_data::validate(&blueprint) {
        eprintln!("Specification 结构校验失败:");
        for err in errors {
            eprintln!("  - {}: {}", err.field, err.message);
        }
        std::process::exit(1);
    }

    println!("Specification OK: {}", blueprint.name);
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
    fn rejects_unknown_specification_kind() {
        let yaml = wrap_blueprint_yaml(blueprint_yaml(), None)
            .unwrap()
            .replace("kind: Specification", "kind: Unknown");

        let err = load_blueprint_from_yaml(&yaml).unwrap_err();

        assert!(err.contains("kind"));
    }
}
