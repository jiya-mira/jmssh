# AGENTS.md (jmssh)

本文件定义本仓库内 agent 的默认工作方式，目标是：**最小改动、可验证、可回滚**。

## 1. 项目速览

- 语言与构建：Rust (`edition = 2024`), `cargo`
- 产物形态：单二进制 CLI/TUI 工具 `jmssh`
- 核心能力：SSH profile 管理、连接编排、密码托管（OS keyring）
- 持久化：SQLite（SeaORM）

关键目录：

- `src/main.rs`：入口与命令分发
- `src/cli.rs`：CLI 参数定义（clap）
- `src/handlers/*`：命令处理层（参数编排、输出、调用 usecase）
- `src/usecase/*`：业务逻辑层（主要规则与查询组合）
- `src/entity/*`：SeaORM 实体与数据模型
- `src/infra/*`：外部系统适配（如 password store）
- `src/ui/*`：交互式 TUI
- `src/db.rs`：数据库路径与 schema 初始化

## 2. 分层约束（必须遵守）

- `cli` 只负责参数结构，不放业务逻辑。
- `handlers` 负责 I/O 与流程编排；避免堆业务规则。
- `usecase` 承载业务规则、错误语义与数据拼装。
- `infra` 只做外部依赖适配（keyring/系统命令等），不反向依赖 UI。
- `entity` 保持数据模型清晰，避免在业务层散落裸字段字符串。

## 3. 安全与隐私红线

- 禁止把密码写入数据库、日志、调试输出。
- 禁止提交密钥、token、账号密码、真实主机敏感信息。
- 日志默认脱敏：可记录 profile label / host 概要，不记录 secret。
- 任何新增“显示敏感信息”能力必须显式 warning，并保持默认关闭。

## 4. 改动策略

- 默认路径：`局部定位 -> 定点读取 -> 最小改动 -> 最小验证`。
- 一次只做一个主题，避免顺手重构无关代码。
- 优先复用现有模块与模式，非必要不加新依赖。
- 错误处理必须显式，禁止静默吞错。
- 涉及 CLI 语义变更时，同步更新 `README.md` 的对应命令说明。

## 5. 常用命令

开发常用：

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

本地运行示例：

```bash
cargo run -- init
cargo run -- profile list
cargo run -- connect <label>
```

## 6. 验证基线

默认最小验证（按改动范围选择）：

- 仅文档改动：检查 Markdown 结构与命令可读性。
- Rust 代码改动：至少执行 `cargo fmt --all` + `cargo build`。
- 涉及逻辑分支/错误处理：补跑 `cargo test`（若无测试则补最小 smoke 验证说明）。
- 涉及 CLI 参数/命令行为：补一条可复现的命令级验证记录。

## 7. 提交前检查

- 若 IntelliJ MCP 可用，提交前执行一轮 inspection 扫描。
- 对“行为不变”的安全优化/简化项可直接修复。
- 对高风险或语义不明确项，不强改，在提交说明列建议。
- 若 MCP 不可用，明确说明“未执行 inspection 扫描”。

## 8. 对 agent 的额外要求

- 先看 `git status`，不得回滚用户已有未提交改动。
- 禁止使用破坏性命令（如 `git reset --hard`）除非用户明确授权。
- 搜索优先 `rg`，读取优先定点文件，避免无目的全仓扫描。
- 输出给用户时说明：改了什么、为什么、如何验证、剩余风险。

## 9. 提交信息建议

- 建议使用 Conventional Commits。
- 示例：
  - `feat(connect): support jump chain by profile route order`
  - `fix(password): handle missing keyring entry gracefully`
  - `docs(readme): clarify sshpass fallback behavior`
