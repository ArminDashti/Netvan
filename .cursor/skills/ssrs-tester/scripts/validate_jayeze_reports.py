#!/usr/bin/env python3
"""Validate page-linked SSRS reports that contain TedadJayezeh column."""
from __future__ import annotations

import argparse
import csv
import io
import json
import os
import re
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request
from decimal import Decimal
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET

SCRIPT_DIR = Path(__file__).resolve().parent
SKILL_DIR = SCRIPT_DIR.parent
REPO_ROOT = SKILL_DIR.parents[2]
DB_QUERY = REPO_ROOT / ".cursor" / "skills" / "pakhsh-database" / "scripts" / "query.py"
DIFF_SCRIPT = SCRIPT_DIR / "normalize_and_diff.py"

REPORT_SERVER = "http://localhost/ReportServer"
CONNECTION_STRING = (
    "Server=10.10.12.52;Database=Pakhsh_Data_New;"
    "User Id=public01;Password=Public01Public01;"
)
COLUMN_HEADERS = ("تعداد جوایز", "تعداد جایزه")
DEFAULT_AZ = "14040325"
DEFAULT_TA = "14040328"
DEFAULT_MARKAZ = "34"


def run_query(sql: str) -> list[list[str]]:
    proc = subprocess.run(
        [sys.executable, str(DB_QUERY), sql],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
    if len(lines) < 2:
        return []
    rows = []
    for line in lines[1:]:
        rows.append(line.split("\t"))
    return rows


def get_test_context() -> dict[str, str]:
    return {
        "AzTarikh": DEFAULT_AZ,
        "TaTarikh": DEFAULT_TA,
        "AzTarikhGozaresh": DEFAULT_AZ,
        "TaTarikhGozaresh": DEFAULT_TA,
        "Report_Time": DEFAULT_TA,
        "ccMarkazPakhsh": DEFAULT_MARKAZ,
    }


def page_reports(vb_file: Path) -> list[str]:
    text = vb_file.read_text(encoding="utf-8", errors="replace")
    return sorted(set(re.findall(r"ReportPath=/PakhshReports/([^&\"]+)", text)))


def rdl_namespace(root: ET.Element) -> str:
    return root.tag.split("}")[0].strip("{") if "}" in root.tag else ""


def parse_rdl(rdl_path: Path) -> dict[str, Any]:
    root = ET.parse(rdl_path).getroot()
    ns = {"r": rdl_namespace(root)}
    if not ns["r"]:
        ns = {}

    def q(tag: str) -> str:
        return f"{{{ns['r']}}}{tag}" if ns else tag

    report_params: dict[str, dict[str, str]] = {}
    for rp in root.findall(".//" + q("ReportParameter")):
        name = rp.attrib.get("Name", "")
        default = ""
        dv = rp.find(q("DefaultValue"))
        if dv is not None:
            val = dv.find(q("Values"))
            if val is not None:
                v = val.find(q("Value"))
                if v is not None and v.text:
                    default = v.text
        report_params[name] = {"default": default}

    datasets = []
    for ds in root.findall(".//" + q("DataSet")):
        ds_name = ds.attrib.get("Name", "")
        query = ds.find(q("Query"))
        if query is None:
            continue
        cmd = query.find(q("CommandText"))
        if cmd is None or not cmd.text:
            continue
        cmd_type_el = query.find(q("CommandType"))
        cmd_type = (cmd_type_el.text if cmd_type_el is not None else "Text") or "Text"
        qparams = []
        qp_root = query.find(q("QueryParameters"))
        if qp_root is not None:
            for qp in qp_root.findall(q("QueryParameter")):
                qparams.append(
                    {
                        "name": qp.attrib.get("Name", ""),
                        "value_expr": (qp.find(q("Value")).text or "")
                        if qp.find(q("Value")) is not None
                        else "",
                    }
                )
        fields = []
        for fld in ds.findall(q("Fields") + "/" + q("Field")):
            fields.append(fld.attrib.get("Name", ""))
        datasets.append(
            {
                "name": ds_name,
                "command": cmd.text.strip(),
                "command_type": cmd_type.strip(),
                "query_params": qparams,
                "fields": fields,
            }
        )

    headers = []
    for val in root.iter():
        if val.tag.endswith("Value") and val.text:
            for needle in COLUMN_HEADERS:
                if needle in val.text:
                    headers.append(val.text.strip())

    tedad_field = "TedadJayezeh" if any(
        "TedadJayezeh" in f for d in datasets for f in d["fields"]
    ) else None

    return {
        "report_params": report_params,
        "datasets": datasets,
        "headers": sorted(set(headers)),
        "tedad_field": tedad_field,
    }


def resolve_param_value(name: str, meta: dict[str, str], ctx: dict[str, str]) -> str:
    defaults = {
        "Report_Company": "Test Company",
        "Report_Name": "Test Report",
        "Report_Time": ctx["Report_Time"],
        "Report_User": "ssrs-tester",
        "AzTarikh": ctx["AzTarikh"],
        "TaTarikh": ctx["TaTarikh"],
        "AzTarikhGozaresh": ctx["AzTarikh"],
        "TaTarikhGozaresh": ctx["TaTarikh"],
        "ccMarkazPakhsh": ctx["ccMarkazPakhsh"],
        "NameMarkazPakhsh": "",
        "ccKalaCode": "",
        "ccMoshtary": "",
        "ccForoshandeh": "",
        "ccTaminKonandeh": "",
        "IsOnDpLine": "0",
        "IsOnDpLineFaktor": "0",
        "NoeGheymat": "0",
        "ccUser": "1",
        "ccSystem": "60890",
        "ShomarehFaktor": "",
        "CodeNoeVorod": "1",
        "TedadRoozForMianginKharid": "180",
        "AzTedadForosh": "",
        "TaTedadForosh": "",
        "AzMojody": "",
        "TaMojody": "",
        "Dolaty": "2",
        "NoeMarkaz": "0",
        "NoeGozareshAghlamForoshRafteh": "0",
        "NoeGozareshKhabKala": "0",
        "NoeGozareshFaktorHighRisk": "0",
        "CodeVazeiat": "",
        "GorohMoshtary": "",
    }
    if name in ctx:
        return ctx[name]
    if meta.get("default"):
        return meta["default"]
    return defaults.get(name, "")


def build_render_params(rdl_info: dict[str, Any], ctx: dict[str, str]) -> dict[str, str]:
    params: dict[str, str] = {}
    for name, meta in rdl_info["report_params"].items():
        if name.startswith("Report_"):
            params[name] = resolve_param_value(name, meta, ctx)
    for name, meta in rdl_info["report_params"].items():
        if name not in params:
            params[name] = resolve_param_value(name, meta, ctx)
    return params


def render_csv(report_path: str, params: dict[str, str]) -> str:
    encoded_path = urllib.parse.quote(report_path, safe="")
    query = [encoded_path, "rs:Command=Render", "rs:Format=CSV"]
    for key, value in sorted(params.items()):
        query.append(f"{urllib.parse.quote(key)}={urllib.parse.quote(str(value))}")
    url = REPORT_SERVER + "?" + "&".join(query)

    ps_script = (
        "$ErrorActionPreference = 'Stop'\n"
        f"$uri = @'\n{url}\n'@\n"
        "$params = @{ Uri = $uri; UseDefaultCredentials = $true; TimeoutSec = 180 }\n"
        "if (Get-Command Invoke-WebRequest | Select-Object -ExpandProperty Parameters | "
        "Where-Object { $_.Keys -contains 'AllowUnencryptedAuthentication' }) {\n"
        "  $params.AllowUnencryptedAuthentication = $true\n"
        "}\n"
        "$r = Invoke-WebRequest @params\n"
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)\n"
        "Write-Output $r.Content\n"
    )
    with tempfile.NamedTemporaryFile("w", suffix=".ps1", delete=False, encoding="utf-8") as fh:
        fh.write(ps_script)
        script_path = fh.name
    try:
        proc = subprocess.run(
            ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script_path],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    finally:
        os.unlink(script_path)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    return proc.stdout


