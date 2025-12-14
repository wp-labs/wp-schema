# Repository Guidelines

## 项目结构与模块
- Rust 库 crate：`wp-schema`（lib 名 `wp_schema`），源码位于 `src/`。
- 入口：`src/lib.rs` 暴露 `engine` 与 `model`。
- 模型：`src/model.rs` 定义 `FieldType`、`SQLTable` 与解析（基于 winnow）。
- 引擎：`src/engine/` 包含 `mysql.rs`、`clickhouse.rs`、`elasticsearch.rs`，负责 SQL/映射生成。
- 测试建议：模块内单测 `#[cfg(test)]`；跨模块放置于 `tests/`。

## 构建、测试与本地开发
- 调试构建：`cargo build`
- 发布构建：`cargo build --release`
- 可选特性示例：`cargo build --features "actix-web,prometheus,regex,uuid"`
- 运行测试：`cargo test`
- Lint/格式化：`cargo clippy --all-targets --all-features -- -D warnings`；`cargo fmt --all`
- 文档：`cargo doc --no-deps --open`

## 代码风格与命名
- Rust 2024；使用 `rustfmt` 默认风格（4 空格缩进）。
- 命名：模块/文件 `snake_case`；类型/枚举 `UpperCamelCase`；函数/变量 `snake_case`；常量 `SCREAMING_SNAKE_CASE`。
- 错误处理：优先 `wp_err` 领域错误；胶水层使用 `anyhow::Result<T>`；避免 `unwrap/expect`。
- 解析：统一使用 `winnow`，避免 panic，返回带上下文的错误信息。

## 测试规范
- 覆盖关键路径：`FieldType::from_str`、`engine::*::create_table`、类型映射函数。
- 测试命名：`<模块>_<行为>`，如 `mysql_creates_auto_increment_pk`。
- 断言建议：校验完整 SQL 字符串；覆盖默认值/NOT NULL/数组与 decimal 等边界。

## 提交与合并请求
- 提交遵循 Conventional Commits（如 `feat(mysql): handle default string`、`fix(clickhouse): quote order_by`)。
- 主题 ≤ 72 字符；正文说明动机、影响与兼容性。
- PR 要求：变更说明、关联 Issue、测试更新、通过 `fmt`/`clippy`、附示例输入/输出（`SQLTable` → 期望 `CREATE TABLE ...`）。

## 安全与配置
- 依赖包含内部 Git 仓库（cnb.cool），确保网络与凭据可用。
- 禁止提交密钥/令牌；使用本地 `.env` 与 `.gitignore` 已忽略的文件。
- 构建产物位于 `target/`，无需提交。

