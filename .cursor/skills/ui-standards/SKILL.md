---
name: ui-standards
description: >-
  Enforces general UI standards for layout, spacing, typography, visual hierarchy,
  control consistency, and accessibility while building or reviewing user interfaces.
  Use when implementing, editing, or reviewing UI code, pages, components, forms,
  or when the user asks to follow UI standards, design system rules, or polish the interface.
---

# UI Standards

## Overview

- Scope: any user-facing UI — web pages, forms, dialogs, dashboards, mobile screens
- Apply during implementation **and** run the verification checklist before marking UI work complete
- Stack-agnostic defaults below; project design system, component library, or nearby files override when present
- Exclusions: backend-only APIs, SSRS report data logic, pure business-rule changes with no visual impact
- Related skills: `exprience` (page-level audit and proposals), `test-frontend-ui-ux` (post-change verification), `frontend-ui-ux-human-prompt-to-technical-prompt` (prompt polish)

## Objectives

1. Match or improve the project's existing visual language — never introduce one-off styling
2. Produce scannable layouts with clear hierarchy, consistent controls, and adequate spacing
3. Meet baseline accessibility: labels, focus, contrast, and touch/click targets
4. Verify every changed screen or component against the checklist before finishing

## Workflow

### Step 1: Discover project UI context

Before writing or changing UI:

1. Search for a design system or style guide:
   - `README`, `docs/design*`, `DESIGN.md`, `tokens.*`, `theme.*`, `variables.css`, `tailwind.config.*`
2. Identify the UI stack and shared primitives:
   - Component folder (`components/`, `Controls/`, `Shared/`)
   - Global styles (`global.css`, `site.css`, `App.css`, master page / layout file)
3. Open **two similar existing screens** in the same app and mirror their patterns for spacing, headings, buttons, tables, and form layout
4. When project rules conflict with defaults in this skill, **project rules win**

### Step 2: Apply standards while implementing

Work through each area that applies to the change:

**Layout and structure**

- [ ] One primary action per section; secondary actions visually subordinate
- [ ] Group related fields and actions (fieldset, card, panel, section heading)
- [ ] Align to a grid — columns and gutters consistent with sibling pages
- [ ] Reserve full width for data-heavy content (tables, wide forms); avoid arbitrary max-width on operational screens

**Spacing**

- [ ] Use the project spacing scale when one exists; otherwise use an 4px or 8px base (4, 8, 12, 16, 24, 32, 48)
- [ ] Section padding: 16–24px; gap between related controls: 8–12px; gap between sections: 24–32px
- [ ] Never butt controls against container edges without padding
- [ ] Labels sit consistently above or beside inputs — match the rest of the app, do not mix

**Typography and hierarchy**

- [ ] Page title largest and distinct; section headings smaller; body text readable (typically 14–16px equivalent)
- [ ] One font family stack unless the project defines more
- [ ] Muted text for hints and secondary metadata only — not for primary labels or required fields
- [ ] Truncate or wrap long text intentionally; no overflow clipping without ellipsis or wrap rule

**Color and surfaces**

- [ ] Use project tokens or existing CSS classes for primary, secondary, danger, success, borders, backgrounds
- [ ] Do not hard-code one-off hex/rgb values when a shared variable or class exists
- [ ] Error and validation states use the app's established danger color and placement (inline under field, summary at top, etc.)
- [ ] Avoid harsh 1px black borders and default unstyled browser controls on production screens

**Controls and consistency**

- [ ] Reuse existing button, input, select, table, modal, and badge components — do not duplicate with raw HTML unless the app already does
- [ ] One button style for primary actions app-wide on the page; destructive actions use danger variant
- [ ] Same control heights within a form row; aligned baselines in horizontal toolbars
- [ ] Icons: same set and size as the rest of the app; include text labels when icons alone are ambiguous
- [ ] Loading, empty, and error states present where data is fetched or lists can be empty

**Forms**

- [ ] Every input has a visible label (or `aria-label` / `aria-labelledby` when the pattern is established)
- [ ] Required fields marked consistently with the rest of the app
- [ ] Tab order follows visual order; logical grouping for keyboard users
- [ ] Validation messages specific and adjacent to the field they describe

**Tables and dense data**

