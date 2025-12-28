---
description: Re-inject repository context (use when Claude forgets project details)
---

# claude-task-manager Repository Context

Use this skill when Claude seems to have lost context about this project.

## Project Overview

**claude-task-manager (ctm)** is a Claude-first task management CLI written in Rust.

- **Binary:** `ctm`
- **Database:** SQLite at `~/.local/share/ctm/ctm.db`
- **Config:** `~/.config/ctm/config.json`
- **Schema version:** 5 (multi-tenant)

## Architecture

```
src/
├── main.rs              # Entry point, CLI setup
├── args/
│   ├── parser.rs        # Clap derive macros, all commands
│   ├── timestr.rs       # Time parsing (today, tomorrow, etc.)
│   ├── priority.rs      # Priority parsing (high/normal/low)
│   └── estimate.rs      # Estimate parsing (2h, 30m)
├── db/
│   ├── conn.rs          # Connection, migrations, schema v5
│   ├── crud.rs          # Insert/update/query items
│   ├── item.rs          # Item struct, ItemQuery builder
│   ├── user.rs          # User CRUD
│   ├── namespace.rs     # Namespace CRUD
│   ├── note.rs          # Task notes
│   ├── link.rs          # Task links (commits, issues, PRs)
│   └── cache.rs         # Index cache for list commands
├── actions/
│   ├── handler.rs       # Routes commands to handlers
│   ├── addition.rs      # task, record commands
│   ├── modify.rs        # done, update, delete commands
│   ├── show.rs          # Detailed task view
│   ├── note.rs          # Note command handler
│   ├── claim.rs         # Claim command handler
│   ├── link.rs          # Link command handler
│   ├── user.rs          # User command handlers
│   ├── namespace.rs     # Namespace command handlers
│   ├── reporting.rs     # team, workload, stats commands
│   └── list/            # List command handlers
├── context/
│   ├── identity.rs      # User/namespace resolution
│   └── mod.rs           # Context struct
├── github/
│   ├── api.rs           # gh CLI wrapper
│   └── mod.rs
├── config/
│   └── mod.rs           # Config file parsing
└── utils/
    └── path.rs          # WSL path conversion
```

## Key Concepts

### Multi-Tenant Model
- **Users:** Team members tracked by the manager
- **Namespaces:** Organize tasks by project/team
- **Identity:** `--as user` / `--ns namespace` flags
- **Resolution:** Flag → ENV → system $USER

### Task Fields
```rust
pub struct Item {
    id, action, category, content,
    create_time, target_time, modify_time, status,
    cron_schedule, human_schedule,      // Recurring
    reminder_days, project,              // Reminders & projects
    owner_id, assignee_id, namespace_id, // Multi-tenant
    priority, estimate_minutes,          // Planning
    github_issue,                        // GitHub link
}
```

### Database Tables (Schema v5)
- `items` - Tasks and records
- `users` - Team members
- `namespaces` - Task groupings
- `user_namespaces` - Membership with roles
- `task_notes` - Append-only notes
- `task_links` - Commits, issues, PRs, URLs
- `audit_log` - Change tracking
- `cache` - Index mapping for commands

## Commands

### Core
- `task` - Add task with deadline
- `record` - Add log entry
- `done` - Complete task
- `update` - Modify task/record
- `delete` - Remove item
- `list task/record` - List items

### Task Details
- `show` - Detailed view
- `note` - Add note
- `link` - Attach commit/issue/PR
- `claim` - Take ownership

### Team
- `user create/list/delete`
- `ns create/list/delete/switch/add-user/remove-user/members`
- `team` - Task distribution
- `workload` - Hours per person
- `stats` - Completion rates

### GitHub
- `--from-issue owner/repo#42` - Create from issue
- `--close-issue` - Close linked issue on done

## Slash Commands

Located in `.claude/commands/`:
```
today, tasks, task, done, overdue, status,
show, note, claim, work, standup,
team, workload, stats, reminders
```

## Testing

```bash
cargo test              # Run all 143 tests
cargo build --release   # Build release binary
```

## Files to Read for Context

If you need more detail:
- `src/args/parser.rs` - All CLI commands and flags
- `src/db/conn.rs` - Schema migrations
- `src/actions/handler.rs` - Command routing
- `.claude/CLAUDE.md` - User-facing documentation
