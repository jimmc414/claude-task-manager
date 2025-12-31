---
description: Spawn a new Claude Code session in a new Windows Terminal tab (WSL/Ubuntu)
---

# Spawn Claude Session

Opens a new Windows Terminal tab with Claude Code running in a specified directory.

## Usage

```
/spawn-claude [linux_path] ["prompt"]
```

**Arguments:**
- `linux_path` - Linux/WSL path to open (default: current directory)
- `"prompt"` - Optional quoted starting prompt for Claude

## Examples

```
/spawn-claude
/spawn-claude /mnt/c/python/myproject
/spawn-claude /mnt/c/python/myproject "Help me fix the login bug"
```

## Implementation

When the user invokes `/spawn-claude`, execute these steps:

### 1. Parse Arguments

Extract from `$ARGUMENTS`:
- If empty: use current working directory
- If one unquoted arg: use as linux_path
- If quoted string at end: use as prompt

### 2. Convert Linux Path to Windows Path

```
/mnt/c/python/myproject → C:\python\myproject
/mnt/d/projects/foo     → D:\projects\foo
```

**Conversion logic:**
- Extract drive letter from position 5 (e.g., `/mnt/c/...` → `c`)
- Uppercase the drive letter
- Take the rest of the path after `/mnt/X/`
- Replace `/` with `\`
- Format as `{DRIVE}:\{rest}`

### 3. Execute Spawn Command

Run this exact command (substituting variables):

```bash
/init /mnt/c/Windows/System32/cmd.exe /c "wt.exe -p Ubuntu -d {WIN_PATH} wsl.exe -e bash -c \"export PATH=\$HOME/.local/bin:\$PATH && claude {PROMPT}\""
```

**Example with real values:**
```bash
/init /mnt/c/Windows/System32/cmd.exe /c "wt.exe -p Ubuntu -d C:\python\myproject wsl.exe -e bash -c \"export PATH=\$HOME/.local/bin:\$PATH && claude\""
```

**With a prompt:**
```bash
/init /mnt/c/Windows/System32/cmd.exe /c "wt.exe -p Ubuntu -d C:\python\myproject wsl.exe -e bash -c \"export PATH=\$HOME/.local/bin:\$PATH && claude \\\"Fix the login bug\\\"\""
```

### 4. Confirm to User

```
Spawned Claude session in {linux_path}
```

## Technical Details

| Component | Purpose |
|-----------|---------|
| `/init` | WSL interop wrapper - bypasses fmask execute bit stripping |
| `wt.exe -p Ubuntu` | Windows Terminal with Ubuntu profile |
| `-d C:\path` | Working directory (must be Windows format) |
| `wsl.exe -e bash -c "..."` | Execute command in WSL |
| `export PATH=...` | Ensures ~/.local/bin is in PATH for Claude |

## Error Handling

- **Path not under /mnt/**: "Cannot convert path: {path}. Only /mnt/X/... paths supported."
- **Directory doesn't exist**: "Directory not found: {path}"
- **Not in WSL**: "This command only works in WSL (Windows Subsystem for Linux)"

## Customization

To use a different terminal profile, modify the `-p Ubuntu` part:
- `-p "Windows PowerShell"` for PowerShell
- `-p "Command Prompt"` for CMD
- `-p "Your Custom Profile"` for custom profiles
