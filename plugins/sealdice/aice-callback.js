// ==UserScript==
// @name         Aice 回调测试
// @author       Aice
// @version      0.1.0
// @description  将 aiceecho 的参数发给 Aice，再回复后端返回的内容。
// ==/UserScript==

(() => {
  const ext = seal.ext.new('aice-callback', 'Aice', '0.1.0');
  const command = seal.ext.newCmdItemInfo();
  command.name = 'aiceecho';
  command.help = '.aiceecho <文本>  测试 Aice 回调';

  async function echo(ctx, msg, body) {
    try {
      const url = seal.ext.getStringConfig(ext, 'callback_url');
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        body,
      });
      if (!response.ok) {
        seal.replyToSender(ctx, msg, `Aice 回调失败（HTTP ${response.status}）`);
        return;
      }
      seal.replyToSender(ctx, msg, await response.text());
    } catch (error) {
      console.error(`Aice 回调失败：${error}`);
      seal.replyToSender(ctx, msg, 'Aice 回调失败，请检查后端地址和服务状态。');
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
    void echo(ctx, msg, body);
    return seal.ext.newCmdExecuteResult(true);
  };

  ext.cmdMap[command.name] = command;
  seal.ext.register(ext);
  seal.ext.registerStringConfig(ext, 'callback_url', 'http://aice:3210/callback');
})();
