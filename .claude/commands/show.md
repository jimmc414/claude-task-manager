---
description: Show detailed view of a task (e.g., /show 3)
---

Show detailed information about a task including notes, links, and history.

## Usage

`/show <index>` where index is the task number from `/tasks` or `/today`

## Steps

1. Parse the index from arguments: `$ARGUMENTS`

2. If no index provided:
   - Run `ctm list task -s open` to show current tasks
   - Ask user which task they want to see details for

3. If index provided:
   - Run `ctm show <index>` to get full task details

## Output Includes

- Task content and category
- Priority level (HIGH/normal/low)
- Owner and assignee
- Due date and estimate
- Project association
- All timestamped notes
- Linked commits, issues, PRs, URLs
- Creation and modification dates

## Example

User: `/show 3`

Shows:
```
Task #3: Fix login timeout bug
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Priority:   HIGH
Assignee:   sarah
Project:    myapp
Due:        2025-01-03 (2 days)
Estimate:   2h

Notes:
  [Dec 26, 10:30] User reported 5s load times
  [Dec 26, 14:15] Tried redis, too slow

Links:
  [issue] myapp#42 - Login timeout in production
  [commit] a1b2c3d
```
