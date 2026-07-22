# Agent Paths — Darou Pakhsh ERP Task Tester

Quick reference for paths used during ERP page testing. Combines legacy Source and Source-NewUI layouts.

## Codebases

| Profile | Root | Default URL |
|---------|------|-------------|
| Legacy (Source) | `C:/Users/a.dashti/TFS/Source` | `http://localhost:3020` |
| New UI (Source-NewUI) | `C:/Users/a.dashti/TFS/Source-NewUI` | `http://localhost:3010` |

## Run scripts

| Profile | Script | Commands |
|---------|--------|----------|
| Legacy | `C:/Users/a.dashti/TFS/Source/run.ps1` | `run`, `stop`, `status`, `build` |
| New UI | `C:/Users/a.dashti/TFS/Source-NewUI/run.ps1` | `run`, `stop`, `status`, `build` |

## Login pages

| Profile | Path |
|---------|------|
| Legacy | `C:/Users/a.dashti/TFS/Source/login.aspx` |
| New UI | `C:/Users/a.dashti/TFS/Source-NewUI/Pages/login.aspx` |

## Page discovery

Given a relative path like `Pages/Sales/03-Amaliat/Foo.aspx`:

1. Search both roots — the file exists in one (sometimes both during migration).
2. Read `.aspx`, `.aspx.vb`, and any `App_Code/**/Foo_DB.vb` companion.

## Database

| Resource | Path |
|----------|------|
| Query script | `C:/Users/a.dashti/.cursor/skills/database-pakhsh-data-new/scripts/query.py` |
| SQL scripts (TFS) | `C:/Users/a.dashti/TFS/SQL` |
| Web.config connection | `{SourceRoot}/Web.config` → `AppConStr` |

## New UI docs (layout / redesign)

| Topic | Path |
|-------|------|
| Main content fill / scroll | `C:/Users/a.dashti/TFS/Source-NewUI/docs/modules/main-content-fill-layout.md` |
| Popup kala selection | `C:/Users/a.dashti/TFS/Source-NewUI/docs/modules/popup-kala-selection.md` |
| Potential bugs log | `C:/Users/a.dashti/TFS/Source-NewUI/docs/potentional-bugs/` |
| Screenshots (before/after) | `C:/Users/a.dashti/TFS/Source-NewUI/Screenshots/` |

## SSRS (when page embeds or links reports)

| Resource | Path |
|----------|------|
| RDL files | `C:/Users/a.dashti/TFS/RDL` |
| SSRS tester skill | `C:/Users/a.dashti/TFS/Source/.cursor/skills/ssrs-tester/SKILL.md` |
| Report server (local) | `http://localhost:3020` (legacy app hosts ReportViewer) |

## Cursor skills (personal)

| Skill | Path |
|-------|------|
| This skill | `C:/Users/a.dashti/.cursor/skills/daroupakhsh-erp-task-tester/SKILL.md` |
| Company context | `C:/Users/a.dashti/.cursor/skills/daroo-pakhsh-distribution-company/SKILL.md` |
| Database | `C:/Users/a.dashti/.cursor/skills/database-pakhsh-data-new/SKILL.md` |
