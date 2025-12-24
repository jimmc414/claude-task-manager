---
description: Quick add a task (e.g., /task review PR tomorrow)
---

Add a new task using the provided arguments.

Parse the input as: `description [timestr] [-c category]`

Examples:
- `/task review PR tomorrow` → `tascli task "review PR" tomorrow`
- `/task "submit report" friday -c work` → `tascli task "submit report" friday -c work`
- `/task standup daily 9am` → `tascli task "standup" "daily 9am"`

If no timestr is provided, default to "today".

Run the tascli command and confirm the task was added.
