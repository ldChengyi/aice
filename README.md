# Aice

面向骰娘接入的 Axum HTTP 服务。当前提供健康检查和回调测试。

## 本地启动

```sh
cargo run
```

服务默认监听 `127.0.0.1:3210`。需要自定义配置时，复制 `config.example.yaml` 为 `config.yaml` 后修改。也可以用 `AICE_CONFIG_PATH` 指定配置文件，或用 `AICE_BIND_ADDR` 覆盖其中的监听地址。

`llm` 是可选配置，包含 `base_url` 和 `model`。目前服务只读取这些值，模型调用将在接入 LLM 时实现。

```sh
curl -i http://127.0.0.1:3210/healthz
```

健康检查返回 HTTP 200 和 `{"status":"ok"}`。前台运行时可按 Ctrl+C 退出。

## 在 Linux 服务器用 Docker 启动

在 Aice 项目根目录启动服务。Compose 默认接入现有的 `sealdice_sealdice_network`。可以用 `docker network ls` 确认名称，名称不同时通过 `SEALDICE_NETWORK` 指定。

Compose 默认只读挂载项目根目录的 `config.yaml`。首次部署时先复制示例配置，再按需修改。如果配置存放在其他位置，可以在 `.env` 中设置 `AICE_CONFIG_FILE`。Dockerfile 为容器设置的 `AICE_BIND_ADDR=0.0.0.0:3210` 优先于 YAML 中的监听地址。

```sh
test -f config.yaml || cp config.example.yaml config.yaml
docker compose up -d --build
curl -fsS http://127.0.0.1:3210/healthz
```

在海豹插件里把 `callback_url` 填为 `http://aice:3210/callback`。`127.0.0.1:3210` 的端口映射供服务器本机验证使用。

构建默认从 Docker Hub 拉取 Rust 和 Debian 基础镜像，并通过 RsProxy 下载 Cargo 依赖。可以用 `AICE_IMAGE_PREFIX` 指定其他镜像源。需要从 crates.io 下载依赖时运行：

```sh
AICE_CARGO_MIRROR= docker compose up -d --build
```

## 海豹回调测试

`POST /callback` 原样返回请求正文。

```sh
curl --data-binary '你好，Aice' http://127.0.0.1:3210/callback
```

1. 在海豹 WebUI 的「扩展功能 → JS 扩展」上传 `plugins/sealdice/aice-callback.js`，然后重载 JS。
2. 插件默认使用 `http://aice:3210/callback`，适用于海豹和 Aice 在同一 Docker 网络的部署。如果已安装过插件，请检查设置里的 `callback_url` 是否仍是旧地址。两个服务都直接在同一宿主机运行时，可以填 `http://127.0.0.1:3210/callback`。
3. 在测试会话发 `.aiceecho 你好，Aice`，骰娘应回复 `你好，Aice`。若会话里还没启用扩展，先发 `.ext aice-callback on`。QQ 官方机器人可使用 `/aiceecho`，如果平台要求登记指令，就在 QQ 开放平台添加 `aiceecho`。

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
