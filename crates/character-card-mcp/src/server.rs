//! MCP 工具定义、参数和结果 schema，以及异步任务调度。

use crate::{CardExports, ExportResult};
use character_card::{CardChanges, CardField, CardTemplate, CharacterCardDocument, TEMPLATES};
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{tool::schema_for_output, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TemplateInput {
    /// 从 character_card_list_templates 获取的模板 ID，目前为 coc7。
    template_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExportInput {
    /// 省略时使用默认 coc7 模板；修改已有卡时必须保留原模板。
    template_id: Option<String>,
    /// 省略时创建新卡；传入上次导出的 ID 时，在该卡的已有数据上应用修改。
    card_id: Option<String>,
    /// 以字段 ID 或中文标签为键；值为文本、非负整数，或用于清空的 null。
    /// 示例：{"力量":55,"敏捷":25,"意志":70,"体质":50,"外貌":50,"教育":55,"体型":60,"智力":75,"幸运":75,"HP":11}.
    /// DB 和 [440/515] 为派生汇总，不是可写字段；技能总值不等于可分配的加点。
    /// 背景也必须提交：描述对应 background.形象描述，思想信念对应 background.思想与信念，特点对应 background.特质。
    /// 重要之人、意义非凡之地、宝贵之物、背景故事可用中文标签或 background. 加完整标签。
    /// 重要之人理由不是独立字段，须合并到 background.重要之人，不添加“理由：”前缀，例如：父辈。\n你试图证明你比他们更出色。
    changes: CardChanges,
}

#[derive(Serialize, JsonSchema)]
struct TemplatesResult {
    templates: &'static [CardTemplate],
}

#[derive(Serialize, JsonSchema)]
struct FieldsResult {
    template_id: &'static str,
    fields: Vec<CardField>,
}

pub struct CharacterCardMcp {
    exports: Arc<CardExports>,
}

#[tool_router]
impl CharacterCardMcp {
    pub fn new(exports: Arc<CardExports>) -> Self {
        Self { exports }
    }

    #[tool(
        description = "列出可创建人物卡的内置模版 ID 和名称。",
        output_schema = schema_for_output::<TemplatesResult>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn character_card_list_templates(&self) -> CallToolResult {
        CallToolResult::structured(json!(TemplatesResult {
            templates: TEMPLATES
        }))
    }

    #[tool(
        description = "查询人物卡可编辑字段的 ID、中文标签和类型，包含属性和各背景栏目。重要之人及理由共用重要之人字段。技能需区分职业、兴趣和成长加点；公式总值不可直接修改。",
        output_schema = schema_for_output::<FieldsResult>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn character_card_get_fields(
        &self,
        Parameters(input): Parameters<TemplateInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let Some(template) = TEMPLATES
            .iter()
            .find(|template| template.id == input.template_id)
        else {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                "未找到该人物卡模版，请调用 character_card_list_templates 获取有效 ID。",
            )]));
        };
        // 许可移入阻塞任务并随任务释放，使并发限制覆盖实际 Excel 处理过程。
        let permit = self
            .exports
            .workers
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ErrorData::internal_error("人物卡服务不可用", None))?;
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            template.create().map(|card| FieldsResult {
                template_id: template.id,
                fields: card.fields().to_vec(),
            })
        })
        .await
        .map_err(task_error)?;
        Ok(match result {
            Ok(value) => CallToolResult::structured(json!(value)),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error)]),
        })
    }

    #[tool(
        description = "根据用户提供的属性、技能加点和背景创建或修改人物卡，并导出真实 Excel 文件。接受中文属性名、STR 等缩写或字段 ID；HP 指当前生命值。修改已有卡需要 card_id，仅提交变化字段；返回新的 card_id 和 download_url。只填用户给出的数据，不编造缺失信息；不写 DB、总点数或公式单元格。",
        output_schema = schema_for_output::<ExportResult>(),
        annotations(read_only_hint = false, destructive_hint = false, idempotent_hint = false, open_world_hint = false)
    )]
    async fn character_card_export(
        &self,
        Parameters(input): Parameters<ExportInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let permit = self
            .exports
            .workers
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ErrorData::internal_error("人物卡服务不可用", None))?;
        let exports = self.exports.clone();
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            exports.export(input.template_id, input.card_id, input.changes)
        })
        .await
        .map_err(task_error)?;
        Ok(match result {
            Ok(value) => CallToolResult::structured(json!(value)),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error)]),
        })
    }
}

/// 底层任务错误只记录到日志，对 MCP 调用方返回统一的内部错误提示。
fn task_error(error: tokio::task::JoinError) -> ErrorData {
    eprintln!("character card MCP task failed: {error}");
    ErrorData::internal_error("人物卡处理失败，请稍后重试", None)
}

#[tool_handler]
impl ServerHandler for CharacterCardMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions("Aice 人物卡工具：查询模版和字段，创建或修改人物卡并导出 Excel。")
    }
}
