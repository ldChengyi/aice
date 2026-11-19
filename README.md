# Aice

面向骰娘接入的 Axum HTTP 服务，提供健康检查、DeepSeek 聊天，以及通过内部 MCP 创建、修改和导出 Excel 人物卡。

## 本地启动

```sh
cargo run
```

服务默认监听 `127.0.0.1:3210`。需要自定义配置时，复制 `config.example.yaml` 为 `config.yaml` 后修改。也可以用 `AICE_CONFIG_PATH` 指定配置文件，或用 `AICE_BIND_ADDR` 覆盖其中的监听地址。

`llm` 是可选配置。要启用聊天，在 `config.yaml` 中启用示例的 `llm` 配置，并在启动服务前设置环境变量 `DEEPSEEK_API_KEY`。省略 `llm.provider` 时默认使用 `deepseek`。如果未启用 `llm`，聊天接口返回 HTTP 503。

环境变量模板为 `.env.template`。首次使用时复制并填写，已有 `.env` 时保留原文件：

```sh
test -f .env || cp .env.template .env
```

Docker Compose 会自动读取 `.env`；`cargo run` 和二进制本身不会自动加载。直接运行时，先在同一终端导入，再启动服务：

```sh
set -a
. ./.env
set +a
./target/release/aice
```

`.env` 已被 Git 忽略，模板可以提交。模型名称、API 地址和 `public_base_url` 仍在 `config.yaml` 中配置。

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
test -f .env || cp .env.template .env
# 启动前填写 .env 中的 AICE_POSTGRES_PASSWORD；启用聊天时同时填写 DEEPSEEK_API_KEY。
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

添加同版式模版时，将文件放入 `character-card-templates`，在资源包的 `src/lib.rs` 中嵌入文件，并在 `crates/character-card/src/lib.rs` 的 `TEMPLATES` 中注册唯一 ID、名称、文件及 `field_mapping`；列表第一项为默认选项。

其他 Rust 调用方也可以直接使用 `character_card::read_xlsx(&bytes)`；读取内置默认卡使用 `character_card::read_default()`。

### 创建、修改和导出人物卡

`CharacterCardDocument` trait 统一提供 `create`、`fields`、`update`、`export`。`ExcelCharacterCard` 是共用的 Excel 实现；每次创建独立人物卡，更新只作用于该实例，导出为 `.xlsx` 字节，不修改内置模板。

```rust
use character_card::{CardChanges, CellValue, CharacterCardDocument, TEMPLATES};

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

`POST /chat` 接收 UTF-8 文本，并返回回复。普通问题由模型回答；建卡请求由模型调用内置 MCP 工具，实际生成 Excel 后返回下载链接和人物卡 ID。每次请求独立，不保存聊天历史。

### 在聊天中创建或修改人物卡

人物卡 MCP 使用官方 Rust SDK `rmcp`，在 Aice 进程内通过双向流连接服务端和客户端，无需另外启动 MCP 进程或开放端口。提供三个工具：

- `character_card_list_templates`：列出内置模版。
- `character_card_get_fields`：查询模版可写字段及类型。
- `character_card_export`：从模版创建人物卡，或根据已有 `card_id` 修改字段，导出 Excel 并返回新 ID 和下载地址。

在 `config.yaml` 中启用 `llm` 并设置环境变量 `DEEPSEEK_API_KEY` 后，可直接发送：

```text
.aicechat 按下面的数据生成 CoC7 人物卡并导出：
力量:55 敏捷:25 意志:70
体质:50 外貌:50 教育:55
体型:60 智力:75 幸运:75
HP:11 <DB:0> [440/515]
```

模型应将九项属性和 HP 传给工具。工具接受中文字段名、`STR` 等属性缩写和 `attributes.STR` 等字段 ID；`HP` 写入当前生命值。`DB:0` 和 `[440/515]` 属于派生摘要，不直接覆盖公式。技能普通/困难/极难值也不能当作职业、兴趣或成长加点写入；分配方式不明确时应先确认。

同一条 `.aicecard` 消息可以继续粘贴背景内容，AI 会分别填入对应栏目。例如：

```text
.aicecard 力量60 敏捷65 意志50
体质35 外貌40 教育85
体型65 智力70 幸运50