def execute_dataset(
    dataset: dict[str, Any], render_params: dict[str, str]
) -> list[dict[str, Any]]:
    import pyodbc

    sql_params: list[tuple[str, Any]] = []
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
        sql = f"{{CALL {dataset['command']} ({placeholders})}}" if sql_params else f"{{CALL {dataset['command']}}}"
        values = [v for _, v in sql_params]
        cur.execute(sql, values)
    else:
        cur.execute(dataset["command"])

    columns = [d[0] for d in cur.description] if cur.description else []
    rows = []
    count = 0
    for row in cur.fetchall():
        rows.append({columns[i]: row[i] for i in range(len(columns))})
        count += 1
        if count >= 500:
            break
    conn.close()
    return rows


def csv_rows_with_column(
    csv_text: str,
    header_needles: tuple[str, ...],
    field_name: str | None = None,
) -> tuple[list[str], list[dict[str, str]]]:
    lines = [ln for ln in csv_text.splitlines() if ln.strip()]
    if not lines:
        return [], []

    header_idx = 0
    for i, line in enumerate(lines):
        if any(n in line for n in header_needles) or (
            field_name and field_name in line
        ):
            header_idx = i
            break

    reader = csv.reader(lines[header_idx:])
    headers = next(reader)
    col_idx = None
    if field_name and field_name in headers:
        col_idx = headers.index(field_name)
    if col_idx is None:
        for needle in header_needles:
            for i, h in enumerate(headers):
                if needle in h:
                    col_idx = i
                    break
            if col_idx is not None:
                break
    if col_idx is None:
        return headers, []

    out = []
    for row in reader:
        if not row or all(not c.strip() for c in row):
            continue
        if len(row) <= col_idx:
            continue
        raw = row[col_idx].strip().strip('"')
        if raw == "":
            continue
        out.append({"raw": raw, "row": row})
    return headers, out


