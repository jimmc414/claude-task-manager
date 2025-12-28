# claude-task-manager - Claude Code Integration

Claude-first task management with multi-tenant support. Use natural language or quick commands.

## Installation

```bash
# From crates.io
cargo install claude-task-manager

# The CLI command is 'ctm'
ctm task "My first task" today
```

## Quick Commands

### Core Commands
| Command | Purpose | Example |
|---------|---------|---------|
| `/today` | Today's tasks + overdue | Quick daily view |
| `/tasks` | Show open tasks | Overview of today/week |
| `/task` | Quick add | `/task review PR tomorrow -P high` |
| `/done` | Mark complete | `/done 1` |
| `/overdue` | Show overdue | What needs attention |
| `/status` | Quick health check | Overdue + priorities |

### Task Details
| Command | Purpose | Example |
|---------|---------|---------|
| `/show` | Detailed task view | `/show 3` |
| `/note` | Add note to task | `/note 3 "found root cause"` |
| `/claim` | Claim unassigned task | `/claim 5` |

### Project & Work
| Command | Purpose | Example |
|---------|---------|---------|
| `/work` | Open Claude in project | `/work 3` |
| `/standup` | Generate standup | Yesterday/Today/Blockers |

### Team & Reporting
| Command | Purpose | Example |
|---------|---------|---------|
| `/team` | Team task distribution | Who has what |
| `/workload` | Hours per person | Capacity planning |
| `/stats` | Completion statistics | Last 30 days |
| `/reminders` | Full task summary | Daily overview |

## Natural Language

Just talk to Claude. The ctm agent understands:

- "What tasks do I have today?"
- "Add a high priority task to review the PR by Friday"
- "Mark the first task done"
- "Reschedule task 2 to next week"
- "What did I complete today?"
- "Show me everything overdue"
- "Add a task for the myapp project"
- "Assign task 2 to sarah"
- "Show me the team workload"
- "Create a task from issue owner/repo#42"
- "Work on the first task" (uses `/work` if task has a project)

## Agent: ctm

Location: `.claude/agents/ctm.md`

Automatically invoked when you mention tasks, reminders, todo, deadlines, or schedules.

**Capabilities:**
- Natural language task management
- Priority and time estimates
- Notes and progress tracking
- Team assignment and workload
- GitHub integration (issues, PRs)
- Project-aware Claude sessions
- Reporting (team, workload, stats)

## Data Storage

Tasks are stored in SQLite at `~/.local/share/ctm/ctm.db`

Configuration (optional): `~/.config/ctm/config.json`

## Quick Reference

### Adding Items
```bash
ctm task "description" [timestr] [-c category]
ctm task "description" [timestr] -P high        # High priority
ctm task "description" [timestr] -e 2h          # 2 hour estimate
ctm task "description" [timestr] --for sarah    # Assign to user
ctm task "description" [timestr] -p myapp       # Link to project
ctm task --from-issue owner/repo#42             # From GitHub issue
ctm record "description" [-c category]
```

### Task Details
```bash
ctm show <index>                    # Full details
ctm note <index> "note text"        # Add note
ctm link <index> --issue repo#42    # Link issue
ctm link <index> --pr repo#43       # Link PR
ctm link <index> --commit abc123    # Link commit
ctm claim <index>                   # Claim task
```

### Listing
```bash
ctm list task              # Open tasks
ctm list task -s all       # All tasks
ctm list task --overdue    # Include overdue
ctm list task -u sarah     # Sarah's tasks
ctm list task --all-users  # Everyone's tasks
ctm list record -d 7       # Records from past week
```

### Managing
```bash
ctm done <index>                    # Complete task
ctm done <index> -c "note"          # Complete with comment
ctm done <index> --close-issue      # Complete + close GitHub issue
ctm update <index> -t "tomorrow"    # Reschedule
ctm update <index> -P high          # Set priority
ctm delete <index>                  # Delete item
```

### Team & Users
```bash
ctm user create sarah -d "Sarah Chen"   # Create user
ctm user list                           # List users
ctm ns create backend                   # Create namespace
ctm ns add-user backend sarah           # Add user to namespace
ctm --as sarah list task                # Act as user
ctm --ns backend list task              # Use namespace
```

### Reporting
```bash
ctm team                    # Task distribution
ctm workload                # Hours per person
ctm stats                   # Completion rates
ctm stats --days 7          # Last week
```

### Time Formats
- Relative: `today`, `tomorrow`, `next week`, `in 3 days`
- Absolute: `2024-01-15`, `jan 15`, `monday`
- With time: `tomorrow 3pm`, `monday 9:00`
- Recurring: `daily 9am`, `weekday 9am`, `weekly monday`, `monthly 1st`

### Status Values
| Status | Code | Description |
|--------|------|-------------|
| ongoing | 0 | In progress |
| done | 1 | Completed |
| cancelled | 2 | Cancelled |
| duplicate | 3 | Duplicate |
| suspended | 4 | On hold |
| pending | 6 | Not started |
| open | 254 | ongoing + pending + suspended |
| closed | 253 | done + cancelled + duplicate |
| all | 255 | All statuses |

## Project Configuration

To use the `/work` command, define projects in `~/.config/ctm/config.json`:

```json
{
  "terminal_profile": "Ubuntu",
  "projects": {
    "ctm": {
      "path": "/mnt/c/python/claude-task-manager"
    },
    "myapp": {
      "path": "/mnt/c/python/myapp",
      "conda_env": "myapp-env",
      "claude_flags": "--dangerously-skip-permissions"
    }
  }
}
```

**Project options:**
- `path` (required): Linux path to project directory
- `conda_env`: Conda environment to activate
- `claude_flags`: Additional Claude CLI flags
- `prompt_template`: Custom prompt template (use `{content}` for task content)

## Session Start Hook

This project includes a SessionStart hook (`.claude/settings.json`) that automatically shows:
- Overdue tasks needing attention
- Tasks due today

The hook runs when you start or resume a Claude Code session in this project.
