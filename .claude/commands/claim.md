---
description: Claim an unassigned task (e.g., /claim 5)
---

Take ownership of an unassigned task and assign it to yourself.

## Usage

`/claim <index>`

## Steps

1. Parse the index from arguments: `$ARGUMENTS`

2. If no index provided:
   - Run `ctm list task -s open --all-users` to show tasks
   - Highlight unassigned tasks
   - Ask user which task to claim

3. If index provided:
   - Run `ctm claim <index>`
   - Confirm the task is now assigned to you

## When to Use

- Picking up work from the team backlog
- Taking over an unassigned task
- Volunteering for open work items

## Example

```
User: /claim 5

Output:
Claimed task #5: "Review API documentation"
Now assigned to: jim
```

## Notes

- Only works on unassigned tasks
- Sets you as both owner and assignee
- Use `ctm list task --all-users` to see unassigned tasks
- Unassigned tasks show as "unassigned" in yellow in task lists
