//! 人物卡 MCP 工具与临时导出存储，不依赖 HTTP 框架或模型供应商。

mod exports;
mod server;

pub use exports::{CardExports, ExportResult};
pub use server::CharacterCardMcp;

pub const EXPORT_TOOL_NAME: &str = "character_card_export";
