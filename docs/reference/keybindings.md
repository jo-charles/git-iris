# Studio Keybinding Reference

Complete keyboard shortcut reference for Iris Studio.

## Global Keybindings

Work in all modes and panels.

| Key         | Action                         |
| ----------- | ------------------------------ |
| `q`         | Quit Studio                    |
| `Ctrl+C`    | Quit Studio                    |
| `?`         | Show help overlay              |
| `/`         | Open chat with Iris            |
| `Tab`       | Next panel                     |
| `Shift+Tab` | Previous panel                 |
| `Esc`       | Close modal / Cancel operation |

## Mode Switching

Switch between Studio modes with Shift+Letter.

| Key       | Mode               |
| --------- | ------------------ |
| `Shift+E` | Explore mode       |
| `Shift+C` | Commit mode        |
| `Shift+R` | Review mode        |
| `Shift+P` | PR mode            |
| `Shift+L` | Changelog mode     |
| `Shift+N` | Release Notes mode |
| `Shift+S` | Settings modal     |

## Explore Mode

Navigate and explore your codebase.

### File Tree Navigation

| Key               | Action                       |
| ----------------- | ---------------------------- |
| `j` / `Down`      | Next item                    |
| `k` / `Up`        | Previous item                |
| `h` / `Left`      | Collapse directory           |
| `l` / `Right`     | Expand directory             |
| `g` / `Home`      | First item                   |
| `G` / `End`       | Last item                    |
| `Ctrl+D` / `PgDn` | Page down                    |
| `Ctrl+U` / `PgUp` | Page up                      |
| `Enter`           | Open file / Toggle directory |

### Explore Actions

| Key | Action                     |
| --- | -------------------------- |
| `w` | Ask "Why does this exist?" |
| `H` | Toggle heat map view       |
| `o` | Open in external editor    |

## Commit Mode

Generate and edit commit messages.

### File Panel (Left)

| Key               | Action                    |
| ----------------- | ------------------------- |
| `j` / `Down`      | Next file                 |
| `k` / `Up`        | Previous file             |
| `h` / `Left`      | Collapse directory        |
| `l` / `Right`     | Expand directory          |
| `g` / `Home`      | First file                |
| `G` / `End`       | Last file                 |
| `Ctrl+D` / `PgDn` | Page down                 |
| `Ctrl+U` / `PgUp` | Page up                   |
| `Enter`           | Select file and view diff |
| `s`               | Stage file                |
| `u`               | Unstage file              |
| `a`               | Stage all files           |
| `U`               | Unstage all files         |
| `A`               | Toggle show all files     |

### Message Panel (Center)

| Key     | Action                           |
| ------- | -------------------------------- |
| `e`     | Edit message                     |
| `p`     | Open preset selector             |
| `r`     | Regenerate message               |
| `R`     | Reset to original                |
| `i`     | Custom instructions              |
| `g`     | Open emoji selector              |
| `E`     | Quick toggle emoji (None ↔ Auto) |
| `Enter` | Commit with message              |
| `Left`  | Previous message variant         |
| `Right` | Next message variant             |
| `y`     | Copy message to clipboard        |

### Diff Panel (Right)

| Key               | Action               |
| ----------------- | -------------------- |
| `j` / `Down`      | Scroll down one line |
| `k` / `Up`        | Scroll up one line   |
| `Ctrl+D` / `PgDn` | Scroll down page     |
| `Ctrl+U` / `PgUp` | Scroll up page       |
| `]`               | Next hunk            |
| `[`               | Previous hunk        |
| `n`               | Next file            |
| `p`               | Previous file        |

### Edit Mode (Message Editor)

When editing a commit message:

| Key         | Action           |
| ----------- | ---------------- |
| `Esc`       | Exit edit mode   |
| `Ctrl+S`    | Save changes     |
| `Arrows`    | Move cursor      |
| `Home`      | Start of line    |
| `End`       | End of line      |
| `Backspace` | Delete character |
| `Delete`    | Delete forward   |
| `Enter`     | New line         |

## Review Mode

Code review generation and viewing.

### Review Panel

| Key               | Action                   |
| ----------------- | ------------------------ |
| `j` / `Down`      | Scroll down              |
| `k` / `Up`        | Scroll up                |
| `Ctrl+D` / `PgDn` | Page down                |
| `Ctrl+U` / `PgUp` | Page up                  |
| `r`               | Regenerate review        |
| `y`               | Copy review to clipboard |
| `f`               | Change from ref          |
| `t`               | Change to ref            |

## PR Mode

Pull request description generation.

### PR Panel

| Key               | Action                    |
| ----------------- | ------------------------- |
| `j` / `Down`      | Scroll down               |
| `k` / `Up`        | Scroll up                 |
| `Ctrl+D` / `PgDn` | Page down                 |
| `Ctrl+U` / `PgUp` | Page up                   |
| `r`               | Regenerate PR description |
| `y`               | Copy to clipboard         |
| `b`               | Change base branch        |
| `t`               | Change target ref         |

