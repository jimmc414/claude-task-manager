---
description: Show team task distribution
---

Display how tasks are distributed across team members.

## Usage

```
/team              # Text output
/team --json       # JSON output
/team --md         # Markdown output
```

## Steps

1. Parse arguments: `$ARGUMENTS`

2. Run the appropriate command:
   - Default: `ctm team`
   - JSON: `ctm team --json`
   - Markdown: `ctm team --md`

3. Present the team overview

## Output Shows

| User | Open | Done | Total |
|------|------|------|-------|
| sarah | 5 | 12 | 17 |
| jim | 3 | 8 | 11 |
| unassigned | 2 | - | 2 |

## Use Cases

- Sprint planning: See who has capacity
- Standups: Quick overview of team workload
- Reporting: Export to Slack/docs with `--md`
- Integrations: Use `--json` for tooling

## Related Commands

- `/workload` - See estimated hours per person
- `/stats` - Task completion statistics
