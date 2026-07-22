#!/usr/bin/env python3
"""Show parameters and definition head for a stored procedure."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from connection import connect  # noqa: E402

if len(sys.argv) < 3:
    print("Usage: proc_info.py <schema> <procedure_name>")
    raise SystemExit(1)

schema, proc_name = sys.argv[1], sys.argv[2]

with connect() as conn:
    cur = conn.cursor()

    cur.execute(
        """
        SELECT p.name, SCHEMA_NAME(p.schema_id), p.create_date, p.modify_date
        FROM sys.procedures p
        WHERE SCHEMA_NAME(p.schema_id) = ? AND p.name = ?
        """,
        (schema, proc_name),
    )
    row = cur.fetchone()
    if not row:
        print(f"Not found: {schema}.{proc_name}")
        raise SystemExit(1)
    print(f"Procedure: {schema}.{proc_name}")
    print(f"Created: {row[2]}  Modified: {row[3]}")
    print()

    cur.execute(
        """
        SELECT
            p.name,
            t.name AS type_name,
            p.max_length,
            p.is_output
        FROM sys.parameters p
        JOIN sys.types t ON p.user_type_id = t.user_type_id
        WHERE p.object_id = OBJECT_ID(?)
        ORDER BY p.parameter_id
        """,
        (f"{schema}.{proc_name}",),
    )
    params = cur.fetchall()
    print("Parameters:")
    if not params:
        print("  (none)")
    else:
        for name, type_name, max_len, is_output in params:
            direction = "OUT" if is_output else "IN"
            print(f"  {name} {type_name}({max_len}) [{direction}]")

    cur.execute(
        """
        SELECT TOP 1 LEFT(OBJECT_DEFINITION(OBJECT_ID(?)), 4000)
        """,
        (f"{schema}.{proc_name}",),
    )
    definition = cur.fetchone()[0]
    print()
    print("Definition (first 4000 chars):")
    print("-" * 60)
    print(definition or "(unavailable)")
