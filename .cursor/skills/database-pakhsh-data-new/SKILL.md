---
name: pakhsh-database
description: >-
  Connect to and query the Pakhsh SQL Server database (Pakhsh_Data_New on
  10.10.12.52). Use when the user asks to run SQL, inspect schema, validate data,
  debug stored procedures, compare DB results with app behavior, or investigate
  database-related issues in the DPDC/Pakhsh project.
---

# Pakhsh Database Subagent

Specialized subagent for read-focused work against the shared Pakhsh SQL Server database.

## Connection

| Setting | Value |
|---------|-------|
| Server | `10.10.12.52` |
| Database | `Pakhsh_Data_New` |
| Username | `public01` |
| Password | `Public01Public01` |
| Driver | `ODBC Driver 17 for SQL Server` |

Credentials resolve in this order:

1. `PAKHSH_DATA_NEW_DB_SQL_SERVER_CONNECTION_STRING` (ADO.NET style; preferred)
2. `PAKHSH_DB_SERVER`, `PAKHSH_DB_NAME`, `PAKHSH_DB_USER`, `PAKHSH_DB_PASSWORD`
3. Defaults above

Connection string (pyodbc):

```
DRIVER={ODBC Driver 17 for SQL Server};SERVER=10.10.12.52;DATABASE=Pakhsh_Data_New;UID=public01;PWD=Public01Public01;Encrypt=no;TrustServerCertificate=yes;Connect Timeout=60;
```

Override via environment variables when needed.

## How to Connect

1. Prefer the utility scripts in `scripts/` (reliable, consistent).
2. For ad-hoc work, use `python` with `pyodbc` and the connection string above.
3. Reuse patterns from `C:\Users\a.dashti\TFS\Source\_test_exports\` when they match the task.

**Test connectivity:**

```bash
python "%USERPROFILE%\.cursor\skills\pakhsh-database\scripts\query.py" "SELECT TOP 1 DB_NAME() AS db"
```



## Utility Scripts

Run from any directory; scripts resolve their own connection.

| Script | Purpose |
|--------|---------|
| `scripts/query.py` | Run a single SQL statement and print rows |
| `scripts/list_schemas.py` | List database schemas |
| `scripts/list_tables.py` | List tables (optional schema filter) |
| `scripts/proc_info.py` | Parameters and definition snippet for a stored procedure |

### Examples

```bash
# Single query
python "%USERPROFILE%\.cursor\skills\pakhsh-database\scripts\query.py" "SELECT TOP 5 name FROM sys.tables ORDER BY create_date DESC"

# List schemas
python "%USERPROFILE%\.cursor\skills\pakhsh-database\scripts\list_schemas.py"

# Tables in Sales schema
python "%USERPROFILE%\.cursor\skills\pakhsh-database\scripts\list_tables.py" Sales

# Stored procedure metadata
python "%USERPROFILE%\.cursor\skills\pakhsh-database\scripts\proc_info.py" Sales rpt_ForoshBarHasbeModatVosul
```

`query.py` options:

```bash
python scripts/query.py --file path/to/query.sql
python scripts/query.py --max-rows 50 "SELECT TOP 100 ..."
```

## Common Schemas

The app uses schema-qualified names. Frequent schemas:

| Schema | Domain |
|--------|--------|
| `Global` | Users (`Afrad`), locations, shared master data |
| `Sales` | Sales orders, reports, IMED |
| `Purchase` | Suppliers (`TaminKonandeh`) |
| `WareHouse` | Inventory, kardex, sefaresh |
| `FinancialAccounting` | Accounts, sanad, tafsily |
| `AssetAccounting` | Fixed assets (amval) |
| `Budget` | Budget / sorat maly |
| `dbo` | System tables, permissions |

For schema exploration details, see [reference.md](reference.md).

## Workflow

When invoked as a database subagent:

1. Clarify the question (table, proc, report, or business entity).
2. Find the matching schema/table from codebase `*_DB.vb` files or `list_tables.py`.
3. Run a **small bounded** query to confirm shape and sample data.
4. Expand only if needed; keep each query narrow.
5. Report findings with table/column names and row counts — not full dumps.

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `IM002` / driver not found | Install [ODBC Driver 17 for SQL Server](https://learn.microsoft.com/en-us/sql/connect/odbc/download-odbc-driver-for-sql-server) |
| Login timeout | Check VPN/network to `10.10.12.52` |
| `pyodbc` missing | `pip install pyodbc` |
| Permission denied on object | User may lack rights; try a view (`v*` prefix) or ask user |

## Related Project Paths

- App connection: `Web.config` → `AppConStr`
- Existing test scripts: `C:\Users\a.dashti\TFS\Source\_test_exports\`
- SQL scripts: `C:\Users\a.dashti\TFS\SQL`
- Workspace rules: `.cursor/rules/small-test-data.mdc`, `.cursor/rules/newest-test-data.mdc`
