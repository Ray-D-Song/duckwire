# DuckWire

DuckDB 的 PostgreSQL 有线协议服务器。连接任何兼容 PostgreSQL 的客户端（psql、Navicat、DBeaver、TablePlus、JDBC 驱动）到 DuckDB 实例（内存或持久化均可）。

## 工作原理

DuckWire 通过 TCP 以 PostgreSQL 有线协议（v3.0）与客户端通信，在后台将客户端消息翻译为 DuckDB 查询并模拟 `pg_catalog` 系统表。

```
┌────────────┐    PG 协议      ┌──────────────┐   DuckDB C API   ┌────────┐
│  客户端     │ ◄──────────────► │  DuckWire    │ ◄──────────────► │ DuckDB │
│ (psql 等)  │     TCP 5433    │  (Rust)      │                  │        │
└────────────┘                  └──────────────┘                  └────────┘
```

## 快速开始

```bash
cargo install duckwire # 或下载 release 中的预编译文件

# 内存数据库，默认端口 5433
duckwire

# 持久化数据库
duckwire --db mydata.db

# 自定义端口和地址
duckwire --port 5439 --host 127.0.0.1
```

使用任意 PostgreSQL 客户端连接：

```bash
psql -h 127.0.0.1 -p 5433
```

## Docker

预构建镜像发布在 [GitHub Container Registry](https://github.com/Ray-D-Song/duckwire/pkgs/container/duckwire)。

```bash
# 内存数据库
docker run -p 5433:5433 ghcr.io/ray-d-song/duckwire:latest

# 持久化数据库（挂载数据卷）
docker run -p 5433:5433 -v ./data:/data ghcr.io/ray-d-song/duckwire:latest --db /data/mydata.db

# 自定义端口
docker run -p 9999:5433 ghcr.io/ray-d-song/duckwire:latest --port 9999
```

## 配置文件

DuckWire 支持 TOML 配置文件，CLI 参数优先级高于配置文件。

```toml
# duckwire.toml
db = "mydata.db"
port = 5432
host = "127.0.0.1"
logfile = "/var/log/duckwire"
```

```bash
duckwire --config duckwire.toml
duckwire -c duckwire.toml --port 9999   # CLI 覆盖配置中的 port
```

**优先级**：CLI 参数 > 配置文件 > 内置默认值（port 5433, host 0.0.0.0）。

## CLI 选项

| 选项 | 说明 |
|------|------|
| `-c, --config <PATH>` | TOML 配置文件路径 |
| `-d, --db <PATH>` | DuckDB 数据库文件路径（默认：内存） |
| `-p, --port <PORT>` | 监听端口（默认：5433） |
| `-H, --host <ADDR>` | 监听地址（默认：0.0.0.0） |
| `-l, --logfile <PATH>` | 日志文件路径；如传目录则在其中写入 `duckwire.log` |

## 日志

默认日志输出到 stderr，级别为 `debug`。通过 `RUST_LOG` 调整：

```bash
RUST_LOG=duckwire=trace duckwire
```

## PostgreSQL 兼容性

### pg_catalog 视图（模拟）

`pg_type`、`pg_class`、`pg_attribute`、`pg_proc`、`pg_namespace`、`pg_index`、`pg_constraint`、`pg_roles`、`pg_database`、`pg_tables`、`pg_views`、`pg_matviews`、`pg_settings`、`pg_locks`、`pg_stat_activity`、`pg_extension`、`pg_enum`、`pg_collation`、`pg_language`、`pg_conversion`、`pg_tablespace`、`pg_inherits`、`pg_depend`、`pg_shdepend`、`pg_description`、`pg_shdescription`、`pg_attrdef`、`pg_authid`、`pg_auth_members`、`pg_user`、`pg_shadow`、`pg_policies`、`pg_foreign_server`、`pg_foreign_table`、`pg_stat_user_tables`

### information_schema

`information_schema.tables`、`information_schema.columns`、`information_schema.routines`、`information_schema.parameters` 会透明地重写为 DuckDB 等价物，并添加 PostgreSQL 兼容的列（如 `udt_name`）。

### SQL 转换

输入的 PostgreSQL SQL 会被转译为 DuckDB SQL：
- `pg_catalog.X` → `pg_compat.X`
- `::regclass` → 移除（类型查找直接丢弃）
- `current_setting('...')` → 模拟的设置值
- `SHOW` 命令 → `SELECT current_setting(...)` 等价形式
- `string_agg(..., ',' ORDER BY ...)` → DuckDB 兼容语法
- 数组字面量 (`'{1,2,3}'`) → 列表语法 (`[1, 2, 3]`)
- `polyglot-sql` 作为剩余 PostgreSQL→DuckDB 转换的后备

### 协议支持

- **简单查询**：完整的 `SELECT`、`INSERT`、`UPDATE`、`DELETE`、`CREATE TABLE`、`SET`、`SHOW`
- **扩展查询**：支持 `$1`、`$2` 等参数化语句（预编译语句协议）
- **事务**：`BEGIN`、`COMMIT`、`ROLLBACK`
- **类型转换**：时间戳、数值等 PG 类型与 DuckDB Arrow 类型之间的双向映射

## 支持的客户端

已测试：**psql**、**DBeaver**、**Navicat**。

## 协议

MIT
