---
name: prepare-for-docker
description: >-
  Creates or edits project-agnostic Docker PowerShell scripts
  (run-on-docker-local.ps1, run-on-docker-server.ps1, create-image.ps1) and
  deploys stacks locally or over SSH. Requires user clarification before any
  deploy. Use when the user asks to run, build, or deploy in Docker, prepare a
  Docker stack, create Docker scripts, or mentions run-on-docker-local.ps1,
  run-on-docker-server.ps1, or create-image.ps1.
---

# Prepare for Docker

## Overview

- **Project-agnostic:** works in any repo; no hardcoded project names, ports, or paths.
- **Primary output:** create or edit these scripts in the target project root:
  - `run-on-docker-local.ps1`
  - `run-on-docker-server.ps1`
  - `create-image.ps1`
- If a script already exists, edit it in place (refresh params/defaults/help); do not skip or overwrite blindly without preserving project-specific build/run logic.
- Deploy via the run scripts after mandatory discovery; build images via `create-image.ps1`.
- Exclusions: do not reimplement Docker steps manually outside these scripts.
- Related: `create-powershell-script` skill for script conventions.

## Objectives

1. Create or edit all three scripts with the parameter contracts below.
2. Gather connection, port, volume, and image choices from the user before acting.
3. Inspect existing images and volumes tied to the target container/stack.
4. Run the appropriate script with flags that match the user's answers.
5. Report endpoints, paths, and any cleanup performed.

## Workflow

### Step 1: Create or edit the three scripts

Target directory: project repo root (or path the user names).

For each file: **create if missing; edit if present** so flags, null defaults, help, and placeholder resolution match this skill.

| Script | Role |
|--------|------|
| `create-image.ps1` | Build (and optionally tag/push) the project image |
| `run-on-docker-local.ps1` | Build/run stack on the local Docker daemon |
| `run-on-docker-server.ps1` | Build/run stack on a remote host over SSH |

#### `run-on-docker-local.ps1` / `run-on-docker-server.ps1` parameters

Use `--flag=value`. Default **null** means “not passed”; resolve at runtime as shown.

| Flag | Default | Null means |
|------|---------|------------|
| `--ssh-string` | `null` | Local: treat as local daemon (`localhost`). Server: **required** — SSH config alias only |
| `--delete-image` | `null` | `no` |
| `--delete-volume` | `null` | `no` |
| `--internal-port` | `null` | Random unused port from safe range `30000–32767` |
| `--volume-dir` | `null` | `<USER-PROFILE-NAME>/docker/<CONTAINER-NAME>` |
| `--volume-name` | `null` | `<CONTAINER-NAME>-volume` |
| `--network-name` | `null` | `<CONTAINER-NAME>-network` |
| `--help` | — | Show help; exit |

#### Placeholder resolution

