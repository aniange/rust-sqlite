# rust-sqlite-xll

> 在 Excel / WPS 中直接运行 SQLite，支持 SQL 查询、参数化防注入、CSV 智能导入、自动类型推断。

[安装指南](./docs/Install.md) · [快速开始](#快速开始) · [API 文档](./docs/API.md) · [常见问题](./docs/FAQ.md) · [更新日志](./CHANGELOG.md)

---

## 功能亮点

- **零配置查询**：`=SqlQuery("SELECT * FROM users")` 直接返回二维数组
- **参数化防注入**：`SqlQueryP` 支持 `?` 占位符，彻底杜绝 SQL 注入
- **CSV 智能导入**：自动识别 UTF-8 / GBK 编码，自动推断列类型
- **内存数据库**：省略路径即可使用共享内存 DB，跨工作表数据互通
- **分页查询**：`SqlQueryL` 自动追加 LIMIT/OFFSET，百万级数据不卡死

---

## 安装

1. 下载 `rust_sqlite.xll`（[Releases](../../releases)）
2. Excel → 文件 → 选项 → 加载项 → 管理：Excel 加载项 → 转到 → 浏览 → 选择 `.xll`
3. 重启 Excel，输入 `=SqlVersion()` 验证

> 详细安装指南（32/64 位选择、WPS 兼容、常见问题排障）→ [docs/Install.md](./docs/Install.md)

---

## 快速开始

### 1. 查询现有数据库
```excel
=SqlQuery("C:\\data\\test.db", "SELECT * FROM sales LIMIT 10")
```

### 2. 从 Excel 区域建表
```excel
=SqlCreateTable(,"orders", A1:D100)
```
> 省略第一个参数使用内存数据库。自动将首行作为列名，并推断每列的 SQLite 类型。

### 3. 参数化查询（防注入）
```excel
=SqlQueryP(,"SELECT * FROM users WHERE name = ? AND age > ?", "Alice", 18)
```

### 4. 导入 CSV 文件
```excel
=SqlImportCsv(,"C:\\data\\file.csv", "imported_table")
```
> 自动识别 UTF-8 / GBK 编码，支持自定义分隔符。

---

## 函数速查表

| 函数 | 用途 | 复杂度 |
|------|------|--------|
| `SqlQuery` | 执行 SELECT 返回结果集 | ⭐ |
| `SqlQueryP` | 参数化查询（防注入） | ⭐⭐ |
| `SqlQueryL` | 分页查询（LIMIT/OFFSET） | ⭐⭐ |
| `SqlExec` | 执行 INSERT/UPDATE/CREATE | ⭐ |
| `SqlCreateTable` | 从 Excel 区域建表 | ⭐⭐ |
| `SqlImportCsv` | 导入 CSV 文件 | ⭐⭐ |
| `SqlConnect` / `SqlDisconnect` | 连接管理 | ⭐ |
| `SqlTables` / `SqlSchema` / `SqlPragma` | 元数据查询 | ⭐⭐ |

> 📖 **完整 API 文档**（含每个函数的参数表、返回值、错误码、详细示例）→ [docs/API.md](./docs/API.md)

---

## 项目结构

```
src/
├── conn.rs              # 连接池、句柄映射、内存数据库 URI
├── xloper.rs            # Excel XLOPER12 ↔ Rust 类型转换
├── error.rs             # 错误码映射（#REF! / #NAME? / #VALUE!）
├── ffi.rs               # Excel FFI 导出层（所有 #[no_mangle] 函数）
├── functions/
│   ├── query.rs         # SqlQuery / SqlQueryP / SqlQueryL
│   ├── exec.rs          # SqlExec / SqlCreateDb
│   ├── table.rs         # SqlCreateTable
│   ├── csv_import.rs    # SqlImportCsv（含编码自动识别）
│   └── metadata.rs      # SqlTables / SqlVersion / SqlSchema / SqlPragma
├── utils/
│   └── types.rs         # 类型推断、列名清洗、SQL 类型规范化
└── core/
    └── pool.rs          # ConnectionPool 结构体
```

> 🔧 架构详解、本地编译指南、测试运行方式 → [docs/Contributing.md](./docs/Contributing.md)

---

## 常见问题速查

| 错误 | 含义 | 解决 |
|------|------|------|
| `#REF!` | 数据库/表/列不存在 | 检查路径或表名 |
| `#NAME?` | SQL 语法错误或连接失败 | 检查 SQL 语句拼写 |
| `#VALUE!` | 查询执行失败或参数错误 | 检查参数数量和类型 |

> 更多问题（中文乱码、32/64 位冲突、WPS 兼容）→ [docs/FAQ.md](./docs/FAQ.md)

---

## 许可证

MIT License
