//! 统一的人物卡创建、字段校验、修改和 Excel 导出接口。

use super::CardTemplate;
use crate::xlsx::{self, CellEdit, CellValue};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;

pub type CardChanges = BTreeMap<String, CellValue>;

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Text,
    Integer,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CardField {
    pub id: String,
    pub label: String,
    pub kind: FieldKind,
    #[serde(skip)]
    pub worksheet: String,
    #[serde(skip)]
    pub cell: String,
}

/// 统一的人物卡生命周期；整批修改先校验再应用，不改写原始模板。
pub trait CharacterCardDocument: Send + Sync {
    /// 根据模板创建独立实例；字段 ID 或单元格位置重复时返回错误。
    fn create(template: &'static CardTemplate) -> Result<Self, String>
    where
        Self: Sized;
    /// 借用可编辑字段列表；返回切片，不复制映射。
    fn fields(&self) -> &[CardField];
    /// 先校验整批修改，再覆盖同名字段；未传字段保留，空值清空。
    fn update(&mut self, changes: &CardChanges) -> Result<(), String>;
    /// 将当前修改写入模板副本，返回 Excel 字节，不修改原模板。
    fn export(&self) -> Result<Vec<u8>, String>;
}

/// 通用 Excel 人物卡实现，由模板提供可编辑字段与单元格的映射。
pub struct ExcelCharacterCard {
    template: &'static CardTemplate,
    fields: Vec<CardField>,
    changes: CardChanges,
}

impl CharacterCardDocument for ExcelCharacterCard {
    fn create(template: &'static CardTemplate) -> Result<Self, String> {
        let fields = (template.field_mapping)(template.bytes)?;
        let mut ids = std::collections::HashSet::new();
        let mut cells = std::collections::HashSet::new();
        for field in &fields {
            if !ids.insert(&field.id) || !cells.insert((&field.worksheet, &field.cell)) {
                return Err(format!("模版存在重复字段映射：{}", field.id));
            }
        }
        Ok(Self {
            template,
            fields,
            changes: BTreeMap::new(),
        })
    }

    fn fields(&self) -> &[CardField] {
        &self.fields
    }

    fn update(&mut self, changes: &CardChanges) -> Result<(), String> {
        for (id, value) in changes {
            let field = self
                .fields
                .iter()
                .find(|field| field.id == *id)
                .ok_or_else(|| format!("未知或不可修改的人物卡字段：{id}"))?;
            match (field.kind, value) {
                (_, CellValue::Empty) | (FieldKind::Integer, CellValue::Integer(_)) => {}
                (FieldKind::Text, CellValue::Text(text)) => {
                    // Excel 按 UTF-16 码元限制文本长度，同时拒绝 XML 不允许的字符。
                    if text.encode_utf16().count() > 32767 || text.chars().any(|c| !matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')) {
                        return Err(format!("字段 {id} 超过 Excel 文本长度限制或含无效字符。"));
                    }
                }
                _ => return Err(format!("字段 {id} 的类型不正确。")),
            }
        }
        self.changes.extend(changes.clone());
        Ok(())
    }

    fn export(&self) -> Result<Vec<u8>, String> {
        let edits = self
            .fields
            .iter()
            .filter_map(|field| {
                self.changes.get(&field.id).map(|value| CellEdit {
                    worksheet: field.worksheet.as_str(),
                    cell: field.cell.as_str(),
                    value,
                })
            })
            .collect::<Vec<_>>();
        xlsx::export(self.template.bytes, &edits)
            .map_err(|error| format!("人物卡导出失败：{error}"))
    }
}
