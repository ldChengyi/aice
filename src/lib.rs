//! Aice 应用公共入口，连接配置、HTTP 接口、业务编排和外部服务。

pub mod config;
pub mod dal;
mod frontend;
mod handle;
pub mod infra;
pub mod init;
pub mod logic;
pub mod router;
pub mod service;
