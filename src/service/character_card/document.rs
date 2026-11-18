use super::CardTemplate;
use crate::infra::xlsx::{self, CellEdit, CellValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type CardChanges = BTreeMap<String, CellValue>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Text,
    Integer,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardField {
    pub id: String,
    pub label: String,
    pub kind: FieldKind,
    #[serde(skip)]
    pub worksheet: String,
    #[serde(skip)]
    pub cell: String,
}

/// Common lifecycle for character cards. Updates are atomic and never modify the template.
pub trait CharacterCardDocument: Send + Sync {
    fn create(template: &'static CardTemplate) -> Result<Self, String>
    where
        Self: Sized;
    fn fields(&self) -> &[CardField];
    fn update(&mut self, changes: &CardChanges) -> Result<(), String>;
    fn export(&self) -> Result<Vec<u8>, String>;
}

/// Shared XLSX implementation; a template supplies only its editable field mapping.
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
                (_, CellValue::Empty) | (FieldKind::Integer, CellValue::Integer(_)) => {},
                (FieldKind::Text, CellValue::Text(text)) => {
                    if text.encode_utf16().count() > 32767 || text.chars().any(|c| !matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')) {
                        return Err(format!("字段 {id} 超过 Excel 文本长度限制或含无效字符。"));
                    }
                },
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
