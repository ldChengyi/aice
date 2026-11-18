//! 编译时嵌入 Excel 模板资源，运行时不依赖源码路径或当前目录。

/// 已恢复为默认空白状态的 CoC7 半自动人物卡模板字节。
pub const COC7: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/coc7-character-card-template.xlsx"
));
