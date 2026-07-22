---
name: daroupakhsh-erp-task-tester
description: >-
  End-to-end browser and database verification for Darou Pakhsh ERP WebForms pages
  in Source (legacy UI, port 3020) and Source-NewUI (redesigned UI, port 3010).
  Use when the user asks to test, verify, validate, or QA an ERP page, feature,
  export, or workflow after implementation — or when checking UI behavior against
  Pakhsh_Data_New. Do not use for SSRS report data validation (use ssrs-tester)
  or pure code review without runtime checks.
---

# Darou Pakhsh ERP Task Tester

Unified testing skill for **Source** (legacy) and **Source-NewUI** (redesign). Merged from the former `daroupakhsh-erp-task-tester` and `daroupakhsh-new-erp-task-tester` skills.

For company context, repo layout, and DB safety rules, see [daroo-pakhsh-distribution-company](../daroo-pakhsh-distribution-company/SKILL.md). For SQL queries, use [database-pakhsh-data-new](../database-pakhsh-data-new/SKILL.md).

## App profiles

| Profile | Codebase | Default port | Login URL | Run script |
|---------|----------|--------------|-----------|------------|
| **Legacy** | `C:/Users/a.dashti/TFS/Source` | 3020 | `/login.aspx` | `.\run.ps1 run [--port=3020]` |
| **New UI** | `C:/Users/a.dashti/TFS/Source-NewUI` | 3010 | `/Pages/login.aspx` | `.\run.ps1 run [--port=3010]` |

Pick the profile from the page path or user request. When unclear, check which repo contains the `.aspx` file.

## Pre-flight

1. Confirm which profile applies.
2. Check the app is running:

```powershell
# Legacy
cd C:\Users\a.dashti\TFS\Source
.\run.ps1 status

# New UI
cd C:\Users\a.dashti\TFS\Source-NewUI
.\run.ps1 status
```

3. Start if needed: `.\run.ps1 run` (add `--port=` only when the default is busy).
4. Read the target page markup (`.aspx`) and code-behind (`.aspx.vb`) before testing — identify controls, UpdatePanels, validation, and save actions.

## Login

Both profiles share the same control IDs on the login form.

| Field | Selector |
|-------|----------|
| Username | `#ctl00_cphMain_txt1` (legacy) or `#txt1` (New UI standalone page) |
| Password | `#ctl00_cphMain_txt2` or `#txt2` |
| Submit | `#ctl00_cphMain_btnLogin` or `#btnLogin` |

Test account (shared dev environment):

| Setting | Value |
|---------|-------|
| Username | `mp402842` |
| Password | `1` |

Use Playwright MCP (`user-playwright`) as the default browser tool. Always register a dialog handler before navigation — many ERP pages show `alert()` for validation and lock messages:

```javascript
page.on('dialog', d => d.accept());
```

Wait for post-login redirect (URL must not contain `login.aspx`) before opening the target page.

## Testing workflow

Copy this checklist and track progress:

```text
Task Progress:
- [ ] Profile selected (Legacy / New UI)
- [ ] App running on expected port
- [ ] Page code reviewed (.aspx + .aspx.vb)
- [ ] Logged in successfully
- [ ] Target page loads without access error
- [ ] Core scenarios exercised
- [ ] DB cross-check (if data-changing or export page)
- [ ] Results reported with evidence
```

### Step 1: Page load and access

- Navigate to the full page URL including query string params.
- Confirm no redirect to login and no access-denied label (`lblErrorBox`, validation summary, etc.).
- Record visible title, default filter values, and grid row count.

### Step 2: Scenario testing

Derive scenarios from the page's actual controls and code-behind — not a generic template:

- **Filters** — change dropdowns/date fields, trigger search, confirm grid updates.
- **CRUD** — add/edit/delete only when the user explicitly requests write testing.
- **Exports** — download CSV/Excel, inspect file contents and row counts.
- **Popups** — kala/supplier pickers, modal dialogs, async UpdatePanel postbacks.
- **Locks** — month locks, approval states, read-only grids (`TargetsLocked`, `lblSuccessBox`).

For New UI pages, also check scroll/layout: grids should fill the viewport inside `#mainContent` (see `Source-NewUI/docs/modules/main-content-fill-layout.md`).

### Step 3: Database cross-validation

When the page reads or writes data, verify against `Pakhsh_Data_New`:

```bash
python "%USERPROFILE%\.cursor\skills\database-pakhsh-data-new\scripts\query.py" --max-rows 10 "SELECT TOP 5 ..."
```

Rules:

- **SELECT only** unless the user explicitly approved write testing.
- Use `TOP N`, narrow `WHERE`, and `WITH (NOLOCK)` for exploratory reads.
- Match schema/table names from the page's `*_DB.vb` file.
- For exports: compare exported row count and key columns against a bounded SQL query.

### Step 4: Report results

Structure findings as:

```markdown
## ERP Test Report

**Page:** [path]
**Profile:** Legacy (3020) / New UI (3010)
**Goal:** [one sentence]

### Scenarios
| # | Scenario | Result | Evidence |
|---|----------|--------|----------|
| 1 | ... | PASS / FAIL / BLOCKED | brief observation |

### Blockers
- [Locked month, missing access, service down, etc.]

### DB checks
| Query | Result |
|-------|--------|
| ... | matches / mismatch / skipped |

### Verdict
**PASS** / **PARTIAL** / **FAIL** — [one paragraph]
```

**Evidence before claims.** Record URLs, control states, row counts, and query output — not "looks fine."

## Safety rules

1. **No unapproved writes** — do not click Save/Delete/Submit that persists data unless the user asked for it.
2. **Shared test DB** — treat `Pakhsh_Data_New` as shared; never run bulk UPDATE/DELETE.
3. **Alert dialogs** — accept and note the message; a lock alert is a valid test result (BLOCKED), not a retry loop.
4. **Screenshots** — capture before/after for UI redesign tasks (`Source-NewUI/Screenshots/`).

## Related skills and paths

See [agent_paths.md](agent_paths.md) for full path reference.

| Need | Skill / path |
|------|-------------|
| Company / DB context | `daroo-pakhsh-distribution-company` |
| SQL queries | `database-pakhsh-data-new` |
| SSRS report data | `TFS/Source/.cursor/skills/ssrs-tester` |
| New UI redesign patterns | `TFS/Source-NewUI/.cursor/skills/redesign` |
| Popup kala selection | `TFS/Source-NewUI/.cursor/skills/popup-kala-selection` |
| Post-task verification | `report-for-test` (goal-verifier subagent) |
