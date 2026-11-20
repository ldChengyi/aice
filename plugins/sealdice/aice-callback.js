// ==UserScript==
// @name         呆猫 · Aice 聊天与人物卡
// @author       Aice
// @version      0.4.1
// @description  通过 Aice 创建或修改 CoC7 人物卡，保留下载链接，并由官方 QQ 机器人发送 Excel。
// ==/UserScript==

(() => {
  const ext = seal.ext.new('aice-callback', 'Aice', '0.4.1');

  // 后端文本只作为文本回复，避免模型内容被海豹解释成 CQ 文件或其他操作。
  function escapeCqText(text) {
    return text.replace(/&/g, '&amp;').replace(/\[/g, '&#91;').replace(/\]/g, '&#93;');
  }

  function sendExportFiles(ctx, msg, reply) {
    // 官方 QQ 群聊和 C2C 私聊使用 OpenQQ；OneBot/NapCat、频道和未知平台均不发送文件。
    if (msg.platform !== 'OpenQQ') return;

    const exports = /^下载 Excel：([^\r\n]+)\r?\n人物卡 ID：([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\r?$/gm;
    const sent = new Set();
    let missingPublicUrl = false;
    let match;
    while ((match = exports.exec(reply)) !== null) {
      const downloadUrl = match[1];
      const cardId = match[2];
      const exportPath = `/ui/character-cards/exports/${cardId}`;
      const namedExportPath = `${exportPath}/character-card.xlsx`;
      if (sent.has(cardId)) continue;
      if (downloadUrl === exportPath || downloadUrl === namedExportPath) {
        missingPublicUrl = true;
        continue;
      }
      // 只识别后端固定格式的导出链接，要求路径与卡 ID 对应，不解析普通聊天里的任意链接。
      if (!/^https?:\/\/[^/?#@\s]+\/ui\/character-cards\/exports\/[0-9a-f-]+(?:\/character-card\.xlsx)?$/.test(downloadUrl)
          || !(downloadUrl.endsWith(exportPath) || downloadUrl.endsWith(namedExportPath))) continue;

      sent.add(cardId);
      try {
        const fileUrl = escapeCqText(downloadUrl).replace(/,/g, '&#44;');
        // 上传、鉴权和被动回复的消息 ID 交给海豹；JS 接口无送达回执，下载链接始终保留。
        seal.replyToSender(ctx, msg, `[CQ:file,url=${fileUrl}]`);
      } catch (error) {
        console.error(`Aice Excel 文件发送失败（${cardId}）：${error}`);
        seal.replyToSender(ctx, msg, 'Excel 文件发送失败，请使用上面的下载链接；管理员可查看海豹日志。');
      }
    }
    if (missingPublicUrl) {
      seal.replyToSender(ctx, msg, 'Excel 自动发送需要完整的公网下载地址，请管理员配置 Aice 的 public_base_url 后重新生成。');
    }
  }

  // 向配置的后端发送原始文本；async/await 等待网络响应后再回复当前会话。
  async function chat(ctx, msg, body) {
    try {
      const url = seal.ext.getStringConfig(ext, 'chat_url').trim();
      if (!url) {
        seal.replyToSender(ctx, msg, '请先在扩展配置中填写 chat_url。同一 Docker 网络下使用 http://aice:3210/chat。');
        return;
      }
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        body,
      });
      if (!response.ok) {
        const hints = {
          404: '请检查插件 chat_url 是否以 /chat 结尾。',
          502: '模型或人物卡工具执行失败，请管理员查看 Aice 后端日志。',
          503: '后端尚未启用聊天模型，请管理员检查 config.yaml 中的 llm 配置。',
        };
        seal.replyToSender(ctx, msg, `Aice 请求失败（HTTP ${response.status}）。${hints[response.status] || '请管理员检查后端服务。'}`);
        return;
      }
      const reply = await response.text();
      seal.replyToSender(ctx, msg, reply.trim() ? escapeCqText(reply) : 'Aice 未返回内容，请管理员查看后端日志。');
      sendExportFiles(ctx, msg, reply);
    } catch (error) {
      console.error(`Aice 聊天请求失败：${error}`);
      seal.replyToSender(ctx, msg, '无法连接 Aice，请检查后端地址和服务状态。');
    }
  }

  // 两个指令共用请求流程；保留原始换行，属性解析与 MCP 执行交给后端。
  function registerCommand(name, help, prefix) {
    const command = seal.ext.newCmdItemInfo();
    command.name = name;
    command.help = help;
    command.solve = (ctx, msg, cmdArgs) => {
      const body = cmdArgs.rawArgs;
      if (!body.trim() || body.trim().toLowerCase() === 'help' || body.trim() === '帮助') {
        const result = seal.ext.newCmdExecuteResult(true);
        result.showHelp = true;
        return result;
      }
      // 海豹要求指令处理器同步返回，网络请求完成后再发消息；不自动重试导出。
      void chat(ctx, msg, prefix + body);
      return seal.ext.newCmdExecuteResult(true);
    };
    ext.cmdMap[command.name] = command;
  }

  registerCommand('aicechat', '.aicechat <问题>  与呆猫聊天，也可直接要求创建或修改人物卡。', '');
  registerCommand('aicecard', [
    '.aicecard <人物数据或修改要求>  创建或修改 CoC7 人物卡并导出 Excel。',
    '创建示例：.aicecard 力量55 敏捷25 意志70 体质50 外貌50 教育55 体型60 智力75 幸运75 HP11',
    '修改示例：.aicecard 把人物卡 <卡ID> 的力量改成60，其他字段保持不变。',
    '支持多行属性、技能加点和背景；技能请说明职业、兴趣或成长加点。',
    '背景可填写：描述、思想信念、重要之人、重要之人理由、意义非凡之地、宝贵之物、特点。重要之人和理由合并保存。',
    '粘贴的『序号…』职业资料段暂不录入，也不会据此自动加点。',
    '修改必须附带上次返回的卡ID；下载链接与ID一小时内有效，服务重启后失效。',
    '官方 QQ 群聊和私聊会额外发送 Excel 文件；NapCat/OneBot 等三方接入只回复下载链接。',
  ].join('\n'), '请根据以下用户要求创建或修改 CoC7 人物卡并导出 Excel。只填写明确给出的数据，修改已有卡必须使用用户提供的卡ID。\n\n');

  seal.ext.register(ext);
  seal.ext.registerStringConfig(ext, 'chat_url', 'http://aice:3210/chat', '海豹访问 Aice 的内网聊天接口地址；用户下载地址在后端 public_base_url 配置。');
})();
