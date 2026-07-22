#!/usr/bin/env python3
"""Run a single SQL query against Pakhsh_Data_New and print results."""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from connection import connect  # noqa: E402

WRITE_KEYWORDS = ("INSERT", "UPDATE", "DELETE", "MERGE", "TRUNCATE", "DROP", "ALTER", "CREATE")


def is_write_sql(sql: str) -> bool:
    first = sql.strip().split(None, 1)[0].upper() if sql.strip() else ""
    return first in WRITE_KEYWORDS


def format_row(columns: list[str], row) -> str:
    parts = []
    for col, val in zip(columns, row):
        parts.append(f"{col}={val!r}")
    return " | ".join(parts)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run SQL against Pakhsh_Data_New")
    parser.add_argument("sql", nargs="?", help="SQL statement to execute")
    parser.add_argument("--file", "-f", help="Read SQL from file")
    parser.add_argument(
        "--allow-write",
        action="store_true",
        help="Allow INSERT/UPDATE/DELETE (use only when user explicitly requested)",
    )
    parser.add_argument(
        "--max-rows",
        type=int,
        default=100,
        help="Max rows to print for result sets (default: 100)",
    )
    args = parser.parse_args()

    if args.file:
        sql = Path(args.file).read_text(encoding="utf-8")
    elif args.sql:
        sql = args.sql
    else:
        parser.error("Provide SQL as an argument or via --file")

    if is_write_sql(sql) and not args.allow_write:
        print("Refusing write statement. Pass --allow-write only if the user explicitly requested it.", file=sys.stderr)
        return 2

    with connect() as conn:
        cur = conn.cursor()
        cur.execute(sql)

        if cur.description is None:
            conn.commit()
            print(f"OK — {cur.rowcount} row(s) affected.")
            return 0

        columns = [col[0] for col in cur.description]
        print("\t".join(columns))
        print("-" * min(120, max(40, len("\t".join(columns)))))
        count = 0
        for row in cur.fetchmany(args.max_rows):
            print("\t".join("" if v is None else str(v) for v in row))
            count += 1
        if count == args.max_rows:
            print(f"... (truncated at {args.max_rows} rows)")
        else:
            print(f"({count} row(s))")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