描述：长头发、乐观的、虚弱的、女人
思想信念：科学万能！科学万岁！
重要之人：父辈。
重要之人理由：你试图证明你比他们更出色。他们的缺点是？
意义非凡之地：重要之人的坟墓。
宝贵之物：与你得意技相关之物。
特点：善待动物。
```

“描述”写入形象描述，“思想信念”写入思想与信念，“特点”写入特质；其余栏目按同名字段填写。“重要之人理由”与重要之人合并，例如 `父辈。\n你试图证明你比他们更出色。他们的缺点是？`，只用换行连接，不添加“理由：”前缀；保留原文中的问题，不由 AI 补写答案。MCP 接受上述背景别名；直接 HTTP 导出仍使用 `/fields` 返回的标准字段 ID，例如 `background.形象描述`。

标题 `『调查员｜…』` 不自动作为姓名或玩家名；以 `『序号…』` 开头的职业资料段暂不录入，也不将其信誉范围、职业属性、本职技能或介绍填入人物卡或当作实际加点。已有职业字段接口保持可用，但本次粘贴解析仅处理属性和背景。

`character_card_export` 的示例参数为：

```json
{
  "template_id": "coc7",
  "changes": {
    "力量": 55, "敏捷": 25, "意志": 70,
    "体质": 50, "外貌": 50, "教育": 55,
    "体型": 60, "智力": 75, "幸运": 75,
    "HP": 11
  }
}
```

修改时发送 `.aicechat 把人物卡 <返回的卡ID> 的力量改成60并重新导出`。工具只覆盖明确提交的字段，其余已填数据保留；`null` 可清空字段。每次导出产生新 ID，旧文件不被覆盖。姓名、背景等未提供的信息不会由工具自动补齐。

导出文件暂存在当前 Aice 进程内存中，最多保留 64 份，每份有效期一小时，重启后失效；到期或不存在的 ID 返回 404。下载链接形如 `/ui/character-cards/exports/<卡ID>`，已包含在现有 Nginx `/ui/` 转发范围内。知道链接的人可以下载该文件；当前没有账号归属管理，不提供列出所有人物卡的接口。

要在 QQ 或远程浏览器点击下载，在 `config.yaml` 增加用户能够访问的根地址，例如：

```yaml
public_base_url: "https://aice.example.com"
```

地址也可使用服务器 IP 和公开端口。未配置时仅返回相对下载路径。部署方式仍按前文启动 Aice 和 Nginx；在 Excel/WPS 打开文件后重算公式。命令行 `deepseek` 示例只执行文本补全，不连接 MCP；生成和下载人物卡请通过 HTTP 服务。

MCP 的 Excel 任务最多同时执行两个；单次聊天最多六轮模型调用、十二次 MCP 工具调用，总超时为 120 秒。同一轮的工具调用全部处理后，由后端汇总实际导出文件的地址；后续操作失败或超时时保留已收到的成功结果，并提示部分操作未完成。工具具备输入/输出 schema、结构化返回值和行为 annotations。

```sh
curl -fsS -H 'Content-Type: text/plain; charset=utf-8' \
  --data-binary '用一句话介绍一下 COC 跑团' http://127.0.0.1:3210/chat
```

### 呆猫插件接入

使用现有海豹机器人接入，插件沿用扩展 ID `aice-callback`，保留 `.aicechat` 并增加专用建卡指令 `.aicecard`。不需要单独部署 MCP，也不需要把模型密钥填入插件。

1. 在海豹 WebUI 的「扩展功能 → JS 扩展」上传或替换 `plugins/sealdice/aice-callback.js`（0.3.1），然后重载 JS；替换旧文件，不要同时保留两份插件。
2. 扩展配置 `chat_url` 保持 `http://aice:3210/chat`，用于同一 Docker 网络中的海豹访问 Aice。确认 Aice 接入了海豹实际使用的网络，必要时设置 `.env` 的 `SEALDICE_NETWORK`。两个服务都直接在同一宿主机运行时才使用 `http://127.0.0.1:3210/chat`；容器内的 `127.0.0.1` 指容器自己。
3. 在后端 `config.yaml` 配置用户可访问的 `public_base_url`。默认 Nginx 只提供 HTTP，未配置 TLS 时应填写 `http://你的服务器IP`；这个地址用于下载，不能填写 Docker 内网服务名。修改后执行 `docker compose restart aice`，重启会清空已有暂存人物卡。
4. 在目标会话发 `.ext aice-callback on`，然后发 `.aicecard help` 检查是否能收到帮助。先验证 `.aicechat 你好`，再发送下面的建卡数据。

```text
.aicecard 力量:55 敏捷:25 意志:70
体质:50 外貌:50 教育:55
体型:60 智力:75 幸运:75
HP:11 <DB:0> [440/515]
```

呆猫会把后端生成的下载地址和卡 ID 回复到当前会话。插件发送文本链接，不直接上传群文件。收到结果后可以继续发送：

```text
.aicecard 把人物卡 <返回的卡ID> 的力量改成60，其他字段保持不变，重新导出。
```

普通聊天继续使用 `.aicechat <问题>`。空输入、`help` 或 `帮助` 会显示指令用法。使用 QQ 官方机器人的斜杠指令时，需要在 QQ 开放平台登记要使用的 `aicechat`、`aicecard` 指令。