- [ ] Column headers align with data type (text left, numbers right when appropriate)
- [ ] Row hover and selection styles match sibling tables
- [ ] Sticky header or pagination for long lists when the app already uses that pattern
- [ ] Avoid unreadable density — minimum row height and cell padding consistent with existing grids

**Accessibility (baseline)**

- [ ] Color contrast ≥ 4.5:1 for normal text, ≥ 3:1 for large text and UI boundaries (use project tokens that already pass)
- [ ] Interactive targets ≥ 44×44px or padded to equivalent click area
- [ ] Focus visible on keyboard navigation — do not remove outline without a replacement
- [ ] Images and icons that convey meaning have accessible names
- [ ] Status not conveyed by color alone — add text, icon, or pattern

**Responsive behavior**

- [ ] Layout works at the project's supported breakpoints; no horizontal scroll on standard viewports unless the screen is intentionally wide-table
- [ ] Touch-friendly spacing on mobile when the app is responsive

### Step 3: Verify before finishing

After implementation, review every touched screen or component:

1. Compare side-by-side with a reference screen from the same app — spacing, fonts, and controls should feel native
2. Walk the **Self-check checklist** below; fix any fail
3. If the app runs locally, open the page in a browser or capture a screenshot; confirm no overflow, overlap, or broken alignment
4. For non-trivial UI changes, note in the completion summary which standards were applied and any project-specific overrides used

**Self-check checklist**

```text
UI standards verification:
- [ ] Matches project design system / sibling pages
- [ ] Clear visual hierarchy (title → sections → actions)
- [ ] Consistent spacing and alignment
- [ ] Reused shared components and tokens (no one-off styles)
- [ ] All inputs labeled; errors clear and localized
- [ ] Primary vs secondary actions distinct
- [ ] Loading / empty / error states handled
- [ ] Contrast and focus acceptable
- [ ] No obvious layout defects at target viewport
```

Mark UI work complete only when all applicable items pass.

### Step 4: Report violations fixed

When correcting existing messy UI, list changes in plain language:

```text
UI standards applied:
- [file or page]: [what was wrong] → [what was changed]
```

## Safety rules

1. **Never** change field names, bindings, or business logic to achieve visual polish
2. **Never** introduce a new color, font, or component pattern when the project already defines one — reuse it
3. **Never** remove focus indicators without an accessible replacement
4. **Never** mark UI work complete without running Step 3 when any visual files changed
5. **Always** read nearby pages and shared styles before adding new UI
6. **Always** prefer the project's design system over the default tables in this skill

## Key facts & reference

| Item | Value |
|------|-------|
| Skill path | `~/.cursor/skills/ui-standards/SKILL.md` |
| Default spacing base | 4px or 8px scale: 4, 8, 12, 16, 24, 32, 48 |
| Section padding | 16–24px |
| Control gap | 8–12px within groups; 24–32px between sections |
| Body text | 14–16px equivalent unless project defines otherwise |
| Min contrast | 4.5:1 normal text; 3:1 large text / UI chrome |
| Min touch target | 44×44px (or equivalent padding) |
| Related skills | `exprience`, `test-frontend-ui-ux`, `frontend-ui-ux-human-prompt-to-technical-prompt` |

### Project discovery paths

| Look for | Typical locations |
|----------|-------------------|
| Design tokens / theme | `tokens.*`, `theme.*`, `variables.css`, `tailwind.config.*`, `_variables.scss` |
| Component library | `components/`, `src/components/`, `Controls/`, `Shared/` |
| Global styles | `global.css`, `site.css`, `App.css`, `MasterPage`, `Layout.*` |
| Docs | `README`, `docs/design*`, `DESIGN.md`, Storybook config |

### Trigger phrases

- "follow UI standards"
- "enforce design system"
- "make this consistent with the rest of the app"
- "polish the UI"
- "fix spacing and alignment"
- "improve visual hierarchy"
- "UI looks inconsistent"

### When standards conflict

| Situation | Rule |
|-----------|------|
| Project design system exists | Follow it |
| No design system | Mirror the two most similar existing pages |
| User asks for explicit style | User request wins if it does not break accessibility baseline |
| Legacy page with mixed patterns | Match the dominant pattern on that module's other pages |
