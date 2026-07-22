#!/usr/bin/env python3
"""
normalize_and_diff.py

Compares rendered SSRS output (CSV or XML) against a known-good dataset
(also CSV - e.g. exported from a SqlDataReader/DataTable), after normalizing
both sides so formatting differences don't masquerade as data bugs.

Usage:
    python normalize_and_diff.py --rendered rendered.csv --expected expected.csv \
        --key-columns OrderID,LineNumber \
        [--epsilon 0.01] [--null-equals-empty] [--output report.json]

Design notes (see SKILL.md Step 3 for rationale):
    - Numbers are parsed and compared numerically, not as strings.
    - Dates are parsed and compared as datetimes.
    - Row order is ignored by default - rows are matched by --key-columns.
      If no key columns are given, falls back to positional comparison
      (only safe if both sides are already identically sorted).
    - NULL/empty string equivalence is configurable, since SSRS often renders
      NULL as empty string.
"""

import argparse
import csv
import json
import sys
from datetime import datetime
from decimal import Decimal, InvalidOperation


def try_parse_number(value):
    if value is None:
        return None
    s = str(value).strip().replace(",", "")
    if s == "":
        return None
    try:
        return Decimal(s)
    except InvalidOperation:
        return None


DATE_FORMATS = [
    "%Y-%m-%d",
    "%Y-%m-%d %H:%M:%S",
    "%m/%d/%Y",
    "%m/%d/%Y %H:%M:%S",
    "%d/%m/%Y",
    "%Y-%m-%dT%H:%M:%S",
]


def try_parse_date(value):
    if value is None:
        return None
    s = str(value).strip()
    if s == "":
        return None
    for fmt in DATE_FORMATS:
        try:
            return datetime.strptime(s, fmt)
        except ValueError:
            continue
    return None


def normalize_value(value, null_equals_empty):
    """Return a comparable form of a single cell value."""
    if value is None:
        value = ""
    s = str(value).strip()

    if null_equals_empty and s.upper() in ("", "NULL"):
        return None

    num = try_parse_number(s)
    if num is not None:
        return num

    date = try_parse_date(s)
    if date is not None:
        return date

    return s


def load_csv(path):
    with open(path, newline="", encoding="utf-8-sig") as f:
        reader = csv.DictReader(f)
        return [dict(row) for row in reader], reader.fieldnames


def normalize_row(row, null_equals_empty):
    return {k: normalize_value(v, null_equals_empty) for k, v in row.items()}


def row_key(row, key_columns):
    return tuple(str(row.get(c, "")).strip() for c in key_columns)


def values_equal(a, b, epsilon):
    if isinstance(a, Decimal) and isinstance(b, Decimal):
        return abs(a - b) <= Decimal(str(epsilon))
    return a == b


def diff(rendered_rows, expected_rows, key_columns, epsilon, null_equals_empty):
    discrepancies = []

    rendered_norm = [normalize_row(r, null_equals_empty) for r in rendered_rows]
    expected_norm = [normalize_row(r, null_equals_empty) for r in expected_rows]

    if key_columns:
        rendered_map = {row_key(r, key_columns): r for r in rendered_norm}
        expected_map = {row_key(r, key_columns): r for r in expected_norm}

        missing_in_rendered = set(expected_map) - set(rendered_map)
        missing_in_expected = set(rendered_map) - set(expected_map)

        for key in sorted(missing_in_rendered):
            discrepancies.append({
                "type": "missing_row_in_rendered",
                "key": dict(zip(key_columns, key)),
            })
        for key in sorted(missing_in_expected):
            discrepancies.append({
                "type": "unexpected_row_in_rendered",
                "key": dict(zip(key_columns, key)),
            })

        common_keys = set(rendered_map) & set(expected_map)
        for key in sorted(common_keys):
            r_row = rendered_map[key]
            e_row = expected_map[key]
            all_cols = set(r_row.keys()) | set(e_row.keys())
            for col in sorted(all_cols):
                r_val = r_row.get(col)
                e_val = e_row.get(col)
                if not values_equal(r_val, e_val, epsilon):
                    discrepancies.append({
                        "type": "value_mismatch",
                        "key": dict(zip(key_columns, key)),
                        "column": col,
                        "expected": str(e_val),
                        "actual": str(r_val),
                    })
    else:
        # Positional fallback - only valid if both sides are pre-sorted identically.
        if len(rendered_norm) != len(expected_norm):
            discrepancies.append({
                "type": "row_count_mismatch",
                "expected_count": len(expected_norm),
                "actual_count": len(rendered_norm),
            })
        for i, (r_row, e_row) in enumerate(zip(rendered_norm, expected_norm)):
            all_cols = set(r_row.keys()) | set(e_row.keys())
            for col in sorted(all_cols):
                r_val = r_row.get(col)
                e_val = e_row.get(col)
                if not values_equal(r_val, e_val, epsilon):
                    discrepancies.append({
                        "type": "value_mismatch",
                        "row_index": i,
                        "column": col,
                        "expected": str(e_val),
                        "actual": str(r_val),
                    })

    return discrepancies


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--rendered", required=True, help="Path to rendered SSRS output (CSV)")
    parser.add_argument("--expected", required=True, help="Path to known-good CSV (e.g. exported query results)")
    parser.add_argument("--key-columns", default="", help="Comma-separated column names to match rows by. Omit for positional comparison (requires identical pre-sorted order on both sides).")
    parser.add_argument("--epsilon", type=float, default=0.0, help="Numeric tolerance for value comparison (default: exact)")
    parser.add_argument("--null-equals-empty", action="store_true", help="Treat NULL and empty string as equal")
    parser.add_argument("--output", default=None, help="Write JSON discrepancy report to this path (default: stdout)")
    args = parser.parse_args()

    key_columns = [c.strip() for c in args.key_columns.split(",") if c.strip()]

    rendered_rows, rendered_fields = load_csv(args.rendered)
    expected_rows, expected_fields = load_csv(args.expected)

    discrepancies = diff(rendered_rows, expected_rows, key_columns, args.epsilon, args.null_equals_empty)

    report = {
        "rendered_file": args.rendered,
        "expected_file": args.expected,
        "key_columns": key_columns or None,
        "comparison_mode": "key-matched" if key_columns else "positional",
        "rendered_row_count": len(rendered_rows),
        "expected_row_count": len(expected_rows),
        "discrepancy_count": len(discrepancies),
        "discrepancies": discrepancies,
        "passed": len(discrepancies) == 0,
    }

    output_json = json.dumps(report, indent=2, default=str)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(output_json)
        print(f"Report written to {args.output}", file=sys.stderr)
        print(f"Result: {'PASS' if report['passed'] else 'FAIL'} ({len(discrepancies)} discrepancies)", file=sys.stderr)
    else:
        print(output_json)

    sys.exit(0 if report["passed"] else 1)


if __name__ == "__main__":
    main()
