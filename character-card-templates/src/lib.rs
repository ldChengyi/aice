//! Workbook resources embedded at compile time, independent of the runtime directory.

pub const COC7: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/coc7-character-card-template.xlsx"
));
