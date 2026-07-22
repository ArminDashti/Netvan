---
name: ui-page-proposals-implement
description: >-
  Implements UI fixes from page URLs, user prompts, and per-page proposal files
  in ./proposals/ui/. Reads improvement prompts, edits markup/CSS/components,
  and verifies against ui-standards. Use when the user supplies page URLs with
  proposals, asks to apply UI proposals, implement page cleanup, or fix messy
  pages from ./proposals/ui/.
---

# UI Page Proposals Implement

## Overview

- Scope: take a user prompt, one or more page URLs/routes, and matching proposal files — then edit source to fix UI/UX issues
- Input: user instructions + page targets + `./proposals/ui/<PAGE-NAME>.txt` (from `ui-page-audit-proposals` or hand-written)
- Output: code changes in the project; optional browser verification after edits
- Exclusions: backend-only APIs, SSRS data logic, business-rule changes, renaming fields/bindings, new features beyond the proposal
- Related skills: `ui-page-audit-proposals` (writes proposals), `ui-standards`, `browser-automation`, `test-frontend-ui-ux`

## Objectives

1. Resolve every supplied page to its source files and open the matching proposal
2. Implement fixes from the proposal **Improvement prompt** and user prompt — UI only
3. Reuse project patterns, shared components, and reference pages named in the proposal
4. Verify each page against the proposal **Verification** checklist and `ui-standards`
5. Report what was changed, which pages were fixed, and any proposals left incomplete

## Workflow

### Step 1: Collect inputs

Accept from the user:

| Input | Examples |
|-------|----------|
| User prompt | "Fix spacing and buttons on these pages" |
| Page URLs / routes | `http://localhost/Admin/Users.aspx`, `/orders`, `~/Reports/Sales.aspx` |
| Proposal files | `./proposals/ui/admin-users.txt`, or "use proposals for all listed pages" |

If proposal paths are omitted, derive `<PAGE-NAME>.txt` from each page slug (same rules as `ui-page-audit-proposals` Step 5).

If a proposal file is missing:

- Ask once for the file or permission to proceed from the user prompt alone
- Do not invent issues — only fix what the prompt or proposal states

Normalize each page:

| Field | Source |
|-------|--------|
| `page_label` | Proposal `# UI improvement prompt` header or user label |
| `open_target` | URL/route from user or proposal **Page** section |
| `source_paths` | Proposal **Source** line; else search by route, `.aspx`, `pages/`, `views/` |
| `proposal_path` | `./proposals/ui/<PAGE-NAME>.txt` |

### Step 2: Read proposals and plan

For each page:

1. Read the full proposal file
2. Extract and follow in order:
   - **Issues found** — what to fix
   - **Reference** — sibling pages and shared components to mirror
   - **Improvement prompt** — primary implementation spec (self-contained)
   - **Verification** — acceptance checklist
3. Read `ui-standards` and run its Step 1 discovery (tokens, shared components, two sibling pages)
4. Open reference pages/files named in the proposal; note patterns to copy
5. List source files to edit (markup, code-behind styling only if needed, CSS, component files)

### Step 3: Implement UI fixes

For each page, apply fixes in this order:

1. **Structure and layout** — grid, sections, alignment, grouping
2. **Spacing and typography** — padding, gaps, headings, hierarchy
3. **Controls** — reuse shared buttons, inputs, tables, modals from reference pages
4. **States and accessibility** — labels, focus, loading/empty/error, contrast
5. **Responsive** — overflow, breakpoints if mentioned in proposal

Rules while editing:

- Match reference pages and project design tokens — no one-off colors, fonts, or control styles
- Preserve business logic, event handlers, field names, IDs used by code-behind, and data bindings
- Prefer editing existing CSS classes and shared components over inline styles
- One page at a time; finish and verify before moving to the next unless the user asks for batch edits

### Step 4: Verify in browser (when app is runnable)

If the app can run or is already running:

1. `browser_navigate` to `open_target`
2. `browser_snapshot` + `browser_screenshot`
3. Compare visually to reference pages and proposal **Issues found**
4. Re-run `ui-standards` Step 3 self-check on the changed screen
5. Fix remaining gaps; repeat until proposal **Verification** items pass

If the app is not runnable, verify from source + `ui-standards` checklist and note that live verification was skipped.

### Step 5: Report results

After all pages are processed:

```text
UI proposal implementation complete:
- Pages fixed: <page_label → files changed>
- Proposals used: <proposal_path>
- Verification: <passed | partial — list open items>
- Skipped: <pages with no proposal or blocked reason>
```

List concrete changes per page (layout, controls, CSS paths) — not a raw diff dump.

## Safety rules

1. **Never** change business logic, validation rules, SQL, API calls, or field/bindings names unless the user explicitly overrides
2. **Never** delete functional controls or hide required fields to "clean up" layout
3. **Never** hard-code secrets, credentials, or environment-specific URLs in UI files
4. **Never** ignore the proposal **Improvement prompt** when a proposal file exists — it is the primary spec
5. **Always** read the proposal before editing; do not guess issues from the URL alone
6. **Always** follow `ui-standards` and mirror **Reference** pages from the proposal
7. **Always** run proposal **Verification** and `ui-standards` self-check before marking a page done

## Key facts & reference

| Item | Value |
|------|-------|
| Skill path | `~/.cursor/skills/ui-page-proposals-implement/SKILL.md` |
| Proposal dir | `./proposals/ui/` |
| Proposal file | `./proposals/ui/<PAGE-NAME>.txt` |
| Page slug rules | Same as `ui-page-audit-proposals` (lowercase, hyphens, strip path chars) |
| Primary spec | Proposal **Improvement prompt** section |
| Quality baseline | `ui-standards` |
| Inspection tools | `browser-automation` MCP when app is available |
| Upstream skill | `ui-page-audit-proposals` |
| Post-verify skill | `test-frontend-ui-ux` (optional, when user wants formal test pass) |

### Trigger phrases

- "implement UI proposals"
- "apply proposals/ui"
- "fix pages from proposals"
- "implement page cleanup"
- "edit code from proposal files"
- page URL + `./proposals/ui/` path together

### Proposal sections used

| Section | Use |
|---------|-----|
| Page | Resolve URL, label, source paths |
| Issues found | Scope and priority of fixes |
| Reference | Patterns and components to reuse |
| Improvement prompt | Ordered implementation instructions |
| Verification | Done criteria before reporting complete |

### When proposal and user prompt conflict

1. User prompt wins on **scope** (which pages, skip items, extra constraints)
2. Proposal wins on **specific UI defects** listed in **Issues found**
3. If still ambiguous, ask once — do not guess
