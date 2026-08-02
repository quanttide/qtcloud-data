//! 生命周期阶段模块（纵向流程）：需求澄清 → 规格设计 → 代码实现 → 流程编排 → 数据传输。
//!
//! 产物逐级流转：DRD → Specification → 代码 → 编排 → 交付。

pub mod clarify;
pub mod design;
pub mod implement;
pub mod process;
pub mod transfer;
