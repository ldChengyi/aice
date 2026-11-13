# Aice

面向骰娘接入的 Axum HTTP 服务，提供健康检查和 DeepSeek 单轮聊天。

## 本地启动

```sh
cargo run
```

服务默认监听 `127.0.0.1:3210`。需要自定义配置时，复制 `config.example.yaml` 为 `config.yaml` 后修改。也可以用 `AICE_CONFIG_PATH` 指定配置文件，或用 `AICE_BIND_ADDR` 覆盖其中的监听地址。

`llm` 是可选配置。要启用聊天，在 `config.yaml` 中启用示例的 `llm` 配置，并在启动服务前设置环境变量 `DEEPSEEK_API_KEY`。省略 `llm.provider` 时默认使用 `deepseek`。如果未启用 `llm`，聊天接口返回 HTTP 503。

也可以单独在终端试跑 DeepSeek。

```sh
cargo run --locked --example deepseek -- '给 1920 年代的记者林秋写一段 100 字的调查员背景。'
```

```sh
curl -i http://127.0.0.1:3210/healthz
```

健康检查返回 HTTP 200 和 `{"status":"ok"}`。前台运行时可按 Ctrl+C 退出。

## 在 Linux 服务器用 Docker 启动

在 Aice 项目根目录启动服务。Compose 默认接入现有的 `sealdice_sealdice_network`。可以用 `docker network ls` 确认名称，名称不同时通过 `SEALDICE_NETWORK` 指定。

Compose 默认只读挂载项目根目录的 `config.yaml`。首次部署时先复制示例配置，在其中启用 `llm`，并在项目根目录的 `.env` 中填写 `DEEPSEEK_API_KEY=你的密钥`。如果配置存放在其他位置，可以在 `.env` 中设置 `AICE_CONFIG_FILE`。Dockerfile 为容器设置的 `AICE_BIND_ADDR=0.0.0.0:3210` 优先于 YAML 中的监听地址。

```sh
test -f config.yaml || cp config.example.yaml config.yaml
docker compose up -d --build
curl -fsS http://127.0.0.1:3210/healthz
```

`127.0.0.1:3210` 的端口映射供服务器本机验证使用。

构建默认从 Docker Hub 拉取 Rust 和 Debian 基础镜像，并通过 RsProxy 下载 Cargo 依赖。可以用 `AICE_IMAGE_PREFIX` 指定其他镜像源。需要从 crates.io 下载依赖时运行：

```sh
AICE_CARGO_MIRROR= docker compose up -d --build
```

## 海豹聊天

`POST /chat` 接收 UTF-8 文本，并返回模型生成的回复。每次请求是独立的一轮对话。

```sh
curl -fsS -H 'Content-Type: text/plain; charset=utf-8' \
  --data-binary '用一句话介绍一下 COC 跑团' http://127.0.0.1:3210/chat
```

1. 在海豹 WebUI 的「扩展功能 → JS 扩展」上传或替换 `plugins/sealdice/aice-callback.js`，然后重载 JS。
2. 插件的 `chat_url` 默认为 `http://aice:3210/chat`，适用于海豹和 Aice 在同一 Docker 网络的部署。旧版用过自定义地址时，把地址填入新的 `chat_url`，并将路径改成 `/chat`。两个服务都直接在同一宿主机运行时，可以填 `http://127.0.0.1:3210/chat`。
3. 在会话里发 `.aicechat 用一句话介绍一下 COC 跑团`。若会话里还没启用扩展，先发 `.ext aice-callback on`。使用 QQ 官方机器人的斜杠指令时，在 QQ 开放平台登记 `aicechat`。

## 目录

```text
src/
├── bin/aice.rs       # 可执行入口，组装并启动服务
├── lib.rs            # 共享模块入口
├── config/           # YAML 和环境变量配置读取
├── router/           # HTTP 路由和请求处理
├── service/          # 业务流程
└── infra/            # 模型、数据库和外部服务的实现
```

新增 HTTP 能力时，在 `router/` 注册入口，在 `service/` 实现业务流程。需要调用 Rig、数据库或外部 API 时，在 `infra/` 添加对应实现。
