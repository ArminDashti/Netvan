#!/usr/bin/env python3
"""Fast validation runner using urllib + pyodbc (no PowerShell subprocess)."""
from __future__ import annotations

import json
import sys
import urllib.parse
import urllib.request
from decimal import Decimal
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from validate_jayeze_reports import (  # noqa: E402
    COLUMN_HEADERS,
    build_render_params,
    csv_rows_with_column,
    db_sum,
    execute_dataset,
    get_test_context,
    page_reports,
    parse_rdl,
    sum_decimal,
)

REPORT_SERVER = "http://localhost/ReportServer"
VB_FILE = Path(
    r"C:\Users\a.dashti\TFS\Source\Pages\Sales\04-Gozareshat\KharidMoshtarianNesbatBeKolLink.aspx.vb"
)
RDL_DIR = Path(r"D:\repos\PakhshReports\PakhshReports")
OUTPUT = SCRIPT_DIR / "jayeze_validation_results.json"


def render_csv_fast(report_path: str, params: dict[str, str]) -> str:
    import os
    import subprocess
    import tempfile

    encoded_path = urllib.parse.quote(report_path, safe="")
    query = [encoded_path, "rs:Command=Render", "rs:Format=CSV"]
    for key, value in sorted(params.items()):
        query.append(
            f"{urllib.parse.quote(key)}={urllib.parse.quote(str(value))}"
        )
    url = REPORT_SERVER + "?" + "&".join(query)
    out_file = tempfile.mktemp(suffix=".csv")
    ps_script = (
        "$ErrorActionPreference = 'Stop'\n"
        f"$uri = '{url}'\n"
        f"$out = '{out_file}'\n"
        "$r = Invoke-WebRequest -Uri $uri -UseDefaultCredentials "
        "-AllowUnencryptedAuthentication -TimeoutSec 180\n"
        "[IO.File]::WriteAllText($out, $r.Content, [Text.UTF8Encoding]::new($false))\n"
    )
    with tempfile.NamedTemporaryFile(
        "w", suffix=".ps1", delete=False, encoding="utf-8"
    ) as fh:
        fh.write(ps_script)
        script_path = fh.name
    try:
        proc = subprocess.run(
            [
                "pwsh",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_path,
            ],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    finally:
        os.unlink(script_path)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    try:
        return Path(out_file).read_text(encoding="utf-8-sig", errors="replace")
    finally:
        if os.path.exists(out_file):
            os.unlink(out_file)


def execute_dataset_all(
    dataset: dict, render_params: dict[str, str], max_rows: int = 50000
) -> list[dict]:
    rows = execute_dataset(dataset, render_params)
    if len(rows) < 500:
        return rows

    import pyodbc

    sql_params: list[tuple[str, object]] = []
    expr_map = {
        "=Parameters!AzTarikh.Value": render_params.get("AzTarikh", ""),
        "=Parameters!TaTarikh.Value": render_params.get("TaTarikh", ""),
        "=Parameters!AzTarikhGozaresh.Value": render_params.get("AzTarikhGozaresh", ""),
        "=Parameters!TaTarikhGozaresh.Value": render_params.get("TaTarikhGozaresh", ""),
        "=Parameters!ccMarkazPakhsh.Value": render_params.get("ccMarkazPakhsh", ""),
        "=Parameters!ccKalaCode.Value": render_params.get("ccKalaCode", ""),
        "=Parameters!ccMoshtary.Value": render_params.get("ccMoshtary", ""),
        "=Parameters!ccForoshandeh.Value": render_params.get("ccForoshandeh", ""),
        "=Parameters!ccTaminKonandeh.Value": render_params.get("ccTaminKonandeh", ""),
        "=Parameters!ccTaminkonandeh.Value": render_params.get(
            "ccTaminkonandeh", render_params.get("ccTaminKonandeh", "")
        ),
        "=Parameters!IsOnDpLine.Value": render_params.get("IsOnDpLine", "0"),
        "=Parameters!IsOnDpLineFaktor.Value": render_params.get(
            "IsOnDpLineFaktor", "0"
        ),
    }
    if dataset["query_params"]:
        for qp in dataset["query_params"]:
            pname = qp["name"]
            expr = qp["value_expr"].strip()
            key = pname.lstrip("@")
            value = expr_map.get(expr, render_params.get(key, ""))
            if key.lower().startswith("ison") and str(value).lower() in ("false", "true"):
                value = 1 if str(value).lower() == "true" else 0
            sql_params.append((pname, value))
    else:
        for k, v in render_params.items():
            if k.startswith("Report_"):
                continue
            if k.lower().startswith("ison") and str(v).lower() in ("false", "true"):
                v = 1 if str(v).lower() == "true" else 0
            sql_params.append((f"@{k}", v))

    conn = pyodbc.connect(
        "DRIVER={ODBC Driver 17 for SQL Server};"
        "SERVER=10.10.12.52;DATABASE=Pakhsh_Data_New;"
        "UID=public01;PWD=Public01Public01;"
    )
    cur = conn.cursor()
    if dataset["command_type"].lower() == "storedprocedure":
        placeholders = ", ".join("?" for _ in sql_params)
        sql = (
            f"{{CALL {dataset['command']} ({placeholders})}}"
            if sql_params
            else f"{{CALL {dataset['command']}}}"
        )
        cur.execute(sql, [v for _, v in sql_params])
    else:
        cur.execute(dataset["command"])
    columns = [d[0] for d in cur.description] if cur.description else []
    all_rows = []
    count = 0
    for row in cur.fetchall():
        all_rows.append({columns[i]: row[i] for i in range(len(columns))})
        count += 1
        if count >= max_rows:
            break
    conn.close()
    return all_rows


def validate_one(report: str, ctx: dict[str, str]) -> dict:
    rdl_path = RDL_DIR / f"{report}.rdl"
    info = parse_rdl(rdl_path)
    result: dict = {
        "report": report,
        "report_path": f"/PakhshReports/{report}",
        "column_header": info["headers"],
        "mode": "RDL-sourced stored proc vs SSRS CSV render",
    }

    if not info["tedad_field"]:
        result.update({"status": "SKIP", "reason": "No TedadJayezeh field in dataset"})
        return result

    main_ds = next(
        (d for d in info["datasets"] if info["tedad_field"] in d["fields"]),
        info["datasets"][0] if info["datasets"] else None,
    )
    if not main_ds:
        result.update({"status": "SKIP", "reason": "No dataset found"})
        return result

    render_params = build_render_params(info, ctx)
    result["dataset"] = main_ds["name"]
    result["proc_or_sql"] = main_ds["command"]

    try:
        csv_text = render_csv_fast(f"/PakhshReports/{report}", render_params)
    except Exception as exc:
        result.update({"status": "RENDER_ERROR", "error": str(exc)})
        return result

    _, rendered_values = csv_rows_with_column(
        csv_text, COLUMN_HEADERS, info["tedad_field"]
    )
    rendered_nums = [r["raw"] for r in rendered_values if r["raw"]]
    rendered_total = sum_decimal(rendered_nums)

    try:
        db_rows = execute_dataset_all(main_ds, render_params)
    except Exception as exc:
        result.update(
            {
                "status": "DB_ERROR",
                "rendered_total": str(rendered_total),
                "rendered_rows": len(rendered_nums),
                "error": str(exc),
            }
        )
        return result

    db_total = db_sum(db_rows, info["tedad_field"])
    result.update(
        {
            "rendered_total": str(rendered_total),
            "db_total": str(db_total),
            "rendered_rows": len(rendered_nums),
            "db_rows": len(db_rows),
        }
    )
    if rendered_total == db_total:
        result["status"] = "PASS"
    else:
        result["status"] = "MISMATCH"
        result["delta"] = str(rendered_total - db_total)
    return result


def main() -> int:
    ctx = get_test_context()
    ctx["ccTaminkonandeh"] = ""

    candidates: list[str] = []
    for report in page_reports(VB_FILE):
        rdl_path = RDL_DIR / f"{report}.rdl"
        if not rdl_path.exists():
            continue
        info = parse_rdl(rdl_path)
        if info["headers"]:
            candidates.append(report)

    results = []
    for report in candidates:
        print(f"Validating {report}...", flush=True)
        row = validate_one(report, ctx)
        results.append(row)
        print(
            json.dumps(
                {k: row[k] for k in ("report", "status", "rendered_total", "db_total", "delta", "error") if k in row},
                ensure_ascii=False,
            ),
            flush=True,
        )

    payload = {
        "test_context": ctx,
        "note": "No RDL uses exact header 'تعداد جوایز'; matched 'تعداد جایزه' / TedadJayezeh",
        "page_total_reports": len(page_reports(VB_FILE)),
        "reports_with_column_local_rdl": len(candidates),
        "results": results,
    }
    OUTPUT.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nWrote {OUTPUT}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
