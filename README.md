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

## 内置人物卡模版

默认数据来自 `character-card-templates/coc7-character-card-template.xlsx`，已恢复为空白建卡模板：姓名、玩家、职业、年龄、性别、属性、当前生命/理智/魔法值、技能加点和背景等角色数据已清空，技能保留基础值，时代等模板设置保留默认值。文件在编译时嵌入程序，并已纳入 Docker 构建。替换这份文件后需要重新编译或构建镜像。

人物卡通过选择内置模版读取，无需上传文件。将 `character-card-templates` 目录随项目提交到 GitHub 即可。

`GET /character-cards/templates` 返回可选模版列表，目前包含 `{"id":"coc7","name":"CoC 7 Character Card"}`。选择其中的 `id` 后调用读取接口：

```sh
curl -fsS http://127.0.0.1:3210/character-cards/templates
curl -fsS http://127.0.0.1:3210/character-cards/templates/coc7
```

`GET /character-cards/default` 保留为默认选项的快捷读取接口，返回同样的人物卡 JSON：

```sh
curl -fsS http://127.0.0.1:3210/character-cards/default
```

目前支持 `character-card-templates` 目录中这份「COC 七版半自动人物卡 - 精简版」的版式，读取 `人物卡` 工作表中的基本资料、九项属性、当前/最大生命值与魔法值、理智、移动力、两列技能（含子技能和普通/困难/极难值）以及背景故事。返回字段包括 `name`、`attributes`、`hit_points`、`sanity`、`magic_points`、`skills` 和 `background`；数值空格返回 `null`，已填的 0 保留为 0。

公式使用文件中已保存的计算结果，不会重新计算；修改模版后，请先用 Excel/WPS 计算并保存，再重新编译。接口只读取并返回结果，不写入数据库；不存在的模版 ID 返回 HTTP 404，内置模版读取失败返回 HTTP 500。当前不支持旧版 `.xls` 或其他布局的 Excel，原文件上传接口已移除。

添加同版式模版时，将文件放入 `character-card-templates`，并在 `src/service/character_card.rs` 的 `TEMPLATES` 中注册唯一 ID、名称、文件及 `field_mapping`；列表第一项为默认选项。

其他 Rust 调用方也可以直接使用 `aice::service::character_card::read_xlsx(&bytes)`；读取内置默认卡使用 `aice::service::character_card::read_default()`。

### 创建、修改和导出人物卡

`CharacterCardDocument` trait 统一提供 `create`、`fields`、`update`、`export`。`ExcelCharacterCard` 是共用的 Excel 实现；每次创建独立人物卡，更新只作用于该实例，导出为 `.xlsx` 字节，不修改内置模板。

```rust
use aice::service::character_card::{CardChanges, CellValue, CharacterCardDocument, TEMPLATES};

let mut card = TEMPLATES[0].create()?;
card.update(&CardChanges::from([
    ("name".into(), CellValue::Text("林秋".into())),
    ("attributes.DEX".into(), CellValue::Integer(80)),
    ("skills.侦查.occupation".into(), CellValue::Integer(45)),
    ("background.背景故事".into(), CellValue::Text("一位寻找失踪朋友的记者。".into())),
]))?;
let xlsx = card.export()?;
```

HTTP 调用先通过 `GET /character-cards/templates/{id}/fields` 获取可编辑字段（`id`、`label`、`kind`），再将字段 ID 与值组成 JSON，提交到 `POST /character-cards/templates/{id}/export`，响应是可下载的 Excel 文件。

```sh
curl -fsS http://127.0.0.1:3210/character-cards/templates/coc7/fields
curl -fsS -H 'Content-Type: application/json' \
  -d '{"name":"林秋","attributes.DEX":80,"skills.侦查.occupation":45,"background.背景故事":"一位寻找失踪朋友的记者。"}' \
  http://127.0.0.1:3210/character-cards/templates/coc7/export \
  -o investigator.xlsx
```

空对象 `{}` 创建并下载空白人物卡。未传的字段保持当前值；`null` 清空字段；数值字段只接受非负整数。`update` 可以多次调用，未知字段、错误类型或无效文本会使整批修改失败。技能通过 `growth`（成长）、`occupation`（职业）、`interest`（兴趣）加点，子技能使用 `specialization`；总值、困难和极难值由模板公式计算，不能直接覆盖。HTTP 导出每次从模板新建，不保存人物卡；多次修改同一张卡可在 Rust 中复用实例，或在请求中提交完整修改集合。

导出保留模板的样式、合并单元格、数据验证、批注、隐藏工作表及公式。修改输入后会清除所有旧的公式缓存，并设置打开时自动重新计算；请用 Excel/WPS 打开导出文件来更新计算结果。后端不执行 Excel 公式，直接读取尚未打开保存的导出文件时，公式结果为空。

不同 Excel 布局可注册各自的字段映射（字段 ID、类型、工作表 XML 路径与已有的输入单元格），复用同一套创建、校验、修改、导出逻辑。非 Excel 实现可单独实现 `CharacterCardDocument`。规则特有的业务校验和现有 CoC 7 结构化读取适配仍需由相应卡种实现。

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

启动 Aice 后，在浏览器打开 `http://127.0.0.1:3210/ui/personas`。页面由 Leptos 组件在服务端渲染，点击列表项可展开示例人设。

要用公网 IP 访问人设页，在 Linux 服务器的项目目录运行以下命令。

```sh
docker compose up -d --build web
```

Nginx 默认监听宿主机的 80 端口。云安全组和服务器防火墙放行对应的 TCP 端口后，访问 `http://服务器IP/` 会跳转到 `/ui/personas`。Nginx 转发整个 `/ui/` 路径。新增页面或样式时，把路由注册到 Axum 的 `frontend` 路由组即可。

如果 80 端口已有服务，在 `.env` 设置 `AICE_PUBLIC_PORT=8080` 后重新运行命令，并通过 `http://服务器IP:8080/` 访问。

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

`POST /chat` 经 `router → handle::chat → logic::chat → service::agent` 调用模型，实际请求由 `infra::llm` 发给 DeepSeek。`GET /ui/personas` 经 `router → handle::personas → frontend::pages::personas` 渲染人设列表，浏览器通过 `/ui/assets/personas.css` 加载样式。

PostgreSQL 的连接由 `init → service::database → dal::postgres` 建立。后续业务需要读写数据时，由 `logic` 调用 `service`，再由 `service` 通过 `dal` 访问数据库。
