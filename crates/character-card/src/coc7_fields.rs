//! CoC7 精简版的可编辑字段映射；公式计算单元格不开放写入。

use super::{
    document::{CardField, FieldKind},
    read_xlsx,
};

/// 依据模板技能顺序生成可写字段；仅映射输入单元格，不暴露技能总值等公式结果。
pub(super) fn fields(bytes: &[u8]) -> Result<Vec<CardField>, String> {
    let card = read_xlsx(bytes)?;
    if card.skills.len() != 66 {
        return Err("CoC 7 模版技能表布局不匹配，请检查技能名称或提供对应的字段映射。".into());
    }
    let mut fields = Vec::new();
    let mut add = |id: String, label: String, kind, cell: String| {
        fields.push(CardField {
            id,
            label,
            kind,
            cell,
            worksheet: "xl/worksheets/sheet1.xml".into(),
        })
    };
    for (id, label, cell) in [
        ("name", "姓名", "E3"),
        ("player", "玩家", "E4"),
        ("occupation", "职业", "E5"),
        ("gender", "性别", "M6"),
        ("era", "时代", "M4"),
        ("residence", "住地", "E7"),
        ("birthplace", "故乡", "M7"),
    ] {
        add(id.into(), label.into(), FieldKind::Text, cell.into());
    }
    for (id, label, cell) in [
        ("age", "年龄", "E6"),
        ("occupation_id", "职业序号", "M5"),
        ("hit_points.current", "当前生命值", "E10"),
        ("sanity", "当前理智", "N10"),
        ("magic_points.current", "当前魔法值", "W10"),
        ("attributes.STR", "力量", "U3"),
        ("attributes.CON", "体质", "U5"),
        ("attributes.SIZ", "体型", "U7"),
        ("attributes.DEX", "敏捷", "AA3"),
        ("attributes.APP", "外貌", "AA5"),
        ("attributes.INT", "智力", "AA7"),
        ("attributes.POW", "意志", "AG3"),
        ("attributes.EDU", "教育", "AG5"),
        ("attributes.LUK", "幸运", "AG7"),
    ] {
        add(id.into(), label.into(), FieldKind::Integer, cell.into());
    }
    for (label, cell) in [
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
        add(
            format!("background.{label}"),
            label.into(),
            FieldKind::Text,
            cell.into(),
        );
    }
    // 模板技能按左侧 34 项、右侧 32 项的顺序读取，并据此计算所在行列。
    for (index, skill) in card.skills.iter().enumerate() {
        let left = index < 34;
        let row = if left { index + 16 } else { index - 34 + 16 };
        for (key, label, col) in if left {
            [
                ("growth", "成长", "L"),
                ("occupation", "职业", "N"),
                ("interest", "兴趣", "P"),
            ]
        } else {
            [
                ("growth", "成长", "AH"),
                ("occupation", "职业", "AJ"),
                ("interest", "兴趣", "AL"),
            ]
        } {
            // 此处职业加点由模板公式计算，不可作为普通输入覆盖。
            if left && row == 27 && key == "occupation" {
                continue;
            }
            add(
                format!("skills.{}.{key}", skill.name),
                format!("{}·{label}加点", skill.name),
                FieldKind::Integer,
                format!("{col}{row}"),
            );
        }
        let specialized = if left {
            matches!(row, 20..=22 | 34..=41 | 46..=49)
        } else {
            matches!(row, 27 | 31..=33 | 37 | 47)
        };
        if specialized {
            add(
                format!("skills.{}.specialization", skill.name),
                format!("{}·子技能", skill.name),
                FieldKind::Text,
                format!("{}{row}", if left { "H" } else { "AD" }),
            );
        }
    }
    Ok(fields)
}
