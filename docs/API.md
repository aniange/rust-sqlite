# API 文档

> 本文档包含所有导出函数的完整说明，按使用场景分组。

---

## 目录

- [通用参数](#通用参数)
- [连接管理类](#连接管理类)
  - [SqlConnect](#sqlconnect)
  - [SqlDisconnect](#sqldisconnect)
- [查询类](#查询类)
  - [SqlQuery](#sqlquery)
  - [SqlQueryP](#sqlqueryp)
  - [SqlQueryL](#sqlqueryl)
  - [SqlQueryScalar](#sqlqueryscalar)
- [执行类](#执行类)
  - [SqlExec](#sqlexec)
  - [SqlScript](#sqlscript)
- [事务控制类](#事务控制类)
  - [SqlBegin](#sqlbegin)
  - [SqlCommit](#sqlcommit)
  - [SqlRollback](#sqlrollback)
- [数据导入类](#数据导入类)
  - [SqlCreateTable](#sqlcreatetable)
  - [SqlImportCsv](#sqlimportcsv)
  - [SqlImportCsvDir](#sqlimportcsvdir)
  - [SqlAppendTable](#sqlappendtable)
- [数据导出类](#数据导出类)
  - [SqlExportCsv](#sqlexportcsv)
- [元数据类](#元数据类)
  - [SqlTables](#sqltables)
  - [SqlVersion](#sqlversion)
  - [SqlSchema](#sqlschema)
  - [SqlPragma](#sqlpragma)

---

## 通用参数

### `conn_str`（连接字符串）

所有函数的第一个参数均为连接标识，支持三种形式：

| 形式 | 示例 | 说明 |
|------|------|------|
| **省略 / 空字符串** | `=SqlQuery(,"SELECT ...")` | 使用共享内存数据库，数据在 Excel 进程存活期间保持 |
| **文件路径** | `=SqlQuery("C:\data\test.db", ...)` | 直接指定 SQLite 数据库文件的完整路径 |
| **连接句柄** | `=SqlQuery("conn_1", ...)` | 使用 `SqlConnect` 返回的句柄，连接会被缓存复用 |

> 路径中的正斜杠 `/` 和反斜杠 `\` 均可使用。

### 错误代码

| 错误 | 触发条件 |
|------|----------|
| `#REF!` | 数据库文件不存在、表不存在、列不存在 |
| `#NAME?` | SQL 语法错误、连接失败、无法识别的 token |
| `#VALUE!` | 参数数量/类型不匹配、查询执行失败、CSV 解析错误 |

#### [<ins>返回目录</ins>](#目录)
---

## 连接管理类

### SqlConnect

连接到数据库并返回一个可复用的句柄。

**语法**
```excel
=SqlConnect([db_path])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `db_path` | 字符串 | 否 | 数据库文件路径，省略则使用内存数据库 |

**返回值**

连接句柄（如 `"conn_1"`、`"conn_memory"`）。

**示例**
```excel
=SqlConnect()                    ' 内存数据库
=SqlConnect("C:\data\app.db")    ' 文件数据库
```

**注意事项**
- 句柄对应的连接会被缓存，后续使用该句柄的公式无需重新打开文件
- 内存数据库的句柄固定为 `"conn_memory"`

#### [<ins>返回目录</ins>](#目录)
---

### SqlDisconnect

断开连接句柄或关闭指定路径的缓存连接。

**语法**
```excel
=SqlDisconnect(handle_or_path)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `handle_or_path` | 字符串 | 是 | 句柄（如 `"conn_1"`）或完整文件路径 |

**返回值**

成功返回断开确认信息。

**示例**
```excel
=SqlDisconnect("conn_1")
=SqlDisconnect("conn_memory")
=SqlDisconnect("C:\data\app.db")
```

**注意事项**
- 断开 `"conn_memory"` 仅释放句柄，数据在 Excel 进程退出前仍然保留
- 断开文件连接会关闭底层数据库连接并清除缓存

#### [<ins>返回目录</ins>](#目录)
---

## 查询类

### SqlQuery

执行 SELECT 查询，返回二维数组（首行为列名）。

**语法**
```excel
=SqlQuery([conn_str], sql)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄，省略使用内存数据库 |
| `sql` | 字符串 | 是 | SELECT 语句 |

**返回值**

二维数组。首行为列名，后续行为数据。空查询返回空字符串。

**示例**
```excel
=SqlQuery("C:\data\sales.db", "SELECT * FROM orders WHERE amount > 1000")
=SqlQuery(,"SELECT name, score FROM students ORDER BY score DESC")
```

**注意事项**
- 返回大量数据时 Excel 可能卡顿，建议配合 `SqlQueryL` 分页使用
- 日期时间以 INTEGER（Unix 时间戳）或 TEXT 形式返回，需手动设置单元格格式

#### [<ins>返回目录</ins>](#目录)
---

### SqlQueryP

执行带参数绑定的 SELECT 查询，防止 SQL 注入。

**语法**
```excel
=SqlQueryP([conn_str], sql, [p1], [p2], [p3], [p4], [p5])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `sql` | 字符串 | 是 | 含 `?` 占位符的 SELECT 语句 |
| `p1` ~ `p5` | 任意 | 否 | 与 `?` 一一对应的参数值，最多 5 个 |

**返回值**

同 `SqlQuery`。

**示例**
```excel
=SqlQueryP(,"SELECT * FROM users WHERE name = ? AND age > ?", "Alice", 18)
=SqlQueryP("conn_1","SELECT * FROM logs WHERE level = ? AND date = ?", "ERROR", "2024-01-01")
```

**注意事项**
- `?` 的数量必须与提供的参数数量完全一致，否则会返回 `#VALUE!`
- 参数类型会自动推断：数字 → INTEGER/REAL，文本 → TEXT

#### [<ins>返回目录</ins>](#目录)
---

### SqlQueryL

执行分页查询，自动追加 LIMIT/OFFSET。

**语法**
```excel
=SqlQueryL([conn_str], sql, [limit], [offset])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `sql` | 字符串 | 是 | SELECT 语句（**不要**包含 LIMIT/OFFSET） |
| `limit` | 数字 | 否 | 返回的最大行数 |
| `offset` | 数字 | 否 | 跳过的行数 |

**返回值**

同 `SqlQuery`。

**示例**
```excel
=SqlQueryL(,"SELECT * FROM big_table", 1000, 0)         ' 第 1 页
=SqlQueryL(,"SELECT * FROM big_table", 1000, 1000)      ' 第 2 页
=SqlQueryL(,"SELECT * FROM logs ORDER BY id DESC", 50)  ' 最新 50 条
```

**注意事项**
- 如果 `sql` 中已包含 `LIMIT`（不区分大小写），则不会追加分页，避免冲突
- 子查询中含 `LIMIT` 也会被检测到，此时分页不会生效（保守策略）

#### [<ins>返回目录</ins>](#目录)
---

### SqlQueryScalar

执行查询并返回第一行第一列的单个标量值。

**语法**
```excel
=SqlQueryScalar([conn_str], sql)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `sql` | 字符串 | 是 | 应返回单行单列的 SELECT 语句 |

**返回值**

单个值（数字、文本或空字符串）。

**示例**
```excel
=SqlQueryScalar(,"SELECT COUNT(*) FROM users")
=SqlQueryScalar(,"SELECT MAX(score) FROM exam")
=SqlQueryScalar("C:\data\app.db","SELECT name FROM users WHERE id = 1")
```

**注意事项**
- 查询无结果时会返回 `#VALUE!`
- 即使查询返回多行多列，也只取第一行第一列
- 适合 COUNT/SUM/MAX/MIN 等聚合查询

#### [<ins>返回目录</ins>](#目录)
---

## 执行类

### SqlExec

执行 INSERT / UPDATE / DELETE / CREATE TABLE 等非查询语句。

**语法**
```excel
=SqlExec([conn_str], sql)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `sql` | 字符串 | 是 | 要执行的 SQL 语句 |

**返回值**

受影响的行数（数字）。

**示例**
```excel
=SqlExec(,"CREATE INDEX idx_name ON users(name)")
=SqlExec(,"UPDATE orders SET status = 'shipped' WHERE id = 100")
=SqlExec(,"DELETE FROM logs WHERE created_at < '2023-01-01'")
```

**注意事项**
- 不能用于 SELECT 查询（不会返回结果集）
- DDL 语句（CREATE/DROP）返回 0

#### [<ins>返回目录</ins>](#目录)
---

### SqlScript

执行包含多条语句的 SQL 脚本，或读取外部 `.sql` 文件执行。

**语法**
```excel
=SqlScript([conn_str], script_or_path)
```
**参数**
| 参数               | 类型  | 必填 | 说明                        |
| ---------------- | --- | -- | ------------------------- |
| `conn_str`       | 字符串 | 否  | 数据库路径或句柄                  |
| `script_or_path` | 字符串 | 是  | SQL 脚本文本，或 `.sql` 文件的完整路径 |

**返回值**

成功返回 "Script executed successfully"。

**示例**
```excel
' 直接执行脚本
=SqlScript(,"CREATE TABLE t1 (id INTEGER); INSERT INTO t1 VALUES (1);")

' 执行外部脚本文件
=SqlScript(,"C:\scripts\setup.sql")

' 引用单元格中的文件路径
=SqlScript(,A1)
```

**注意事项**
- 若 script_or_path 对应的路径存在，则按文件读取；否则当作原始 SQL 执行
- 文件编码自动检测：先尝试 UTF-8，失败则 fallback 到 GB18030（兼容 GBK）
- 脚本中不要包含 SELECT 查询并期望返回结果集——查询结果会被丢弃
- execute_batch 不会自动包裹事务，如需原子性，建议先 =SqlBegin()，再执行脚本，再 =SqlCommit()

#### [<ins>返回目录</ins>](#目录)
---

## 事务控制类

### SqlBegin

开始一个数据库事务。

**语法**
```excel
=SqlBegin([conn_str])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄，省略使用内存数据库 |

**返回值**

成功返回 `"Transaction started"`。

**示例**
```excel
=SqlBegin()
=SqlBegin("C:\data\app.db")
```

---

### SqlCommit

提交当前事务，使所有更改永久生效。

**语法**
```excel
=SqlCommit([conn_str])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄，省略使用内存数据库 |

**返回值**

成功返回 `"Transaction committed"`。

**示例**
```excel
=SqlCommit()
=SqlCommit("C:\data\app.db")
```

---

### SqlRollback

回滚当前事务，撤销所有未提交的更改。

**语法**
```excel
=SqlRollback([conn_str])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄，省略使用内存数据库 |

**返回值**

成功返回 `"Transaction rolled back"`。

**示例**
```excel
=SqlRollback()
=SqlRollback("C:\data\app.db")
```

**注意事项**
- 事务与连接绑定，同一连接上的所有操作在事务提交前不会持久化
- 内存数据库的事务在 Excel 进程退出后同样会丢失（如果未提交）
- 建议配合连接句柄使用，确保多步操作使用同一连接

#### [<ins>返回目录</ins>](#目录)
---

## 数据导入类

### SqlCreateTable

从 Excel 数据区域创建 SQLite 表，自动推断列名和类型。

**语法**
```excel
=SqlCreateTable([db_path], table_name, data, [columns], [types])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `db_path` | 字符串 | 否 | 目标数据库路径或句柄，省略使用内存数据库 |
| `table_name` | 字符串 | 是 | 新表名 |
| `data` | 区域 | 是 | Excel 二维数组（如 `A1:D100`） |
| `columns` | 区域/数组 | 否 | 自定义列名。省略则使用 `data` 的首行 |
| `types` | 区域/数组 | 否 | 自定义列类型。省略则自动推断 |

**返回值**

成功返回 `"Table 'xxx' created: N columns, M rows"`。

**示例**
```excel
' 使用首行作为列名，自动推断类型
=SqlCreateTable(,"employees", A1:E50)

' 指定列名，自动推断类型
=SqlCreateTable(,"products", A2:C100, {"id","name","price"})

' 完全自定义列名和类型
=SqlCreateTable(,"orders", B2:F200, {"order_id","customer","amount","date","status"}, {"INTEGER","TEXT","REAL","TEXT","TEXT"})
```

**注意事项**
- 如果表已存在，会先 **DROP** 再重建，避免重复追加
- 空单元格会被推断为 `""`（TEXT）或 `0`（INTEGER），建议清理空行
- 列名会自动去除引号、去空格；空列名自动命名为 `col_N`

#### [<ins>返回目录</ins>](#目录)
---

### SqlImportCsv

从 CSV 文件导入数据到 SQLite 表，自动识别编码和列类型。

**语法**
```excel
=SqlImportCsv([conn_str], csv_path, table_name, [has_header], [delimiter], [columns], [types])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 目标数据库路径或句柄 |
| `csv_path` | 字符串 | 是 | CSV 文件的完整路径 |
| `table_name` | 字符串 | 是 | 新表名 |
| `has_header` | 布尔 | 否 | 首行是否为列名，默认 `TRUE` |
| `delimiter` | 字符串 | 否 | 分隔符，默认逗号 `,` |
| `columns` | 区域/数组 | 否 | 自定义列名 |
| `types` | 区域/数组 | 否 | 自定义列类型 |

**返回值**

成功返回 `"Table 'xxx' created from CSV: N columns, M rows"`。

**示例**
```excel
' 默认：逗号分隔，首行为列名
=SqlImportCsv(,"C:\data\sales.csv", "sales")

' 制表符分隔，无表头
=SqlImportCsv(,"C:\data\data.tsv", "raw_data", FALSE, "\t")

' 自定义列名和类型
=SqlImportCsv(,"C:\data\file.csv", "imported", TRUE, ",", {"id","value"}, {"INTEGER","REAL"})
```

**注意事项**
- 编码自动识别：先尝试 UTF-8，失败则 fallback 到 GB18030（兼容 GBK）
- 如果 CSV 包含 BOM（UTF-8 签名），会自动处理
- 大文件使用事务批量插入，性能优于逐条 INSERT
- 空行会被跳过，但格式错误的行可能导致 `#VALUE!`

#### [<ins>返回目录</ins>](#目录)
---

### SqlImportCsvDir

批量导入指定目录下的所有 `.csv` 文件。每个 CSV 自动创建为独立的表，表名取自文件名（去除扩展名并清洗为合法 SQL 标识符）。

**语法**
```excel
=SqlImportCsvDir([conn_str], dir_path, [has_header], [delimiter], [columns], [types])
```
**参数**

| 参数           | 类型  | 必填 | 说明                   |
| ------------ | --- | -- | -------------------- |
| `conn_str`   | 字符串 | 否  | 数据库路径或句柄             |
| `dir_path`   | 字符串 | 是  | 包含 CSV 文件的目录完整路径     |
| `has_header` | 布尔  | 否  | CSV 是否含表头行，默认 `TRUE` |
| `delimiter`  | 字符串 | 否  | 分隔符，默认逗号 `,`         |
| `columns`    | 数组  | 否  | 统一指定所有文件的列名（可选）      |
| `types`      | 数组  | 否  | 统一指定所有文件的列类型（可选）     |

**返回值**

成功返回导入摘要，包含每个文件的导入结果和可能的错误列表。

**示例**
```excel
' 导入 C:\data\ 下所有 CSV，自动建表
=SqlImportCsvDir(,"C:\data\", TRUE, ",")

' 使用分号分隔符导入
=SqlImportCsvDir(,"C:\reports\", TRUE, ";")
```

**注意事项**
- 只处理扩展名为 .csv（大小写不敏感）的文件
- 非 CSV 文件（如 .txt、.xlsx）会被自动跳过
- 表名清洗规则：非字母数字字符替换为下划线；数字开头自动加 t_ 前缀
- 每个文件独立调用 sqlimportcsv_impl，因此编码（UTF-8/GBK）是逐文件自动检测的
- 如果 columns/types 被指定，会应用到所有文件；如果各文件结构不同，建议分批导入或使用 SqlImportCsv 单文件导入

#### [<ins>返回目录</ins>](#目录)
---

### SqlAppendTable

将 Excel 数据区域追加到已有表中。

**语法**
```excel
=SqlAppendTable([db_path], table_name, data)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `db_path` | 字符串 | 否 | 数据库路径或句柄 |
| `table_name` | 字符串 | 是 | 要追加数据的已有表名 |
| `data` | 区域 | 是 | Excel 二维数组（不含表头） |

**返回值**

成功返回 `"Table 'xxx': N rows appended"`。

**示例**
```excel
' 将 A101:D200 的数据追加到 orders 表
=SqlAppendTable(,"orders", A101:D200)
```

**注意事项**
- 目标表必须已存在，否则返回 `#VALUE!`
- 数据列数必须与目标表列数完全一致
- 空单元格会被当作空字符串 `""` 插入
- 使用事务批量插入，性能优于逐条执行 INSERT

#### [<ins>返回目录</ins>](#目录)
---

## 数据导出类

### SqlExportCsv

执行查询并将结果导出为 CSV 文件。

**语法**
```excel
=SqlExportCsv([conn_str], sql, csv_path, [delimiter])
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `sql` | 字符串 | 是 | SELECT 语句 |
| `csv_path` | 字符串 | 是 | 输出 CSV 文件的完整路径 |
| `delimiter` | 字符串 | 否 | 分隔符，默认逗号 `,` |

**返回值**

成功返回 `"Exported N rows to 'path'"`。

**示例**
```excel
=SqlExportCsv(,"SELECT * FROM sales", "C:\data\report.csv")
=SqlExportCsv(,"SELECT * FROM sales", "C:\data\report.tsv", "\t")
```

**注意事项**
- 输出文件路径的父目录必须存在
- 如果文件已存在，会被覆盖
- BLOB 字段以十六进制字符串形式导出
- 编码固定为 UTF-8（如需 GBK，请先用 `SqlQuery` 获取结果，再通过其他工具转换）

#### [<ins>返回目录</ins>](#目录)
---

## 元数据类

### SqlTables

列出数据库中的所有表。

**语法**
```excel
=SqlTables([conn_str])
```

**返回值**

单列数组，每个元素为一个表名。

**示例**
```excel
=SqlTables()                   ' 内存数据库中的表
=SqlTables("C:\data\app.db")   ' 指定数据库中的表
```

#### [<ins>返回目录</ins>](#目录)
---

### SqlVersion

返回 SQLite 引擎版本号。

**语法**
```excel
=SqlVersion([conn_str])
```

**返回值**

版本号字符串（如 `"3.45.1"`）。

**示例**
```excel
=SqlVersion()
```

#### [<ins>返回目录</ins>](#目录)
---

### SqlSchema

返回指定表的列结构信息（PRAGMA table_info）。

**语法**
```excel
=SqlSchema([conn_str], table_name)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `table_name` | 字符串 | 是 | 要查看结构的表名 |

**返回值**

二维数组，包含列：cid, name, type, notnull, dflt_value, pk

**示例**
```excel
=SqlSchema(,"users")
=SqlSchema("C:\data\app.db", "orders")
```

#### [<ins>返回目录</ins>](#目录)
---

### SqlPragma

执行 PRAGMA 语句并返回结果。

**语法**
```excel
=SqlPragma([conn_str], pragma_name)
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `conn_str` | 字符串 | 否 | 数据库路径或句柄 |
| `pragma_name` | 字符串 | 是 | PRAGMA 名称及参数 |

**返回值**

PRAGMA 查询结果，格式取决于具体 PRAGMA。

**示例**
```excel 
=SqlPragma(,"journal_mode")                    ' 返回当前日志模式
=SqlPragma(,"table_info(users)")               ' 返回 users 表结构
=SqlPragma("C:\data\app.db","foreign_keys")    ' 检查外键约束是否启用
```

**注意事项**
- 部分 PRAGMA 只返回标量值，部分返回表格数据
- 修改数据库状态的 PRAGMA（如 `journal_mode = WAL`）需要写权限

#### [<ins>返回目录</ins>](#目录)