若提示无法连接，检查 Docker 网络与 `chat_url`；503 表示后端未配置模型，502 需查看 `docker compose logs --tail=100 aice`。直接 HTTP 建卡已成功、插件也能回复但链接打不开时，检查 `public_base_url` 的协议、地址和 Nginx 转发。插件不保存聊天历史或自动记忆卡 ID，修改时需明确附带 ID。

## 机器人人设预览

启动 Aice 后，在浏览器打开 `http://127.0.0.1:3210/ui/personas`。页面由 Leptos 组件在服务端渲染，点击列表项可展开示例人设。

要用公网 IP 访问人设页，在 Linux 服务器的项目目录运行以下命令。

```sh
docker compose up -d --build web
```

Nginx 默认监听宿主机的 80 端口。云安全组和服务器防火墙放行对应的 TCP 端口后，访问 `http://服务器IP/` 会跳转到 `/ui/personas`。Nginx 转发整个 `/ui/` 路径。新增页面或样式时，把路由注册到 Axum 的 `frontend` 路由组即可。

如果 80 端口已有服务，在 `.env` 设置 `AICE_PUBLIC_PORT=8080` 后重新运行命令，并通过 `http://服务器IP:8080/` 访问。

## 包与目录

项目使用 Cargo workspace，共四个包；公共依赖版本集中在根 `Cargo.toml`，包之间单向依赖：

```text
aice → character-card-mcp → character-card → character-card-templates
  └───────────────────────────────↑
```

```text
aice/
├── Cargo.toml                   # workspace 与 aice 应用包
├── AGENTS.md                    # 开发约定入口
├── AGENT.md                     # 三个 Rust skill、mcp-builder 与包边界
├── character-card-templates/    # 资源包：Excel 文件与编译时嵌入
├── crates/
│   ├── character-card/          # 核心包：读取、字段映射、修改和 Excel 导出
│   │   └── src/
│   │       ├── lib.rs           # 模板注册、CoC7 读取与公共接口
│   │       ├── document.rs      # CharacterCardDocument trait 与通用实现
│   │       ├── coc7_fields.rs   # CoC7 可编辑字段映射
│   │       └── xlsx.rs          # 保留样式和公式的 XLSX 写入
│   └── character-card-mcp/      # MCP 包：工具定义、字段别名和导出暂存
│       └── src/
│           ├── lib.rs          # 公共接口
│           ├── server.rs       # MCP 工具与 schema
│           └── exports.rs      # 暂存、过期管理及修改后导出
├── src/
│   ├── bin/aice.rs              # HTTP 服务入口
│   ├── lib.rs                   # 应用模块入口
│   ├── config/                 # YAML 和环境变量配置
│   ├── init/                   # 组装数据库、模型、Agent 与导出存储
│   ├── router/                 # HTTP 路由
│   ├── handle/                 # HTTP 请求与响应
│   ├── frontend/               # Leptos 页面和样式
│   ├── logic/
│   │   ├── chat.rs             # 聊天输入和可用性检查
│   │   └── agent/              # MCP 连接、模型/工具循环和结果汇总
│   │       └── instructions.md # 人物卡 Agent 指令
│   ├── service/                # 模型接口和数据库服务
│   ├── dal/                    # PostgreSQL 数据访问
│   └── infra/llm.rs            # DeepSeek 配置与单次模型补全
├── examples/deepseek.rs         # 文本模型调用示例
├── plugins/sealdice/            # 海豹接入
└── deploy/                     # 部署配置
```

人物卡核心包不依赖 Web 框架、模型或 Tokio。MCP 包不依赖 Aice 和 DeepSeek，可以单独构建并由其他 Rust 宿主接入；目前没有独立 MCP 可执行程序。模板包把 Excel 编译进二进制，运行时无需查找源码目录。原来的 `aice::service::character_card` 路径保留为核心包的重导出。

```sh
cargo build --locked -p character-card-templates
cargo build --locked -p character-card
cargo build --locked -p character-card-mcp
cargo build --locked -p aice --bin aice

cargo fmt --all --check
cargo check --workspace --locked
cargo check --locked --example deepseek
cargo clippy --workspace --locked -- -D warnings
```

包可独立编译；部署产物仍为一个 `aice` 二进制。Docker 构建会复制所有 workspace 包，运行镜像只包含二进制和 CA 证书。`cargo run` 默认启动根包。

启动时，`main` 通过 `init::AppState::new` 创建共享的连接池、模型、Agent 和导出存储，再交给 Axum。

`POST /chat` 经 `router → handle::chat → logic::chat → logic::agent` 执行。Agent 通过 `service::agent::AgentModel` 请求补全，由 `infra::llm` 调用 DeepSeek；工具调用经进程内 MCP 交给 `character-card-mcp`，再调用 `character-card` 完成人物卡操作。

`GET /ui/personas` 经 `router → handle::personas → frontend::pages::personas` 渲染人设列表，浏览器通过 `/ui/assets/personas.css` 加载样式。PostgreSQL 的连接由 `init → service::database → dal::postgres` 建立。
