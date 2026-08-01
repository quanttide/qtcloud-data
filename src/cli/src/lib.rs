pub mod blueprint;
pub mod blueprint_core;
pub mod catalog;
pub mod clarify;
pub mod contract;
pub mod design;
pub mod doctor;
pub mod error;
pub mod implement;
pub mod pipeline;
pub mod process;
pub mod providers;
pub mod review;
pub mod spec;
pub mod store;
pub mod transfer;
pub mod version;

/// 测试共享的全局环境变量锁：各模块测试直接 `std::env::set_var` 时统一互斥，
/// 避免并行执行互相污染进程级环境变量。
#[cfg(test)]
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
