# 项目开发约定

## Rust skills

每次编写、修改或重构项目代码前，必须使用以下三个 skill：

1. **rust-architect**：检查模块职责、依赖方向、目录结构及 trait 边界。
2. **rust-async-patterns**：检查异步调用、阻塞操作、任务生命周期、并发控制及错误传播。
3. **rust-best-practices**：检查所有权与借用、类型设计、错误处理、克隆与分配，以及 Rust 代码惯用写法。

从当前环境的 skills 列表找到并阅读各自的 `SKILL.md`，按需读取相关参考资料；同一会话已读且未更新的内容无需重复读取。三个 skill 都要参与判断，与当前改动无关的建议不引入代码。若 skill 缺失，明确说明，不能声称已经使用。

## MCP 与 Agent 规范

涉及 MCP 工具或 Agent 工具调用时，必须额外使用 **mcp-builder** skill，读取其 `SKILL.md` 及适用的 MCP best practices。沿用 Rust 官方 SDK `rmcp`，不因为 skill 的语言示例改用 TypeScript 或 Python。

- 工具采用统一的 `character_card_` 前缀，明确参数说明、输入/输出 schema、结构化结果及只读、幂等、破坏性、开放世界 annotations。
- 参数校验失败返回工具错误及可执行的修正建议；协议或服务故障与业务错误区分处理。
- Agent 负责工具发现、调用循环和结果汇总；模型适配器只负责模型配置和单次补全调用。保留模型原始工具调用 ID、消息及推理内容，不丢弃同轮调用。
- 阻塞的 Excel 处理使用 `spawn_blocking` 并限制并发；明确超时、调用次数及 MCP 连接生命周期。产生文件的调用不自动重试，下载链接只能来自实际导出结果。
- 当前只接入 Aice 内部聊天，使用进程内 MCP；不自行新增外部监听、身份体系或数据库存储。

## 包与目录边界

- 根包 `aice`：HTTP、配置、初始化、Agent 编排和外部模型适配。
- `character-card-templates/`：独立资源包，编译时嵌入 Excel 模板，不依赖运行目录。
- `crates/character-card/`：读取、字段映射、创建、修改和 Excel 导出，不依赖 HTTP、模型或异步运行时。
- `crates/character-card-mcp/`：MCP 工具、字段别名和导出暂存，依赖人物卡核心，不依赖 Aice 或具体模型。
- 共享依赖版本由根 `Cargo.toml` 的 `[workspace.dependencies]` 管理；仅实际使用的包声明依赖。拆包时同步维护调用方、示例、Docker 构建和 README。

## 项目约束

- 用户明确要求优先于 skill 的默认流程。
- 不编写或添加测试代码、测试目录、测试样本和仅供测试使用的依赖；不执行 skill 中强制 TDD 或覆盖率要求。
- 优先沿用现有结构，按实际需求拆分模块、引入 trait 或拆分 crate；不为套用架构模板新增空目录、占位实现或多层封装。
- 不因使用 skill 自动生成整套架构文档、引入依赖或扩展功能。
- 默认直接完成工作，不因 skill 的通用流程重复询问已知信息；仅在用户明确要求时使用多代理协作。
- 保留必要的输入校验、错误处理及防止数据丢失的逻辑。
- Rust 代码改动后运行适用的 `cargo fmt --all --check`、`cargo check --workspace --locked` 和 `cargo clippy --workspace --locked -- -D warnings`，检查受影响的示例；若无法完成，说明原因。纯文档修改只检查内容和差异格式。

## CodeGraph

若项目根目录存在 `.codegraph/`，理解或定位代码时优先使用 `codegraph_explore` 或 `codegraph explore`，再进行文本搜索或文件读取。不存在时跳过，不自动建立索引。
