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

首次启动前，在项目根目录的 `.env` 中设置 `AICE_POSTGRES_PASSWORD`。Compose 会启动 PostgreSQL 17，使用 `postgres_data` 卷保存数据。Aice 在 Docker 内部网络连接数据库，宿主机无需映射 5432 端口。

```sh
test -f config.yaml || cp config.example.yaml config.yaml
docker compose up -d --build
curl -fsS http://127.0.0.1:3210/healthz
```

本地直接运行 Aice 时，可通过 `PGHOST` 启用数据库连接，并用 `PGPORT`、`PGDATABASE`、`PGUSER`、`PGPASSWORD` 指定连接参数。

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

## 机器人人设预览

启动 Aice 后，在浏览器打开 `http://127.0.0.1:3210/personas`。页面由 Leptos 组件在服务端渲染，点击列表项可展开示例人设。

要用公网 IP 访问人设页，在 Linux 服务器的项目目录运行以下命令。

```sh
docker compose up -d --build web
```

Nginx 默认监听宿主机的 80 端口。云安全组和服务器防火墙放行对应的 TCP 端口后，访问 `http://服务器IP/` 会跳转到 `/personas`。公网入口只提供人设页和它的样式。如果 80 端口已有服务，在 `.env` 设置 `AICE_PUBLIC_PORT=8080` 后重新运行命令，再访问 `http://服务器IP:8080/`。

## 目录

```text
src/
├── bin/aice.rs       # 读取配置并启动 HTTP 服务
├── lib.rs            # 共享模块入口
├── config/           # YAML 和环境变量配置读取
├── init/             # 初始化 PostgreSQL 连接池和模型，组装 AppState
├── router/           # 注册 HTTP 路由并绑定处理函数
├── handle/           # 接收 HTTP 请求并生成响应
├── frontend/
│   ├── pages/        # Leptos 页面和组件
│   └── styles/       # CSS 样式
├── logic/            # 组合 service，执行具体业务流程
├── service/          # 模型调用接口和数据库服务
├── dal/              # PostgreSQL 连接池和数据访问
└── infra/            # Rig 和 DeepSeek 等外部服务的实现
```

启动时，`main` 通过 `init::AppState::new` 初始化共享实例，再把 `AppState` 交给 Axum。连接池和模型在启动时创建一次，由请求共享。

`POST /chat` 经 `router → handle::chat → logic::chat → service::agent` 调用模型，实际请求由 `infra::llm` 发给 DeepSeek。`GET /personas` 经 `router → handle::personas → frontend::pages::personas` 渲染人设列表，浏览器通过 `/assets/personas.css` 加载样式。

PostgreSQL 的连接由 `init → service::database → dal::postgres` 建立。后续业务需要读写数据时，由 `logic` 调用 `service`，再由 `service` 通过 `dal` 访问数据库。
