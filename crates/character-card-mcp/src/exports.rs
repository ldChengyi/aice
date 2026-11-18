//! 人物卡快照暂存、字段别名归一化及修改后重新导出。

use bytes::Bytes;
use character_card::{CardChanges, CharacterCardDocument, TEMPLATES};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use uuid::Uuid;

pub const EXPORT_TTL: Duration = Duration::from_secs(3600);
const MAX_EXPORTS: usize = 64;

struct SavedCard {
    template_id: &'static str,
    changes: CardChanges,
    bytes: Bytes,
    created: Instant,
}

/// 可下载人物卡的临时存储；ID 对应不可变的导出快照，不代表用户身份。
pub struct CardExports {
    cards: Mutex<BTreeMap<String, SavedCard>>,
    pub(crate) workers: std::sync::Arc<Semaphore>,
    public_base_url: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ExportResult {
    pub card_id: String,
    pub download_url: String,
    pub applied_fields: CardChanges,
    pub expires_in_seconds: u64,
    pub note: String,
}

impl CardExports {
    /// 创建内存存储；根地址为空时返回相对链接，非空地址应由宿主提前校验。
    pub fn new(public_base_url: String) -> Self {
        Self {
            cards: Mutex::new(BTreeMap::new()),
            workers: std::sync::Arc::new(Semaphore::new(2)),
            public_base_url,
        }
    }

    /// 清理过期记录后读取文件；不存在时返回 None，Bytes 克隆共享底层缓冲区。
    pub fn download(&self, id: &str) -> Result<Option<Bytes>, String> {
        let mut cards = self.cards.lock().map_err(|_| "人物卡暂存不可用")?;
        cards.retain(|_, card| card.created.elapsed() < EXPORT_TTL);
        Ok(cards.get(id).map(|card| card.bytes.clone()))
    }

    /// 归一化字段别名并合并已有修改，导出为新快照；旧 ID 的文件保持不变。
    pub(crate) fn export(
        &self,
        template_id: Option<String>,
        card_id: Option<String>,
        changes: CardChanges,
    ) -> Result<ExportResult, String> {
        let (template_id, mut combined) = if let Some(id) = card_id {
            let cards = self.cards.lock().map_err(|_| "人物卡暂存不可用")?;
            let card = cards
                .get(&id)
                .filter(|card| card.created.elapsed() < EXPORT_TTL)
                .ok_or("人物卡不存在或已过期，请重新提供完整数据创建")?;
            if template_id
                .as_deref()
                .is_some_and(|id| id != card.template_id)
            {
                return Err("修改已有卡时不能更换模版".into());
            }
            (card.template_id.to_owned(), card.changes.clone())
        } else {
            (
                template_id.unwrap_or_else(|| "coc7".into()),
                CardChanges::new(),
            )
        };
        // 离开上面的锁作用域后再处理 Excel，避免耗时操作阻塞其他下载。
        let template = TEMPLATES
            .iter()
            .find(|template| template.id == template_id)
            .ok_or("未找到该人物卡模版")?;
        let mut card = template.create()?;
        let mut normalized = CardChanges::new();
        for (key, value) in changes {
            let alias = match key.as_str() {
                "HP" | "hp" => "hit_points.current",
                "MP" | "mp" => "magic_points.current",
                "SAN" | "san" => "sanity",
                "STR" => "attributes.STR",
                "CON" => "attributes.CON",
                "SIZ" => "attributes.SIZ",
                "DEX" => "attributes.DEX",
                "APP" => "attributes.APP",
                "INT" => "attributes.INT",
                "POW" => "attributes.POW",
                "EDU" => "attributes.EDU",
                "LUK" => "attributes.LUK",
                _ => &key,
            };
            let field = card.fields().iter()
                .find(|field| field.id == alias || field.label == alias)
                .ok_or_else(|| format!("未知或不可写字段：{key}；请先查询可编辑字段。DB、属性总和与技能总值不能直接写入。"))?;
            // 中文名与英文别名可能指向同一字段；拒绝重复，避免静默覆盖输入。
            if normalized.insert(field.id.clone(), value).is_some() {
                return Err(format!("字段重复：{key}，请只提供一种字段名称"));
            }
        }
        combined.extend(normalized);
        card.update(&combined)?;
        let bytes = Bytes::from(card.export()?);
        let id = Uuid::new_v4().to_string();
        let result = ExportResult {
            download_url: format!("{}/ui/character-cards/exports/{id}", self.public_base_url),
            card_id: id.clone(),
            applied_fields: combined.clone(),
            expires_in_seconds: EXPORT_TTL.as_secs(),
            note: "文件及卡 ID 暂存一小时，服务重启后失效。用 Excel/WPS 打开以重算公式；DB 和属性总和不直接写入。修改时提交 card_id 和需变更的字段，会生成新卡 ID，保留旧文件。".into(),
        };
        let mut cards = self.cards.lock().map_err(|_| "人物卡暂存不可用")?;
        cards.retain(|_, card| card.created.elapsed() < EXPORT_TTL);
        if cards.len() >= MAX_EXPORTS {
            return Err("人物卡暂存已满，请等待旧文件过期后重试".into());
        }
        cards.insert(
            id,
            SavedCard {
                template_id: template.id,
                changes: combined,
                bytes,
                created: Instant::now(),
            },
        );
        Ok(result)
    }
}
