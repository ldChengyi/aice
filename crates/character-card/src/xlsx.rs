//! 直接改写 XLSX 内的输入单元格，保留模板样式、附件和公式。

use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum CellValue {
    Text(String),
    Integer(u32),
    Empty,
}

pub struct CellEdit<'a> {
    pub worksheet: &'a str,
    pub cell: &'a str,
    pub value: &'a CellValue,
}

/// 改写相关 XML 并原样复制其他 ZIP 条目；目标单元格缺失或试图覆盖公式时返回错误。
pub fn export(template: &[u8], edits: &[CellEdit<'_>]) -> Result<Vec<u8>> {
    if edits.is_empty() {
        return Ok(template.to_vec());
    }
    let mut source = ZipArchive::new(Cursor::new(template))?;
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    let mut pending: BTreeSet<_> = edits
        .iter()
        .map(|e| (e.worksheet.to_owned(), e.cell.to_owned()))
        .collect();
    for i in 0..source.len() {
        let mut file = source.by_index(i)?;
        let name = file.name()?.into_owned();
        let worksheet = name.starts_with("xl/worksheets/") && name.ends_with(".xml");
        if worksheet || name == "xl/workbook.xml" {
            let mut xml = String::new();
            file.read_to_string(&mut xml)?;
            let result = if worksheet {
                let edits: BTreeMap<_, _> = edits
                    .iter()
                    .filter(|e| e.worksheet == name)
                    .map(|e| (e.cell, e.value))
                    .collect();
                rewrite_sheet(&xml, &edits, |cell| {
                    pending.remove(&(name.clone(), cell.to_owned()));
                })?
            } else {
                recalculate_on_open(&xml)?
            };
            output.start_file(
                name,
                SimpleFileOptions::default().compression_method(file.compression()),
            )?;
            output.write_all(&result)?;
        } else {
            output.raw_copy_file(file)?;
        }
    }
    if let Some((sheet, cell)) = pending.first() {
        return Err(format!("模版缺少可编辑单元格：{sheet}!{cell}").into());
    }
    Ok(output.finish()?.into_inner())
}

fn without_attributes<'a>(start: &BytesStart<'a>, keys: &[&str]) -> Result<BytesStart<'static>> {
    let mut result = start.to_owned();
    result.clear_attributes();
    for attribute in start.attributes() {
        let attribute = attribute?;
        if !keys.contains(&attribute.key.as_ref()) {
            result.push_attribute(attribute);
        }
    }
    Ok(result)
}

/// 保留单元格地址和样式，按值类型重写内容；文本用内联字符串避免修改共享字符串表。
fn write_input(
    writer: &mut Writer<Vec<u8>>,
    start: &BytesStart<'_>,
    value: &CellValue,
) -> Result<()> {
    let tag = |local| qualified_name(start.name().as_ref(), local);
    let mut cell = without_attributes(start, &["t"])?;
    if matches!(value, CellValue::Text(_)) {
        cell.push_attribute(("t", "inlineStr"));
    }
    writer.write_event(Event::Start(cell))?;
    match value {
        CellValue::Text(text) => {
            writer.write_event(Event::Start(BytesStart::new(tag("is"))))?;
            let mut t = BytesStart::new(tag("t"));
            t.push_attribute(("xml:space", "preserve"));
            writer.write_event(Event::Start(t))?;
            writer.write_event(Event::Text(BytesText::new(&escape_excel_text(text))))?;
            writer.write_event(Event::End(BytesEnd::new(tag("t"))))?;
            writer.write_event(Event::End(BytesEnd::new(tag("is"))))?;
        }
        CellValue::Integer(number) => {
            writer.write_event(Event::Start(BytesStart::new(tag("v"))))?;
            writer.write_event(Event::Text(BytesText::new(&number.to_string())))?;
            writer.write_event(Event::End(BytesEnd::new(tag("v"))))?;
        }
        CellValue::Empty => {}
    }
    writer.write_event(Event::End(start.to_end()))?;
    Ok(())
}

