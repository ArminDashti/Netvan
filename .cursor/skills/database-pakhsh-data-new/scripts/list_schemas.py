#!/usr/bin/env python3
"""List schemas in Pakhsh_Data_New."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from connection import connect  # noqa: E402

SQL = """
SELECT TOP 50 s.name AS schema_name, COUNT(t.object_id) AS table_count
FROM sys.schemas s
LEFT JOIN sys.tables t ON t.schema_id = s.schema_id
GROUP BY s.name
ORDER BY table_count DESC, s.name
"""

with connect() as conn:
    cur = conn.cursor()
    cur.execute(SQL)
    print(f"{'schema_name':<30} table_count")
    print("-" * 45)
    for name, count in cur.fetchall():
        print(f"{name:<30} {count}")
