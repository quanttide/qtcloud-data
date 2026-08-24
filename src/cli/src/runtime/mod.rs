//! 语言运行时适配：codegen（`implement` 用）+ 执行（`process` 用），注册表驱动。
//!
//! 与 `storage/` 同构：trait + `from_name`/`from_ext` 注册表 + 注入式 mock（`execute_with`）。
//! 模块名 = 概念名：`runtime::Runtime`、`runtime::PythonRuntime`（↔ `storage::Storage`、`storage::DropboxStorage`）。

pub mod bash;
pub mod builtin;
pub mod matlab;
pub mod python;
pub mod r;
pub mod stata;

use std::path::Path;

/// 语言运行时公共接口
///
/// 两个职责：
/// - **codegen**：为语言生成代码的 prompt 与提取规则（`implement` 命令用）
/// - **execute**：执行脚本（`process` 命令用）
pub trait Runtime: Send + Sync {
    /// 运行时名称（`--lang` 参数值）
    fn name(&self) -> &'static str;

    /// 脚本文件扩展名（不含点）
    fn extension(&self) -> &'static str;

    /// 默认执行命令
    fn command(&self) -> &'static str;

    /// 外部解释器名称；builtin 等内置运行时不需要 PATH 检查。
    fn doctor_command(&self) -> Option<&'static str> {
        Some(self.command())
    }

    // ── codegen（默认不支持，codegen 语言覆盖）──

    /// 单步实现 prompt
    fn step_prompt(
        &self,
        _step_name: &str,
        _from_desc: &str,
        _to_desc: &str,
        _step_desc: &str,
        _prev_functions: &str,
    ) -> String {
        String::new()
    }

    /// 组装 prompt
    fn assemble_prompt(
        &self,
        _project_name: &str,
        _all_functions: &str,
        _pipeline_desc: &str,
    ) -> String {
        String::new()
    }

    /// 从 LLM 响应提取代码
    fn extract(&self, _response: &str) -> String {
        String::new()
    }

    /// 提取函数签名（供下一步上下文）
    fn extract_signature(&self, _code: &str, _step_name: &str) -> String {
        String::new()
    }

    /// 名称转函数命名规范
    fn to_snake(&self, _s: &str) -> String {
        String::new()
    }

    // ── execute ──

    /// 执行脚本：`{cmd} {script} {input} {output}`（cwd = work_dir）
    fn execute(
        &self,
        script: &Path,
        input: &str,
        output: &str,
        work_dir: &str,
    ) -> Result<String, String> {
        self.execute_with(script, input, output, work_dir, self.command())
    }

    /// 注入式执行（mock）：测试可用 fake 命令替换真实解释器，类似 `storage` 的 `*_with_base`
    fn execute_with(
        &self,
        script: &Path,
        input: &str,
        output: &str,
        work_dir: &str,
        cmd: &str,
    ) -> Result<String, String>;
}

/// 按名称创建运行时（codegen 语言注册表，`implement --lang` 用）
pub fn from_name(name: &str) -> Option<Box<dyn Runtime>> {
    match name {
        "python" => Some(Box::new(python::PythonRuntime)),
        "r" => Some(Box::new(r::RRuntime)),
        "stata" => Some(Box::new(stata::StataRuntime)),
        "matlab" => Some(Box::new(matlab::MatlabRuntime)),
        _ => None,
    }
}

/// 按扩展名创建运行时（执行注册表，`process` 用）
pub fn from_ext(ext: &str) -> Option<Box<dyn Runtime>> {
    match ext {
        "py" => Some(Box::new(python::PythonRuntime)),
        "r" => Some(Box::new(r::RRuntime)),
        "do" => Some(Box::new(stata::StataRuntime)),
        "m" => Some(Box::new(matlab::MatlabRuntime)),
        "sh" => Some(Box::new(bash::BashRuntime)),
        _ => None,
    }
}

/// 返回所有已注册运行时，供 doctor 和集成层消费。
pub fn registered() -> Vec<Box<dyn Runtime>> {
    vec![
        Box::new(python::PythonRuntime),
        Box::new(r::RRuntime),
        Box::new(stata::StataRuntime),
        Box::new(matlab::MatlabRuntime),
        Box::new(bash::BashRuntime),
        Box::new(builtin::BuiltinRuntime),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_registers_codegen_runtimes() {
        assert!(from_name("python").is_some());
        assert!(from_name("r").is_some());
        assert!(from_name("stata").is_some());
        assert!(from_name("matlab").is_some());
        assert!(from_name("bash").is_none(), "bash 仅执行，不做 codegen");
    }

    #[test]
    fn from_ext_registers_execution_runtimes() {
        assert!(from_ext("py").is_some());
        assert!(from_ext("r").is_some());
        assert!(from_ext("do").is_some());
        assert!(from_ext("m").is_some());
        assert!(from_ext("sh").is_some());
        assert!(from_ext("csv").is_none());
    }

    #[test]
    fn registered_includes_builtin_without_exposing_it_as_codegen() {
        let names: Vec<_> = registered().iter().map(|runtime| runtime.name()).collect();

        assert_eq!(
            names,
            vec!["python", "r", "stata", "matlab", "bash", "builtin"]
        );
        assert!(from_name("builtin").is_none());
        assert!(
            registered()
                .iter()
                .find(|runtime| runtime.name() == "builtin")
                .unwrap()
                .doctor_command()
                .is_none()
        );
    }
}
