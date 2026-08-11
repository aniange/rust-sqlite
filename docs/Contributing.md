# 开发指南

## 项目架构

本项目采用分层架构，职责分离清晰：

```
┌─────────────────────────────────────┐
│  ffi.rs          Excel FFI 导出层    │  ← 唯一与 Excel 交互的层
├─────────────────────────────────────┤
│  functions/      业务函数实现        │  ← query/exec/table/csv_import/metadata
├─────────────────────────────────────┤
│  conn.rs         连接生命周期管理    │  ← 缓存、句柄、内存 DB
├─────────────────────────────────────┤
│  xloper.rs       XLOPER 类型转换   │  ← Excel 数组 ↔ Rust Vec
│  error.rs        错误码映射          │  ← Rust 错误 → Excel #ERR!
│  utils/types.rs  纯逻辑工具          │  ← 类型推断、列名清洗
└─────────────────────────────────────┘
```

### 各模块职责

| 模块 | 职责 | 不做什么 |
|------|------|----------|
| `ffi.rs` | 所有 `#[no_mangle]` 导出函数、函数注册 | 不包含任何 SQL 业务逻辑 |
| `functions/query.rs` | SELECT 查询、参数绑定、分页 | 不管理连接 |
| `functions/exec.rs` | 非查询 SQL 执行、创建数据库 | 不处理 Excel 数据区域 |
| `functions/table.rs` | 从 Excel 区域解析并建表 | 不处理 CSV 文件 |
| `functions/csv_import.rs` | CSV 读取、编码识别、建表 | 不处理 Excel 区域 |
| `functions/metadata.rs` | 表列表、版本、结构、PRAGMA | 不执行用户 SQL |
| `conn.rs` | 连接缓存、句柄映射、with_conn | 不执行 SQL |
| `xloper.rs` | XLOPER ↔ Rust 双向转换 | 不接触数据库 |
| `utils/types.rs` | 类型推断、列名清洗、类型规范化 | 不依赖任何外部 crate |

## 本地编译

### 环境要求

- Windows（XLL 只能在 Windows 上运行）
- Rust 工具链（`rustup`）
- MSVC 构建工具（Visual Studio Build Tools）

### 编译命令

```bash
# 64 位版本（默认）
cargo build --release

# 32 位版本（兼容 WPS）
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

编译产物位于 `target/release/rust-sqlite.xll`（64 位）或 `target/i686-pc-windows-msvc/release/rust-sqlite.xll`（32 位）。

## 运行测试

```bash
# 运行所有测试（包括单元测试和集成测试）
cargo test

# 只运行单元测试（纯算法，秒级）
cargo test --lib

# 只运行集成测试（需要数据库/文件 IO）
cargo test --test test_table
cargo test --test test_csv_import
cargo test --test test_exec
```

### 测试分层

| 类型 | 位置 | 覆盖内容 | 是否需要 Excel |
|------|------|----------|---------------|
| 单元测试 | `src/utils/types/tests.rs` | 类型推断、列名清洗、类型规范化 | ❌ |
| 单元测试 | `src/functions/query.rs` | SQL 分页拼接逻辑 | ❌ |
| 集成测试 | `tests/test_exec.rs` | SqlExec 执行、建库 | ❌ |
| 集成测试 | `tests/test_table.rs` | SqlCreateTable 自动推断、显式列名/类型、幂等重建 | ❌ |
| 集成测试 | `tests/test_csv_import.rs` | CSV 导入（UTF-8/GBK、制表符、自定义列/类型） | ❌ |

所有测试均使用内存数据库或临时文件，**不需要启动 Excel**。

## 添加新函数

如果你想添加一个新的 Excel 函数（如 `SqlVacuum`），需要修改三个文件：

1. **`src/functions/exec.rs`**（或新建模块）添加 `impl` 函数：
   ```rust
   pub fn sqlvacuum_impl(conn_str: &str) -> Result<String, String> {
       with_conn(conn_str, |conn| {
           conn.execute("VACUUM", [])
               .map_err(|e| format!("Vacuum failed: {}", e))?;
           Ok("Database vacuumed".to_string())
       })
   }
   ```

2. **`src/ffi.rs`** 添加 `#[no_mangle]` 导出函数：
   ```rust
   #[no_mangle]
   pub extern "system" fn sqlvacuum(conn_str: *mut XLOPER12) -> *mut XLOPER12 {
       // ... 参数解析 ...
       match sqlvacuum_impl(&conn) {
           Ok(msg) => Box::into_raw(Box::new(XLOPER12::from_str(&msg))),
           Err(e) => Box::into_raw(Box::new(error_to_xloper(&e))),
       }
   }
   ```

3. **`src/ffi.rs`** 的 `xlAutoOpen` 中添加函数注册：
   ```rust
   let _ = reg.add(
       "sqlvacuum",
       &build_type_string('Q', &['Q'], true, false, false),
       "SqlVacuum",
       "conn_str",
       "SQLite",
       "Reclaim free space and optimize the database",
       &["Database handle, full file path, or omit for in-memory database"],
   );
   ```

## 代码风格

- 使用 `cargo fmt` 自动格式化
- 使用 `cargo clippy` 检查常见代码问题
- 所有 `pub` 函数必须有文档注释
- 纯逻辑函数（如 `infer_column_type`）优先放在 `utils/types.rs`，便于独立测试
