#!/usr/bin/env python3
"""List tables in Pakhsh_Data_New, optionally filtered by schema."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from connection import connect  # noqa: E402

schema = sys.argv[1] if len(sys.argv) > 1 else None

sql = """
SELECT TOP 200
    SCHEMA_NAME(t.schema_id) AS schema_name,
    t.name AS table_name,
    t.create_date
FROM sys.tables t
"""
params: list[str] = []
if schema:
    sql += " WHERE SCHEMA_NAME(t.schema_id) = ?"
    params.append(schema)
sql += " ORDER BY t.create_date DESC, schema_name, table_name"

with connect() as conn:
    cur = conn.cursor()
    cur.execute(sql, params)
    print(f"{'schema':<25} {'table':<45} create_date")
    print("-" * 90)
    for schema_name, table_name, created in cur.fetchall():
        print(f"{schema_name:<25} {table_name:<45} {created}")
