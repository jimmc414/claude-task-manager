---
description: Add a note to a task (e.g., /note 3 "investigated root cause")
---

Add a timestamped note to a task to track progress.

## Usage

`/note <index> "<note text>"`

## Steps

1. Parse arguments: `$ARGUMENTS`
   - First argument: task index
   - Remaining text: note content

2. If no index provided:
   - Run `ctm list task -s open` to show current tasks
   - Ask user which task and what note to add

3. If index and note provided:
   - Run `ctm note <index> "<note text>"`
   - Confirm the note was added

4. Optionally show updated task with `ctm show <index>`

## Examples

```
/note 3 "Investigated root cause - DB connection timeout"
/note 5 "Waiting on API team response"
/note 1 "PR #42 submitted for review"
```

## Notes Are

- Timestamped automatically
- Attributed to current user
- Visible in `ctm show <index>` output
- Append-only (cannot be edited)

## Tips

Use notes to:
- Track investigation progress
- Document blockers
- Record decisions made
- Log communications about the task