def sum_decimal(values: list[str]) -> Decimal:
    total = Decimal("0")
    for v in values:
        s = v.replace(",", "").strip()
        if not s:
            continue
        try:
            total += Decimal(s)
        except Exception:
            pass
    return total


def db_sum(rows: list[dict[str, Any]], field: str) -> Decimal:
    total = Decimal("0")
    for row in rows:
        val = row.get(field)
        if val is None:
            continue
        total += Decimal(str(val))
    return total


def validate_report(
    report_name: str,
    rdl_path: Path,
    ctx: dict[str, str],
    work_dir: Path,
) -> dict[str, Any]:
    info = parse_rdl(rdl_path)
    if not info["tedad_field"]:
        return {
            "report": report_name,
            "status": "SKIP",
            "reason": "No TedadJayezeh field in dataset",
            "headers": info["headers"],
        }

    main_ds = next(
        (d for d in info["datasets"] if info["tedad_field"] in d["fields"]),
        info["datasets"][0] if info["datasets"] else None,
    )
    if not main_ds:
        return {
            "report": report_name,
            "status": "SKIP",
            "reason": "No dataset found",
            "headers": info["headers"],
        }

    render_params = build_render_params(info, ctx)
    report_path = f"/PakhshReports/{report_name}"

    result: dict[str, Any] = {
        "report": report_name,
        "report_path": report_path,
        "rdl": str(rdl_path),
        "headers": info["headers"],
        "dataset": main_ds["name"],
        "proc_or_sql": main_ds["command"],
        "params": render_params,
        "mode": "RDL-sourced stored proc vs SSRS CSV render",
    }

    try:
        csv_text = render_csv(report_path, render_params)
    except Exception as exc:
        result.update({"status": "RENDER_ERROR", "error": str(exc)})
        return result

    rendered_file = work_dir / f"{report_name}_rendered.csv"
    rendered_file.write_text(csv_text, encoding="utf-8")

    _, rendered_values = csv_rows_with_column(
        csv_text, COLUMN_HEADERS, info["tedad_field"]
    )
    rendered_nums = [r["raw"] for r in rendered_values if r["raw"]]
    rendered_total = sum_decimal(rendered_nums)

    try:
        db_rows = execute_dataset(main_ds, render_params)
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
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--vb-file",
        default=str(
            REPO_ROOT
            / "Pages"
            / "Sales"
            / "04-Gozareshat"
            / "KharidMoshtarianNesbatBeKolLink.aspx.vb"
        ),
    )
    parser.add_argument(
        "--rdl-dir",
        default=r"D:\repos\PakhshReports\PakhshReports",
    )
    parser.add_argument("--output-json", default="")
    args = parser.parse_args()

    vb_file = Path(args.vb_file)
    rdl_dir = Path(args.rdl_dir)

    ctx = get_test_context()

    candidates = []
    for report in page_reports(vb_file):
        rdl_path = rdl_dir / f"{report}.rdl"
        if not rdl_path.exists():
            continue
        info = parse_rdl(rdl_path)
        if info["headers"]:
            candidates.append((report, rdl_path, info))

    work_dir = Path(tempfile.mkdtemp(prefix="ssrs-jayeze-"))
    results = []
    for report, rdl_path, _ in candidates:
        results.append(validate_report(report, rdl_path, ctx, work_dir))

    payload = {
        "test_context": ctx,
        "note": "No RDL uses exact header 'تعداد جوایز'; matched 'تعداد جایزه' / TedadJayezeh",
        "results": results,
        "work_dir": str(work_dir),
    }
    text = json.dumps(payload, ensure_ascii=False, indent=2)
    print(text)
    if args.output_json:
        Path(args.output_json).write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
