# 更新日志

## [0.2.0] - 2026-08-12

### 重构

- **模块化架构**：将原来的上帝文件 `lib.rs`（25KB）拆分为 `conn.rs`、`xloper.rs`、`error.rs`、`ffi.rs` 等独立模块
- **业务拆分**：将臃肿的 `functions/execute.rs` 拆分为 `exec.rs`、`table.rs`、`csv_import.rs`
- **工具提取**：新增 `utils/types.rs`，集中管理类型推断、列名清洗、SQL 类型规范化，消除重复代码
- **清理死代码**：删除空占位模块 `core/result.rs`、`core/types.rs`、`utils/error.rs`

### 测试

- 新增 30+ 单元测试和集成测试
- 纯算法测试（类型推断、列名清洗）放在 `src/utils/types/tests.rs`
- 数据库/IO 测试放在 `tests/` 目录，使用内存数据库和临时文件
- 所有测试无需启动 Excel 即可运行

### 文档

- 重写 README，采用"电梯演讲 + 链接下沉"结构
- 新增 `docs/API.md`（完整函数手册）
- 新增 `docs/Install.md`（安装排障指南）
- 新增 `docs/FAQ.md`（常见问题）
- 新增 `docs/Contributing.md`（开发贡献指南）

## [0.1.0] - 2024-XX-XX

### 初始版本

- 12 个 Excel 函数：SqlQuery / SqlQueryP / SqlQueryL / SqlExec / SqlCreateDb / SqlCreateTable / SqlImportCsv / SqlConnect / SqlDisconnect / SqlTables / SqlVersion / SqlSchema / SqlPragma
- 连接缓存与句柄管理
- CSV 自动编码识别（UTF-8 / GBK）
- 自动列类型推断
- 参数化查询防注入