| Placeholder | Resolve from |
|-------------|--------------|
| `<CONTAINER-NAME>` | Compose `container_name`, `.docker/stack.manifest.json` `stackName`, image name, or repo folder name |
| `<USER-PROFILE-NAME>` | `$env:USERNAME` on Windows; `$USER` on Linux/macOS (path under that user's home / profile when materializing `--volume-dir`) |
| Safe port | Random free port in `30000–32767`; verify with `Get-NetTCPConnection` / `netstat` before binding |

#### `create-image.ps1`

- Project-agnostic image build from the project's `Dockerfile` (or path the user names).
- Must support `--help` / `-h` / `/?`.
- Keep flags minimal (e.g. `--image-name`, `--tag`, `--dockerfile`, `--context`); resolve name/tag from container/image/repo when null.
- Prefer calling this from the run scripts when a rebuild is needed, or run it alone when the user only wants a build.

#### Script conventions

- Follow `create-powershell-script`: `--flag=value`, `Show-Help`, help on missing/invalid args, simple colorful progress.
- Test with `--help` after create/edit; ask before a full deploy or destructive run.

**`--help` requirements** for every script — print when `--help`, `-h`, `/?`, or validation fails:

1. **Usage** — one-line syntax with all flags
2. **Flags** — each flag with default (`null` + resolved meaning) and one-line description
3. **Examples** — at least three copy-paste examples
4. **Notes** — SSH alias rule, truthy yes/no values, safe port range, null-default behavior

Help template for run scripts (adapt script name and local vs server notes):

```text
run-on-docker-local.ps1 — deploy <CONTAINER-NAME> on local Docker

USAGE:
  .\run-on-docker-local.ps1 [flags]

FLAGS:
  --ssh-string=<alias>       SSH alias; null → local daemon (default: null)
  --delete-image=<no|yes>    Remove built images during teardown (default: null → no)
  --delete-volume=<no|yes>   Remove volumes before recreate (default: null → no)
  --internal-port=<port>     Host port mapped to the container (default: null → random 30000–32767)
  --volume-dir=<path>        Bind-mount data directory (default: null → <USER-PROFILE-NAME>/docker/<CONTAINER-NAME>)
  --volume-name=<name>       Named Docker volume (default: null → <CONTAINER-NAME>-volume)
  --network-name=<name>      Docker network (default: null → <CONTAINER-NAME>-network)
  --help                     Show this help

EXAMPLES:
  .\run-on-docker-local.ps1
  .\run-on-docker-local.ps1 --delete-volume=yes
  .\run-on-docker-local.ps1 --internal-port=30042

NOTES:
  - Use SSH config alias only; do not include "ssh" in --ssh-string.
  - Null defaults resolve as described in FLAGS.
  - Truthy values for yes/no flags: yes, true, 1, y, on.
  - Default internal port is picked randomly from 30000–32767 if not specified.
```

For `run-on-docker-server.ps1`, require `--ssh-string=<alias>` and use remote examples.

Implementation sketch:

```powershell
if ($args -match '^(--help|-h|/\?)$') { Show-Help; exit 0 }
```

### Step 2: Clarify with the user (required — do this before deploy)

**Do not build, deploy, delete, or run deploy scripts until the user answers.**

| Topic | Ask |
|-------|-----|
| **Target** | Local (`run-on-docker-local.ps1`) or server (`run-on-docker-server.ps1`)? |
| **Connection** | For server: SSH config alias only (never include `ssh`) |
| **Container name** | If not obvious from the project |
| **Existing images** | After Step 3: keep or delete/rebuild? |
| **Existing volumes** | After Step 3: keep data or delete? |
| **Internal port** | Random from `30000–32767` or fixed? |
| **Volume location** | Default `<USER-PROFILE-NAME>/docker/<CONTAINER-NAME>` or custom? |
| **Destructive actions** | Confirm before `--delete-image=yes`, `--delete-volume=yes`, or remote deploy |

### Step 3: Inspect existing Docker state

```powershell
docker images --format "{{.Repository}}:{{.Tag}}" | Select-String -Pattern "<stack-or-container-name>"
docker volume ls
docker ps -a --filter "name=<container-name>"
```

Remote (after user provides SSH alias):

```powershell
ssh <alias> "docker images; docker volume ls; docker ps -a"
```

Present findings, then ask keep vs delete.

### Step 4: Map answers to script flags

| User choice | Flag |
|-------------|------|
| Local | Run `.\run-on-docker-local.ps1` (omit `--ssh-string` or leave null) |
| SSH alias `example` | `.\run-on-docker-server.ps1 --ssh-string=example` |
| Delete images | `--delete-image=yes` |
| Keep images | omit flag or `--delete-image=no` |
| Delete volumes | `--delete-volume=yes` |
| Keep volumes | omit flag or `--delete-volume=no` |
| Internal port | `--internal-port=<port>` or omit for random |
| Custom volume dir | `--volume-dir=<path>` |
| Volume name | `--volume-name=<name>` or omit for `<CONTAINER-NAME>-volume` |
| Network name | `--network-name=<name>` or omit for `<CONTAINER-NAME>-network` |

Truthy yes/no: `yes`, `true`, `1`, `y`, `on`.

### Step 5: Run the script

From the directory that contains the scripts:

```powershell
.\create-image.ps1
.\run-on-docker-local.ps1 [--flags]
.\run-on-docker-server.ps1 --ssh-string=<alias> [--flags]
```

When unsure:

```powershell
.\run-on-docker-local.ps1 --help
.\run-on-docker-server.ps1 --help
.\create-image.ps1 --help
```

### Step 6: Verify and report

- Report endpoint (host, port, URL), volume path, image tags, network name, internal port, cleanup performed.
- On failure: read script error output; scripts print help on validation exit.

## Safety rules

1. **Never** run build, deploy, or delete commands before completing Step 2 and Step 3.
2. **Never** pass `ssh` inside `--ssh-string` — config alias only.
3. **Never** use `--delete-image=yes` or `--delete-volume=yes` without explicit user confirmation.
4. **Never** hardcode a project name, port, volume path, or network into the skill or into new scripts; resolve from placeholders / project discovery.
5. **Always** create or edit all three scripts (`create-image.ps1`, `run-on-docker-local.ps1`, `run-on-docker-server.ps1`) so they match this contract.
6. **Always** implement `--help` with flags, examples, and notes in every script.
7. **Always** treat parameter default `null` as the resolved meanings in the parameter table (no ≠ omit incorrectly for server `--ssh-string`).
8. **Always** pick a free port inside `30000–32767` when `--internal-port` is null.
9. **Always** execute scripts from the directory that contains them.
10. **Always** show existing images/volumes before asking keep vs delete.

## Key facts & reference

### Prerequisites

- Docker CLI available (local and/or on remote host)
- `Dockerfile` and/or compose file in the target project
- Optional: `.docker/stack.manifest.json`
- Remote: SSH config alias in `~/.ssh/config`

### Scripts

| File | Purpose |
|------|---------|
| `create-image.ps1` | Build project image |
| `run-on-docker-local.ps1` | Local deploy |
| `run-on-docker-server.ps1` | Remote deploy over SSH |

### Run-script parameters

| Flag | Default | Resolved when null |
|------|---------|-------------------|
| `--ssh-string` | `null` | Local → local daemon; server → required alias |
| `--delete-image` | `null` | `no` |
| `--delete-volume` | `null` | `no` |
| `--internal-port` | `null` | Random `30000–32767` |
| `--volume-dir` | `null` | `<USER-PROFILE-NAME>/docker/<CONTAINER-NAME>` |
| `--volume-name` | `null` | `<CONTAINER-NAME>-volume` |
| `--network-name` | `null` | `<CONTAINER-NAME>-network` |

### Safe ports range

| Item | Value |
|------|-------|
| Min | `30000` |
| Max | `32767` |
| Selection | Random; confirm unused before binding |

### Troubleshooting

| Symptom | Check |
|---------|-------|
| Missing Dockerfile/compose | Run from the project that contains Docker config |
| `Docker CLI is not available` | Start Docker Desktop / daemon |
| Remote SSH fails | Alias in `~/.ssh/config`; value must not include `ssh` |
| Port in use | Re-pick from `30000–32767` or pass `--internal-port` |
| Old `run-on-docker.ps1` only | Replace/split into local + server scripts per this skill |
