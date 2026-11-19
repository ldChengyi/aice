// ==UserScript==
// @name         呆猫 · Aice 聊天与人物卡
// @author       Aice
// @version      0.3.0
// @description  通过 Aice 聊天、创建或修改 CoC7 人物卡，返回 Excel 下载链接。
// ==/UserScript==

(() => {
  const ext = seal.ext.new('aice-callback', 'Aice', '0.3.0');

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
      seal.replyToSender(ctx, msg, reply.trim() ? reply : 'Aice 未返回内容，请管理员查看后端日志。');
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
    '修改必须附带上次返回的卡ID；下载链接与ID一小时内有效，服务重启后失效。',
  ].join('\n'), '请根据以下用户要求创建或修改 CoC7 人物卡并导出 Excel。只填写明确给出的数据，修改已有卡必须使用用户提供的卡ID。\n\n');

  seal.ext.register(ext);
  seal.ext.registerStringConfig(ext, 'chat_url', 'http://aice:3210/chat', '海豹访问 Aice 的内网聊天接口地址；用户下载地址在后端 public_base_url 配置。');
})();
