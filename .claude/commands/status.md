---
description: Quick status check - overdue tasks and today's priorities
---

Show a quick status overview at session start or anytime.

## Usage

`/status`

## Steps

1. Check for overdue tasks:
   ```bash
   ctm list task --overdue -s open
   ```

2. Check for high priority tasks due soon:
   ```bash
   ctm list task -d 3 -s open
   ```

3. Show summary statistics:
   ```bash
   ctm stats --days 7
   ```

## Output Format

```
📋 Task Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  3 overdue tasks need attention
🔴 2 high priority tasks due in 3 days
📊 Last 7 days: 12 created, 8 completed (67%)

Run /today for full task list
Run /overdue to see overdue items
```

## Use Cases

- Start of work session: Quick health check
- Before standups: Know what needs attention
- End of day: Check if anything slipped

## Recommended Hook

Add to your Claude Code settings for automatic check:

```json
{
  "hooks": {
    "session_start": {
      "command": "ctm list task --overdue -s open 2>/dev/null | head -5",
      "show_output": true
    }
  }
}
```

## Related Commands

- `/today` - Full today's task list
- `/overdue` - All overdue items
- `/stats` - Detailed statistics
