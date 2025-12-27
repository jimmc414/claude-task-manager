# Resume Prompt: CTM Multi-Tenant Implementation

Use this prompt to resume work after context compaction.

---

## Context

I'm implementing a major feature set for claude-task-manager (ctm), a Rust CLI task manager. **Phases 1-3 are complete, ready for Phase 4.**

**Project location:** `/mnt/c/python/claude-task-manager`

## Current Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Schema v5 Migration | COMPLETE |
| 2 | Identity Context System | COMPLETE |
| 3 | User/Namespace Commands | COMPLETE |
| 4 | Task Enhancements | **NEXT** |
| 5 | Notes/Show/Claim/Link | Not started |
| 6 | Reporting Commands | Not started |
| 7 | GitHub Integration | Not started |
| 8 | /work + /standup | Not started |

**All 77 tests pass.**

## What Was Completed

### Phase 1: Schema v5
- New tables: `users`, `namespaces`, `user_namespaces`, `task_links`, `task_notes`, `audit_log`
- New `items` columns: `owner_id`, `assignee_id`, `namespace_id`, `priority`, `estimate_minutes`, `github_issue`
- Auto-setup creates default user + namespace on first run
- Files: `src/db/conn.rs`, `src/db/item.rs`, `src/db/crud.rs`

### Phase 2: Identity Context
- `Context` struct resolves current user/namespace
- Global `--as` and `--ns` CLI flags added
- Files: `src/context/mod.rs`, `src/context/identity.rs`, `src/main.rs`, `src/actions/handler.rs`, `src/args/parser.rs`

### Phase 3: User/Namespace Commands
- User struct + CRUD: `src/db/user.rs`
- Namespace struct + CRUD: `src/db/namespace.rs`
- User command handlers: `src/actions/user.rs`
- Namespace command handlers: `src/actions/namespace.rs`
- Commands: `ctm user create/list/delete`, `ctm ns create/list/delete/switch/add-user/remove-user/members`

## Key Files

- **Full plan:** `/home/jim/.claude/plans/modular-fluttering-aurora.md`
- **Checkpoint:** `.claude/checkpoint.md`
- **Schema migrations:** `src/db/conn.rs`
- **CLI commands:** `src/args/parser.rs`
- **Command handlers:** `src/actions/handler.rs`
- **Context:** `src/context/identity.rs`

## To Resume

```
1. Read checkpoint: .claude/checkpoint.md
2. Start Phase 4: Task Enhancements
   - Add src/args/priority.rs (parse high/normal/low)
   - Add src/args/estimate.rs (parse 2h/30m/1h30m)
   - Modify src/args/parser.rs - Add -P, -e, --for, --from-issue flags
   - Modify src/db/crud.rs - Add assignee/namespace filtering
   - Modify src/actions/addition.rs - Handle priority, estimate, assignee
```

## Design Decisions (Don't Re-Ask)

- Single DB, multi-tenant (not separate DBs per namespace)
- Single manager model (one person runs ctm, tracks team)
- Roles are per-namespace (owner/admin/member/viewer)
- Task ownership: owner_id (accountable) + assignee_id (working)
- Unassigned tasks allowed, claimable via `ctm claim`
- Identity: --as flag → CTM_USER env → system $USER
- Auto-setup on first run (create user + default namespace)
- GitHub: use `gh` CLI wrapper, not HTTP API
- Future-proof for concurrent access (proper IDs, audit trails)
- Reports support --json and --md output flags

## Phase 4 Commands to Implement

```bash
# Task creation with enhancements
ctm task "description" [timestr] [-P high|normal|low] [-e 2h] [--for user]
ctm task --from-issue owner/repo#42  # Create from GitHub issue

# Task listing with filters
ctm list task [--user sarah] [--all-users]
```

## Global Flags (Already Implemented)

```bash
ctm --as sarah list task    # Act as user
ctm --ns work list task     # Use namespace
```
