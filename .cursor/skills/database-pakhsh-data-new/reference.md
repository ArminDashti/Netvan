# Pakhsh Database Reference

## Exploration Queries

```sql
-- Recent tables
SELECT TOP 20 SCHEMA_NAME(schema_id) AS s, name, create_date
FROM sys.tables
ORDER BY create_date DESC;

-- Find table by name fragment
SELECT TOP 20 SCHEMA_NAME(schema_id) AS s, name
FROM sys.tables
WHERE name LIKE '%Sanad%'
ORDER BY name;

-- Find stored procedures
SELECT TOP 20 SCHEMA_NAME(schema_id) AS s, name
FROM sys.procedures
WHERE name LIKE 'rpt_%'
ORDER BY name;

-- Column search
SELECT TOP 30
    SCHEMA_NAME(t.schema_id) AS schema_name,
    t.name AS table_name,
    c.name AS column_name,
    ty.name AS type_name
FROM sys.tables t
JOIN sys.columns c ON c.object_id = t.object_id
JOIN sys.types ty ON c.user_type_id = ty.user_type_id
WHERE c.name LIKE '%ccSanad%'
ORDER BY schema_name, table_name;
```

## App ↔ Database Mapping

VB data classes live under `App_Code/<Module>/` with `*_DB.vb` files containing SQL. When debugging:

1. Find the `*_DB.vb` for the entity (e.g. `Sanad_DB.vb`).
2. Copy the `FROM` / `JOIN` pattern into a bounded `SELECT TOP ...`.
3. Compare UI/export output with the same filters the proc or query uses.

## Report Server

Some validation scripts also connect to `ReportServer` on the same host. Only use when the task involves SSRS reports.

## Web.config Mirror

Production app uses failover partner `mirror-sql`. Direct scripts connect to `10.10.12.52` only — sufficient for read/debug work.
