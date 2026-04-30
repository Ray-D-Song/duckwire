# DuckWire

PostgreSQL wire protocol server for DuckDB. Connect any PostgreSQL-compatible client (psql, Navicat, DBeaver, TablePlus, JDBC drivers) to a DuckDB instance — in-memory or persistent.

## How It Works

DuckWire speaks the PostgreSQL wire protocol (v3.0) and translates client messages into DuckDB queries while emulating `pg_catalog` system tables.

```
┌────────────┐   PG Protocol   ┌──────────────┐   DuckDB C API   ┌────────┐
│  Client    │ ◄──────────────► │  DuckWire    │ ◄──────────────► │ DuckDB │
│ (psql/etc) │     TCP 5433    │  (Rust)      │                  │        │
└────────────┘                  └──────────────┘                  └────────┘
```

## Quick Start

```bash
cargo install duckwire  # or download a pre-built binary from releases

# In-memory database, default port 5433
duckwire

# Persistent database
duckwire --db mydata.db

# Custom port and host
duckwire --port 5439 --host 127.0.0.1
```

Connect with any PostgreSQL client:

```bash
psql -h 127.0.0.1 -p 5433
```

## Docker

Pre-built images are published to [GitHub Container Registry](https://github.com/Ray-D-Song/duckwire/pkgs/container/duckwire).

```bash
# In-memory
docker run -p 5433:5433 ghcr.io/ray-d-song/duckwire:latest

# Persistent database (mount a volume)
docker run -p 5433:5433 -v ./data:/data ghcr.io/ray-d-song/duckwire:latest --db /data/mydata.db

# Custom port
docker run -p 9999:5433 ghcr.io/ray-d-song/duckwire:latest --port 9999
```

## Configuration File

DuckWire supports TOML configuration. CLI flags override config values.

```toml
# duckwire.toml
db = "mydata.db"
port = 5432
host = "127.0.0.1"
logfile = "/var/log/duckwire"
```

```bash
duckwire --config duckwire.toml
duckwire -c duckwire.toml --port 9999   # CLI overrides config port
```

**Priority**: CLI flags > config file > built-in defaults (port 5433, host 0.0.0.0).

## CLI Options

| Flag | Description |
|------|-------------|
| `-c, --config <PATH>` | TOML config file |
| `-d, --db <PATH>` | DuckDB file path (default: in-memory) |
| `-p, --port <PORT>` | Listen port (default: 5433) |
| `-H, --host <ADDR>` | Listen address (default: 0.0.0.0) |
| `-l, --logfile <PATH>` | Log file path; if a directory, writes `duckwire.log` inside |

## Logging

## PostgreSQL Compatibility

### pg_catalog Views (emulated)

`pg_type`, `pg_class`, `pg_attribute`, `pg_proc`, `pg_namespace`, `pg_index`, `pg_constraint`, `pg_roles`, `pg_database`, `pg_tables`, `pg_views`, `pg_matviews`, `pg_settings`, `pg_locks`, `pg_stat_activity`, `pg_extension`, `pg_enum`, `pg_collation`, `pg_language`, `pg_conversion`, `pg_tablespace`, `pg_inherits`, `pg_depend`, `pg_shdepend`, `pg_description`, `pg_shdescription`, `pg_attrdef`, `pg_authid`, `pg_auth_members`, `pg_user`, `pg_shadow`, `pg_policies`, `pg_foreign_server`, `pg_foreign_table`, `pg_stat_user_tables`

### information_schema

`information_schema.tables`, `information_schema.columns`, `information_schema.routines`, `information_schema.parameters` are transparently rewritten to DuckDB equivalents with added PostgreSQL-compatible columns (e.g. `udt_name`).

### SQL Transpilation

Incoming PostgreSQL SQL is transpiled to DuckDB SQL:
- `pg_catalog.X` → `pg_compat.X`
- `::regclass` → removed (cast lookup is silently dropped)
- `current_setting('...')` → emulated setting values
- `SHOW` commands → `SELECT current_setting(...)` equivalents
- `string_agg(..., ',' ORDER BY ...)` → DuckDB-compatible syntax
- Array literals (`'{1,2,3}'`) → list syntax (`[1, 2, 3]`)
- `polyglot-sql` as fallback for remaining PostgreSQL→DuckDB conversions

### Protocols

- **Simple Query**: full `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE TABLE`, `SET`, `SHOW`
- **Extended Query**: parameterized statements with `$1`, `$2`, etc. (prepared statement protocol)
- **Transactions**: `BEGIN`, `COMMIT`, `ROLLBACK`
- **Type coercion**: timestamps, numerics, and other PG types mapped to DuckDB Arrow types and back

## Supported Clients

Tested with: **psql**, **DBeaver**, **Navicat**.

## License

MIT
