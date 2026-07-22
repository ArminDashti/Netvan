"""Shared connection helpers for Pakhsh database scripts."""
from __future__ import annotations

import os

DEFAULT_CONN = (
    "DRIVER={{ODBC Driver 17 for SQL Server}};"
    "SERVER={server};DATABASE={database};"
    "UID={user};PWD={password};"
    "Encrypt=no;TrustServerCertificate=yes;Connect Timeout=60;"
)


def _parse_ado_connection_string(ado: str) -> dict[str, str]:
    parts: dict[str, str] = {}
    for segment in ado.split(";"):
        segment = segment.strip()
        if not segment or "=" not in segment:
            continue
        key, value = segment.split("=", 1)
        parts[key.strip().lower()] = value.strip()
    return parts


def _ado_to_odbc(ado: str) -> str:
    parts = _parse_ado_connection_string(ado)
    server = parts.get("server") or parts.get("data source", "")
    database = parts.get("database") or parts.get("initial catalog", "")
    user = parts.get("user id") or parts.get("uid", "")
    password = parts.get("password") or parts.get("pwd", "")
    encrypt = parts.get("encrypt", "false").lower() in {"true", "yes", "1"}
    trust = parts.get("trustservercertificate", "false").lower() in {"true", "yes", "1"}

    return (
        "DRIVER={ODBC Driver 17 for SQL Server};"
        f"SERVER={server};DATABASE={database};UID={user};PWD={password};"
        f"Encrypt={'yes' if encrypt else 'no'};"
        f"TrustServerCertificate={'yes' if trust else 'no'};"
        "Connect Timeout=60;"
    )


def get_connection_string() -> str:
    ado = os.environ.get("PAKHSH_DATA_NEW_DB_SQL_SERVER_CONNECTION_STRING", "").strip()
    if ado:
        return _ado_to_odbc(ado)

    return DEFAULT_CONN.format(
        server=os.environ.get("PAKHSH_DB_SERVER", "10.10.12.52"),
        database=os.environ.get("PAKHSH_DB_NAME", "Pakhsh_Data_New"),
        user=os.environ.get("PAKHSH_DB_USER", "public01"),
        password=os.environ.get("PAKHSH_DB_PASSWORD", "Public01Public01"),
    )


def connect():
    import pyodbc

    return pyodbc.connect(get_connection_string())
