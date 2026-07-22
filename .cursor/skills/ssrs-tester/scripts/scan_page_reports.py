#!/usr/bin/env python3
"""Scan page-linked RDL files for a column header text."""
import argparse
import os
import re
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--vb-file", required=True)
    parser.add_argument("--rdl-dir", required=True)
    parser.add_argument("--needle", required=True)
    args = parser.parse_args()

    vb = open(args.vb_file, encoding="utf-8", errors="replace").read()
    reports = sorted(set(re.findall(r"ReportPath=/PakhshReports/([^&\"]+)", vb)))
    matches = []
    for report in reports:
        path = os.path.join(args.rdl_dir, report + ".rdl")
        if not os.path.exists(path):
            matches.append({"report": report, "status": "RDL_NOT_FOUND"})
            continue
        txt = open(path, encoding="utf-8", errors="replace").read()
        if args.needle in txt:
            hits = re.findall(
                r"<Value>([^<]*" + re.escape(args.needle) + r"[^<]*)</Value>", txt
            )
            matches.append(
                {"report": report, "status": "HAS_COLUMN", "headers": hits[:5]}
            )

    print(f"Total reports in page: {len(reports)}")
    has = [m for m in matches if m["status"] == "HAS_COLUMN"]
    print(f"Reports with column '{args.needle}': {len(has)}")
    for m in has:
        print(f"--- {m['report']}")
        for h in m["headers"]:
            print(f"  {h}")
    missing = [m for m in matches if m["status"] == "RDL_NOT_FOUND"]
    if missing:
        print("Missing RDL:")
        for m in missing:
            print(f"  {m['report']}")
    return 0 if has else 1


if __name__ == "__main__":
    sys.exit(main())