## Changelog Mode

Changelog generation.

### Changelog Panel

| Key               | Action                   |
| ----------------- | ------------------------ |
| `j` / `Down`      | Scroll down              |
| `k` / `Up`        | Scroll up                |
| `Ctrl+D` / `PgDn` | Page down                |
| `Ctrl+U` / `PgUp` | Page up                  |
| `r`               | Regenerate changelog     |
| `y`               | Copy to clipboard        |
| `f`               | Change from ref          |
| `t`               | Change to ref            |
| `u`               | Update CHANGELOG.md file |

## Release Notes Mode

Release notes generation.

### Release Notes Panel

| Key               | Action                   |
| ----------------- | ------------------------ |
| `j` / `Down`      | Scroll down              |
| `k` / `Up`        | Scroll up                |
| `Ctrl+D` / `PgDn` | Page down                |
| `Ctrl+U` / `PgUp` | Page up                  |
| `r`               | Regenerate release notes |
| `y`               | Copy to clipboard        |
| `f`               | Change from ref          |
| `t`               | Change to ref            |

## Modal Keybindings

### Settings Modal

| Key          | Action           |
| ------------ | ---------------- |
| `Esc`        | Close settings   |
| `j` / `Down` | Next setting     |
| `k` / `Up`   | Previous setting |
| `Enter`      | Edit setting     |
| `s`          | Save and close   |

### Preset Selector

| Key          | Action          |
| ------------ | --------------- |
| `Esc`        | Cancel          |
| `j` / `Down` | Next preset     |
| `k` / `Up`   | Previous preset |
| `Enter`      | Select preset   |
| Type         | Filter presets  |

### Emoji Selector

| Key          | Action         |
| ------------ | -------------- |
| `Esc`        | Cancel         |
| `j` / `Down` | Next emoji     |
| `k` / `Up`   | Previous emoji |
| `Enter`      | Select emoji   |
| `n`          | Select "None"  |
| `a`          | Select "Auto"  |
| Type         | Search emojis  |

### Chat Panel

| Key          | Action          |
| ------------ | --------------- |
| `Esc`        | Close chat      |
| `j` / `Down` | Scroll down     |
| `k` / `Up`   | Scroll up       |
| `Enter`      | Send message    |
| Type         | Compose message |

## Quick Reference by Task

### Quick Commit Workflow

```
1. Shift+C           → Switch to Commit mode
2. r                 → Regenerate if needed
3. e                 → Edit message
4. Esc               → Exit edit mode
5. Enter             → Commit
```

### Quick Review Workflow

```
1. Shift+R           → Switch to Review mode
2. f / t             → Set refs if needed
3. r                 → Generate review
4. y                 → Copy to clipboard
```

### Quick PR Workflow

```
1. Shift+P           → Switch to PR mode
2. b / t             → Set branches if needed
3. r                 → Generate PR description
4. y                 → Copy to clipboard
```

### File Staging Workflow

```
1. Shift+C           → Commit mode
2. Tab               → Focus file panel
3. j/k               → Navigate files
4. s                 → Stage file
5. a                 → Stage all (or U for unstage all)
```

### Exploring Codebase

```
1. Shift+E           → Explore mode
2. j/k               → Navigate
3. l                 → Expand directory
4. w                 → Ask "why" about file
5. o                 → Open in editor
```

## Vim-Style Navigation

Most panels support Vim-style keybindings:

| Key      | Action          |
| -------- | --------------- |
| `h`      | Left / Collapse |
| `j`      | Down / Next     |
| `k`      | Up / Previous   |
| `l`      | Right / Expand  |
| `g`      | Go to top       |
| `G`      | Go to bottom    |
| `Ctrl+D` | Half page down  |
| `Ctrl+U` | Half page up    |

## Arrow Key Navigation

All Vim bindings also work with arrow keys:

| Key     | Equivalent |
| ------- | ---------- |
| `Up`    | `k`        |
| `Down`  | `j`        |
| `Left`  | `h`        |
| `Right` | `l`        |
| `Home`  | `g`        |
| `End`   | `G`        |
| `PgUp`  | `Ctrl+U`   |
| `PgDn`  | `Ctrl+D`   |

## Tips

### Efficient Panel Navigation

- Use `Tab` to cycle through panels in order
- Use `Shift+Tab` to reverse cycle
- Most actions work on the focused panel

### Speed Tips

- Use `j`/`k` for single-line movement
- Use `Ctrl+D`/`Ctrl+U` for page-sized jumps
- Use `g`/`G` to jump to start/end
- Press `r` to regenerate content quickly

### Learn As You Go

- Press `?` anytime to see context-sensitive help
- Help overlay shows only relevant keybindings for current mode

### Chat Integration

- Press `/` in any mode to chat with Iris
- Iris can update content directly via tools
- Continue working while Iris thinks
