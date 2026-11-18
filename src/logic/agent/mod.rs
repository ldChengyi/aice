use crate::service::agent::{AgentModel, ModelError};
use character_card_mcp::{CardExports, CharacterCardMcp, EXPORT_TOOL_NAME, ExportResult};
use rig_core::{
    completion::{CompletionRequest, ToolDefinition},
    message::{Message, ToolName},
};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use std::{sync::Arc, time::Duration};

const INSTRUCTIONS: &str = include_str!("instructions.md");
const MAX_ROUNDS: usize = 6;
const MAX_TOOL_CALLS: usize = 12;
const TIMEOUT: Duration = Duration::from_secs(120);

pub struct ChatAgent {
    model: Box<dyn AgentModel>,
    exports: Arc<CardExports>,
}

impl ChatAgent {
    pub fn new(model: Box<dyn AgentModel>, exports: Arc<CardExports>) -> Self {
        Self { model, exports }
    }

    pub async fn reply(&self, prompt: &str) -> Result<String, ModelError> {
        // Keep completed artifacts even when a later call fails or times out.
        let mut artifacts = Vec::new();
        let outcome = tokio::time::timeout(TIMEOUT, self.run(prompt, &mut artifacts))
            .await
            .unwrap_or_else(|_| Err("模型与人物卡工具处理超过 120 秒，请稍后重试".into()));
        if artifacts.is_empty() {
            return outcome;
        }

        let mut reply = artifacts
            .iter()
            .map(|card| {
                format!(
                    "已生成人物卡，记录了 {} 项字段。\n下载 Excel：{}\n人物卡 ID：{}\n{}",
                    card.applied_fields.len(),
                    card.download_url,
                    card.card_id,
                    card.note,
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        if let Err(error) = outcome {
            eprintln!("agent stopped after partial export: {error}");
            reply.push_str(
                "\n\n部分操作未完成，以上文件已成功生成。请检查结果后，仅重试未完成的部分。",
            );
        }
        Ok(reply)
    }

    async fn run(
        &self,
        prompt: &str,
        artifacts: &mut Vec<ExportResult>,
    ) -> Result<String, ModelError> {
        // Dropping these running services cancels both ends, including on timeout.
        let (server_io, client_io) = tokio::io::duplex(64 * 1024);
        let server = CharacterCardMcp::new(self.exports.clone());
        let (_server, client) = tokio::try_join!(
            async {
                server
                    .serve(server_io)
                    .await
                    .map_err(|e| -> ModelError { Box::new(e) })
            },
            async { ().serve(client_io).await.map_err(|e| -> ModelError { Box::new(e) }) },
        )?;
        let mut definitions = Vec::new();
        for tool in client.list_all_tools().await? {
            let mut definition = ToolDefinition::new(
                ToolName::new(tool.name.into_owned())?,
                tool.description.unwrap_or_default().into_owned(),
                serde_json::Value::Object((*tool.input_schema).clone()),
            );
            if let Some(annotations) = tool.annotations {
                definition = definition
                    .with_annotations(serde_json::from_value(serde_json::to_value(annotations)?)?);
            }
            definitions.push(definition);
        }
        let mut request = CompletionRequest::new(prompt)
            .preamble(INSTRUCTIONS)
            .tools(definitions);
        let mut tool_count = 0;
        for _ in 0..MAX_ROUNDS {
            let response = self.model.complete(request.clone()).await?;
            let calls = response.tool_calls().collect::<Vec<_>>();
            if calls.is_empty() {
                let text = response.text();
                return if text.trim().is_empty() {
                    Err("模型没有返回文本".into())
                } else {
                    Ok(text)
                };
            }
            tool_count += calls.len();
            if tool_count > MAX_TOOL_CALLS {
                return Err("人物卡工具调用次数超出限制，请简化请求后重试".into());
            }
            let message = response.message().ok_or("模型没有返回工具调用消息")?;
            request.chat_history.push(Arc::new(message));
            let mut failed = false;
            for call in calls {
                let result = if call.function.invalid_arguments.is_some() {
                    serde_json::json!({"isError": true, "message": "工具参数必须是 JSON 对象"})
                } else {
                    match client
                        .call_tool(
                            CallToolRequestParams::new(call.function.name.to_string())
                                .with_arguments(call.function.arguments.clone()),
                        )
                        .await
                    {
                        Ok(result) => {
                            if call.function.name.as_str() == EXPORT_TOOL_NAME
                                && result.is_error != Some(true)
                            {
                                let value = result
                                    .structured_content
                                    .as_ref()
                                    .ok_or("人物卡导出缺少结构化结果")?;
                                artifacts.push(serde_json::from_value(value.clone())?);
                            }
                            serde_json::to_value(result)?
                        }
                        Err(error) => {
                            serde_json::json!({"isError": true, "message": error.to_string()})
                        }
                    }
                };
                failed |= result["isError"] == true;
                request.chat_history.push(Arc::new(Message::tool_result(
                    call.id.clone(),
                    call.function.name.clone(),
                    result.to_string(),
                )));
            }
            // Finish the entire batch before returning links from actual exports.
            if !artifacts.is_empty() {
                return if failed {
                    Err("同轮部分工具调用失败".into())
                } else {
                    Ok(String::new())
                };
            }
        }
        Err("人物卡工具处理轮数超出限制，请简化请求后重试".into())
    }
}
