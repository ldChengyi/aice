use calamine::{Data, Range, Reader, Xlsx};
use serde::Serialize;
use std::{collections::BTreeMap, io::Cursor};

mod coc7_fields;
mod document;
mod xlsx;
pub use document::{CardChanges, CardField, CharacterCardDocument, ExcelCharacterCard, FieldKind};
pub use xlsx::CellValue;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CardTemplate {
    pub id: &'static str,
    pub name: &'static str,
    #[serde(skip)]
    bytes: &'static [u8],
    #[serde(skip)]
    field_mapping: fn(&[u8]) -> Result<Vec<CardField>, String>,
}

pub const TEMPLATES: &[CardTemplate] = &[CardTemplate {
    id: "coc7",
    name: "CoC 7 Character Card",
    bytes: character_card_templates::COC7,
    field_mapping: coc7_fields::fields,
}];

impl CardTemplate {
    pub fn create(&'static self) -> Result<ExcelCharacterCard, String> {
        ExcelCharacterCard::create(self)
    }

    pub fn read(&self) -> Result<CharacterCard, String> {
        read_xlsx(self.bytes)
    }
}

pub fn read_default() -> Result<CharacterCard, String> {
    TEMPLATES[0].read()
}

#[derive(Debug, Serialize)]
pub struct CharacterCard {
    pub name: String,
    pub player: String,
    pub occupation: String,
    pub age: Option<u32>,
    pub gender: String,
    pub era: String,
    pub residence: String,
    pub birthplace: String,
    pub attributes: BTreeMap<String, Option<u32>>,
    pub hit_points: Points,
    pub sanity: Option<u32>,
    pub magic_points: Points,
    pub movement: Option<u32>,
    pub skills: Vec<Skill>,
    pub background: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct Points {
    pub current: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Skill {
    pub name: String,
    pub specialization: String,
    pub regular: Option<u32>,
    pub hard: Option<u32>,
    pub extreme: Option<u32>,
}

/// Reads the saved values of the project's CoC 7 simplified XLSX template.
/// Formulas are not recalculated; empty cells remain None rather than becoming zero.
pub fn read_xlsx(bytes: &[u8]) -> Result<CharacterCard, String> {
    let mut workbook = Xlsx::new(Cursor::new(bytes))
        .map_err(|_| "无法读取 Excel，请使用有效的 .xlsx 文件。".to_owned())?;
    let sheet = workbook
        .worksheet_range("人物卡")
        .map_err(|_| "无法读取「人物卡」工作表，请使用 CoC 七版精简版人物卡。".to_owned())?;
    parse_sheet(&sheet)
}

fn parse_sheet(sheet: &Range<Data>) -> Result<CharacterCard, String> {
    // shortcut: fixed cells support the supplied simplified template; add mappings when supporting other layouts.
    for (cell, expected) in [
        ("B3", "姓名"),
        ("B5", "职业"),
        ("B14", "技能表"),
        ("F15", "技能名称"),
        ("AB15", "技能名称"),
        ("S3", "力量\nSTR"),
    ] {
        if text(sheet, cell)? != expected {
            return Err(format!("人物卡版式不匹配：{cell} 应为「{expected}」。"));
        }
    }

    let mut attributes = BTreeMap::new();
    for (name, cell) in [
        ("STR", "U3"),
        ("CON", "U5"),
        ("SIZ", "U7"),
        ("DEX", "AA3"),
        ("APP", "AA5"),
        ("INT", "AA7"),
        ("POW", "AG3"),
        ("EDU", "AG5"),
        ("LUK", "AG7"),
    ] {
        attributes.insert(name.to_owned(), number(sheet, cell)?);
    }

    let mut skills = Vec::new();
    for (name_col, specialization_col, regular_col, hard_col, extreme_col) in
        [("F", "H", "R", "T", "V"), ("AB", "AD", "AN", "AP", "AR")]
    {
        for row in 16..=49 {
            let name = text(sheet, &format!("{name_col}{row}"))?;
            if name.is_empty() || name == "自定义技能" {
                continue;
            }
            skills.push(Skill {
                name,
                specialization: text(sheet, &format!("{specialization_col}{row}"))?,
                regular: number(sheet, &format!("{regular_col}{row}"))?,
                hard: number(sheet, &format!("{hard_col}{row}"))?,
                extreme: number(sheet, &format!("{extreme_col}{row}"))?,
            });
        }
    }

    let mut background = BTreeMap::new();
    for (name, cell) in [
        ("形象描述", "AA61"),
        ("思想与信念", "AA63"),
        ("重要之人", "AA65"),
        ("意义非凡之地", "AA67"),
        ("宝贵之物", "AA69"),
        ("特质", "AA71"),
        ("伤口和疤痕", "AA73"),
        ("恐惧症和躁狂症", "AA75"),
        ("背景故事", "W77"),
    ] {
        background.insert(name.to_owned(), text(sheet, cell)?);
    }

    Ok(CharacterCard {
        name: text(sheet, "E3")?,
        player: text(sheet, "E4")?,
        occupation: text(sheet, "E5")?,
        age: number(sheet, "E6")?,
        gender: text(sheet, "M6")?,
        era: text(sheet, "M4")?,
        residence: text(sheet, "E7")?,
        birthplace: text(sheet, "M7")?,
        attributes,
        hit_points: Points {
            current: number(sheet, "E10")?,
            max: number(sheet, "G10")?,
        },
        sanity: number(sheet, "N10")?,
        magic_points: Points {
            current: number(sheet, "W10")?,
            max: number(sheet, "Y10")?,
        },
        movement: number(sheet, "AF10")?,
        skills,
        background,
    })
}

fn value<'a>(sheet: &'a Range<Data>, cell: &str) -> &'a Data {
    let split = cell.find(|c: char| c.is_ascii_digit()).unwrap();
    let column = cell[..split]
        .bytes()
        .fold(0_u32, |n, c| n * 26 + u32::from(c - b'A' + 1));
    let row: u32 = cell[split..].parse().unwrap();
    sheet
        .get_value((row - 1, column - 1))
        .unwrap_or(&Data::Empty)
}

fn text(sheet: &Range<Data>, cell: &str) -> Result<String, String> {
    match value(sheet, cell) {
        Data::Error(_) => Err(format!(
            "人物卡 {cell} 存在公式错误，请在 Excel/WPS 中修正并保存。"
        )),
        data => Ok(data.to_string().trim().to_owned()),
    }
}

fn number(sheet: &Range<Data>, cell: &str) -> Result<Option<u32>, String> {
    let data = value(sheet, cell);
    if matches!(data, Data::Empty) || matches!(data, Data::String(s) if s.trim().is_empty()) {
        return Ok(None);
    }
    let number = match data {
        Data::Int(n) => Some(*n as f64),
        Data::Float(n) => Some(*n),
        Data::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    };
    match number {
        Some(n) if n.is_finite() && n >= 0.0 && n <= u32::MAX as f64 && n.fract() == 0.0 => {
            Ok(Some(n as u32))
        }
        _ => Err(format!(
            "人物卡 {cell} 不是有效的非负整数，请检查数值或公式后重新保存。"
        )),
    }
}
