---
description: Show workload by user (estimated hours)
---

Display estimated workload for each team member based on task estimates.

## Usage

```
/workload                 # All users
/workload --user sarah    # Single user
/workload --json          # JSON output
/workload --md            # Markdown output
```

## Steps

1. Parse arguments: `$ARGUMENTS`

2. Build and run command:
   - Default: `ctm workload`
   - User filter: `ctm workload --user <name>`
   - JSON: `ctm workload --json`
   - Markdown: `ctm workload --md`

3. Present the workload summary

## Output Shows

```
Workload Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
User          Tasks   Estimated
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
sarah         5       8h 30m
jim           3       4h 15m
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total         8       12h 45m
```

## Notes

- Only counts open tasks (ongoing, pending, suspended)
- Uses `-e` estimate values from tasks
- Tasks without estimates show as 0
- Useful for capacity planning

## Related Commands

- `/team` - Task count distribution
- `/stats` - Completion rates
