// ==UserScript==
// @name         Aice 聊天
// @author       Aice
// @version      0.2.0
// @description  通过 Aice 向聊天模型提问并回复当前会话。
// ==/UserScript==

(() => {
  const ext = seal.ext.new('aice-callback', 'Aice', '0.2.0');
  const command = seal.ext.newCmdItemInfo();
  command.name = 'aicechat';
  command.help = '.aicechat <问题>  向 Aice 提问';

  async function chat(ctx, msg, body) {
    try {
      const url = seal.ext.getStringConfig(ext, 'chat_url');
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        body,
      });
      if (!response.ok) {
        seal.replyToSender(ctx, msg, `Aice 聊天请求失败（HTTP ${response.status}）。`);
        return;
      }
      seal.replyToSender(ctx, msg, await response.text());
    } catch (error) {
      console.error(`Aice 聊天请求失败：${error}`);
      seal.replyToSender(ctx, msg, '无法连接 Aice，请检查后端地址和服务状态。');
    }
  }

  command.solve = (ctx, msg, cmdArgs) => {
    const body = cmdArgs.rawArgs;
    if (!body.trim()) {
      const result = seal.ext.newCmdExecuteResult(true);
      result.showHelp = true;
      return result;
    }
    // 海豹要求指令处理器同步返回，网络请求完成后再发消息。
    void chat(ctx, msg, body);
    return seal.ext.newCmdExecuteResult(true);
  };

  ext.cmdMap[command.name] = command;
  seal.ext.register(ext);
  seal.ext.registerStringConfig(ext, 'chat_url', 'http://aice:3210/chat');
})();
