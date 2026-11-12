# Aice

面向骰娘接入的 Axum HTTP 服务。当前提供健康检查和回调测试。

## 本地启动

```sh
cargo run
```

服务默认监听 `127.0.0.1:3210`，可通过 `AICE_BIND_ADDR` 覆盖监听地址。

```sh
curl -i http://127.0.0.1:3210/healthz
```

健康检查返回 HTTP 200 和 `{"status":"ok"}`。前台运行时可按 Ctrl+C 退出。

## 在 Linux 服务器用 Docker 启动

在 Aice 项目根目录启动服务。Compose 默认接入现有的 `sealdice_sealdice_network`。可以用 `docker network ls` 确认名称，名称不同时通过 `SEALDICE_NETWORK` 指定。

```sh
docker compose up -d --build
curl -fsS http://127.0.0.1:3210/healthz
```

在海豹插件里把 `callback_url` 填为 `http://aice:3210/callback`。`127.0.0.1:3210` 的端口映射供服务器本机验证使用。

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
├── config.rs         # 环境变量读取和校验
├── router/           # HTTP 路由和请求处理
├── service/          # 业务流程
└── infra/            # 模型、数据库和外部服务的实现
```

新增 HTTP 能力时，在 `router/` 注册入口，在 `service/` 实现业务流程。需要调用 Rig、数据库或外部 API 时，在 `infra/` 添加对应实现。
