//! 服务端渲染页面与静态资源，样式在编译时嵌入程序。

pub(crate) mod pages;

/// 内嵌样式资源，部署时无需额外复制 CSS 文件。
pub(crate) const PERSONAS_CSS: &str = include_str!("styles/personas.css");
