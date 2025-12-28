---
description: Show task statistics (completion rates, overdue analysis)
---

Display task statistics including completion rates and overdue analysis.

## Usage

```
/stats              # Last 30 days (default)
/stats --days 7     # Last week
/stats --json       # JSON output
/stats --md         # Markdown output
```

## Steps

1. Parse arguments: `$ARGUMENTS`

2. Build and run command:
   - Default: `ctm stats`
   - Custom period: `ctm stats --days <N>`
   - JSON: `ctm stats --json`
   - Markdown: `ctm stats --md`

3. Present the statistics

## Output Shows

```
Task Statistics (last 30 days)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Created:        25
Completed:      18
Completion:     72%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Overdue:        3
High Priority:  2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
By Status:
  ongoing       5
  pending       2
  done          18
```

## Use Cases

- Sprint retrospectives
- Weekly reports
- Identifying bottlenecks (high overdue)
- Tracking team velocity

## Related Commands

- `/team` - Task distribution
- `/workload` - Estimated hours
- `/overdue` - List overdue tasks