/// 逐单元格处理 XML：替换输入值、拒绝覆盖公式，并清除所有公式的旧计算缓存。
fn rewrite_sheet(
    xml: &str,
    edits: &BTreeMap<&str, &CellValue>,
    mut applied: impl FnMut(&str),
) -> Result<Vec<u8>> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::with_capacity(xml.len()));
    loop {
        let event = reader.read_event()?;
        match event {
            Event::Start(ref start) | Event::Empty(ref start)
                if start.local_name().as_ref() == "c" =>
            {
                let empty = matches!(event, Event::Empty(_));
                let cell_end = BytesEnd::new(start.name().as_ref().to_owned());
                let address = start
                    .try_get_attribute("r")?
                    .ok_or("单元格缺少地址")?
                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)?
                    .into_owned();
                let mut contents = Vec::new();
                if !empty {
                    loop {
                        let child = reader.read_event()?;
                        if matches!(&child, Event::End(end) if end.local_name().as_ref() == "c") {
                            break;
                        }
                        if matches!(child, Event::Eof) {
                            return Err("单元格 XML 不完整".into());
                        }
                        contents.push(child);
                    }
                }
                let formula = contents.iter().any(|event| matches!(event, Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "f"));
                if let Some(value) = edits.get(address.as_str()) {
                    if formula {
                        return Err(format!("不能覆盖公式单元格：{address}").into());
                    }
                    write_input(&mut writer, start, value)?;
                    applied(&address);
                } else if formula {
                    // 清除旧输入对应的公式缓存，等待 Excel/WPS 打开文件后重新计算。
                    writer.write_event(Event::Start(without_attributes(start, &["t"])?))?;
                    let mut skip = false;
                    for child in contents {
                        match &child {
                            Event::Start(e) if e.local_name().as_ref() == "v" => {
                                skip = true;
                                continue;
                            }
                            Event::End(e) if e.local_name().as_ref() == "v" => {
                                skip = false;
                                continue;
                            }
                            Event::Empty(e) if e.local_name().as_ref() == "v" => continue,
                            _ => {}
                        }
                        if !skip {
                            writer.write_event(child)?;
                        }
                    }
                    writer.write_event(Event::End(cell_end))?;
                } else {
                    writer.write_event(event)?;
                    if !empty {
                        for child in contents {
                            writer.write_event(child)?;
                        }
                        writer.write_event(Event::End(cell_end))?;
                    }
                }
            }
            Event::Eof => break,
            _ => writer.write_event(event)?,
        }
    }
    Ok(writer.into_inner())
}

/// 更新或补充工作簿计算设置，让 Excel/WPS 打开时重算；后端自身不执行公式。
fn recalculate_on_open(xml: &str) -> Result<Vec<u8>> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let mut found = false;
    loop {
        let event = reader.read_event()?;
        match event {
            Event::Start(ref start) | Event::Empty(ref start)
                if start.local_name().as_ref() == "calcPr" =>
            {
                let mut calc =
                    without_attributes(start, &["calcMode", "fullCalcOnLoad", "forceFullCalc"])?;
                calc.push_attribute(("calcMode", "auto"));
                calc.push_attribute(("fullCalcOnLoad", "1"));
                calc.push_attribute(("forceFullCalc", "1"));
                writer.write_event(if matches!(event, Event::Empty(_)) {
                    Event::Empty(calc)
                } else {
                    Event::Start(calc)
                })?;
                found = true;
            }
            Event::End(ref end) if end.local_name().as_ref() == "workbook" => {
                if !found {
                    let mut calc = BytesStart::new(qualified_name(end.name().as_ref(), "calcPr"));
                    calc.push_attribute(("calcMode", "auto"));
                    calc.push_attribute(("fullCalcOnLoad", "1"));
                    calc.push_attribute(("forceFullCalc", "1"));
                    writer.write_event(Event::Empty(calc))?;
                }
                writer.write_event(event)?;
            }
            Event::Eof => break,
            _ => writer.write_event(event)?,
        }
    }
    Ok(writer.into_inner())
}

/// 为新标签沿用原标签的命名空间前缀，兼容带前缀的工作簿 XML。
fn qualified_name(parent: &str, local: &str) -> String {
    match parent.split_once(':') {
        Some((prefix, _)) => format!("{prefix}:{local}"),
        None => local.to_owned(),
    }
}

/// 转义字面量 _xHHHH_ 和回车，避免 Excel 再次解码时改变用户原始文本。
fn escape_excel_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for (index, ch) in text.char_indices() {
        // Excel 会在 XML 解码后解析 _xHHHH_，因此先转义文本中原样出现的同形序列。
        if ch == '_'
            && text.as_bytes().get(index..index + 7).is_some_and(|part| {
                part[1] == b'x' && part[6] == b'_' && part[2..6].iter().all(u8::is_ascii_hexdigit)
            })
        {
            escaped.push_str("_x005F_");
        } else if ch == '\r' {
            escaped.push_str("_x000D_");
        } else {
            escaped.push(ch);
        }
    }
    escaped
}
