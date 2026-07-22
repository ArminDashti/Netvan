---
name: ssrs-data-validation
description: Use this skill whenever the user wants to validate, test, or verify that an SSRS (SQL Server Reporting Services) report's DATA is correct — as opposed to its layout/visuals. Triggers on phrases like "test this SSRS report", "validate report data", "check if the report numbers are right", "diff the report against the database", "is the SSRS output correct", or any request involving comparing rendered SSRS output (CSV/XML/PDF) against the underlying SQL Server data. Applies to SSRS 2017+ (ReportServer URL access and ReportExecutionService SOAP), with code samples in VB.NET. Also use this when the user references an .rdl file and wants to know whether its output matches its query, or wants to build/run a regression check after a report change. Do NOT use for general SSRS report authoring/design, or for pure UI/layout/pixel testing — this skill is specifically about data correctness.
---

# SSRS Data Validation

## What this skill is for

SSRS reports are a black box from the outside: you get rendered output, not the
underlying dataset. Validating "is the data correct" means building a path from
**rendered report output → comparable structured data → known-good data**, and
diffing the two. This skill governs how an agent should reason through that path,
not a fixed sequence of steps — the right approach depends on what's available
(RDL access, DB access, parameter complexity).

Environment assumptions: SSRS 2017 (version 14), VB.NET as the calling language,
Windows/NTLM auth against the report server, SQL Server as the data source.

**Project-specific paths:**
- RDL files: `C:/Users/a.dashti/TFS/RDL`
- Report server base URL: `http://localhost:3020` (test) or the shared server at `10.10.12.52`
- Database: `Pakhsh_Data_New` on `10.10.12.52`, user `public01`

## Core principle: prefer the RDL as source of truth, fall back to raw SQL

The dataset SQL embedded in the `.rdl` is the actual query SSRS runs. Querying it
directly and diffing against the rendered output tests "did SSRS render what the
query produced" — a narrower, more reliable claim than "is the report correct,"
which requires business-logic judgment the agent doesn't have.

**Reasoning order, not a checklist:**

1. **Is the `.rdl` file available?** If yes, parse it to extract the embedded or
   shared dataset SQL (see `references/rdl-parsing.md`). RDL files for this project
   live at `C:/Users/a.dashti/TFS/RDL`. Run that exact query independently. This is
   the strongest comparison — same query, two execution paths (SSRS engine vs. direct
   ADO.NET) — and it isolates rendering/formatting bugs from query/business-logic bugs.
2. **Is only a raw SQL query or table available, no `.rdl`?** Fall back to running
   that query directly via SQL Server and treat it as ground truth. Be explicit
   with the user that this assumes the supplied query is correct — the agent is
   then only testing "does the render match a query," not "is the query right."
3. **Is neither available, just a live report?** Render the report and inspect
   it for internal consistency (totals reconcile with line items, no duplicate
   keys, grouping/sorting behaves) rather than fabricating an external source of
   truth. Say plainly that this is a weaker check.

Always tell the user which mode you're in and what it does and doesn't prove —
"render matches query" is not the same claim as "query is semantically correct."

## Step 1: Get the rendered output

Use **URL Access** (HTTP GET) as the default — it's a single request, trivial to
script, and returns CSV/XML cleanly. Reach for **SOAP** (`ReportExecutionService`)
only when URL Access can't do what's needed: report history snapshots, render-to-
stream without touching disk, or session-based multi-step execution. Both are
documented with working VB.NET in `references/rendering-methods.md`.

**Format choice matters for testability:** request `CSV` or `XML`, never `PDF`/
`EXCELOPENXML`/`WORD`, unless the task is specifically layout testing. Formatted
binary output forces you to fight rendering noise instead of comparing data.

## Step 2: Get comparison data

Per the core principle above — RDL-extracted SQL first, raw SQL fallback second.
See `references/rdl-parsing.md` for extracting dataset queries from `.rdl` XML
(handles both embedded `<Query>` blocks and `SharedDataSet` references).

Execute the query via `System.Data.SqlClient` (or `Microsoft.Data.SqlClient` if
the codebase has migrated) directly against SQL Server — don't go through SSRS
for this half of the comparison, or you're not testing anything independent.

Connection string: `Server=10.10.12.52;Database=Pakhsh_Data_New;User Id=public01;Password=Public01Public01;`

## Step 3: Normalize before diffing

Don't string-diff raw CSV against a `DataTable`. Normalize both sides to a common
shape first, or every formatting difference looks like a data bug:

- **Numbers**: parse to `Decimal`/`Double`, don't compare formatted strings
  (`"1,234.50"` vs `1234.5` are equal; a naive diff says they aren't).
- **Dates**: parse to `DateTime`, watch for locale/format mismatches and SSRS's
  tendency to apply report-level formatting that the raw SQL won't have.
- **Nulls/blanks**: SSRS often renders `NULL` as empty string — decide up front
  whether empty string and NULL should be treated as equal for this report.
- **Row order**: if the report doesn't have an explicit, stable sort, don't
  diff positionally — sort both sides by a shared key first.
- **Tolerance**: for computed/aggregate values, use a small epsilon for floating
  point comparison rather than exact equality.

`scripts/normalize_and_diff.py` implements this normalization and produces a
structured discrepancy report (row/column references, expected vs. actual).
Prefer it over writing ad hoc comparison code each time.

```bash
python .cursor/skills/ssrs-tester/scripts/normalize_and_diff.py \
    --rendered rendered.csv --expected expected.csv \
    --key-columns OrderID,LineNumber --null-equals-empty
```

## Step 4: Report findings

Structure findings as: which mode was used (RDL-sourced vs. raw-SQL vs.
internal-consistency-only), what was compared, and a list of discrepancies with
enough context to locate them (row key, column, expected, actual) — not just
"3 rows differ." If everything matches, say what was actually verified, not just
"passed," since the scope of the check varies by mode.

## Known SSRS gotchas to account for

- **Cached/snapshot data**: a report may serve a cached execution or snapshot
  rather than live data. If the report has caching or a snapshot schedule
  configured, the rendered output can legitimately differ from a live query run
  right now — check execution settings before treating a mismatch as a bug.
- **Shared datasets**: defined once, referenced by multiple reports — the `.rdl`
  may only contain a reference, not the SQL itself. See `references/rdl-parsing.md`
  for resolving these via the `.rsd` file or ReportServer catalog.
- **Parameter cascades**: dependent parameters (e.g., branch → region) can
  silently produce empty result sets if a parameter combination is invalid —
  distinguish "report is broken" from "this parameter combination has no data."
- **Locale formatting**: number/date formatting is applied at render time and
  varies by report locale settings — always compare parsed values, never
  formatted strings.

## Reference files

- `references/rendering-methods.md` — URL Access and SOAP VB.NET examples,
  authentication, format options, parameter passing.
- `references/rdl-parsing.md` — How to extract dataset SQL from `.rdl` XML,
  including shared datasets.
- `scripts/normalize_and_diff.py` — Normalization and structured diff utility
  for comparing rendered CSV/XML output against query results.
