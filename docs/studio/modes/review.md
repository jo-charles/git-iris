# Review Mode

**Review Mode** provides AI-powered code reviews analyzing security, performance, best practices, and potential bugs. Get comprehensive feedback before committing or creating PRs.

## When to Use Review Mode

- **Pre-commit quality checks**: Review changes before committing
- **PR preparation**: Catch issues before reviewers see them
- **Learning best practices**: Understand why certain patterns are recommended
- **Security audit**: Identify potential vulnerabilities
- **Performance analysis**: Find bottlenecks and inefficiencies

## Panel Layout

```
┌─────────────┬──────────────────────┬─────────────────────┐
│ Changed     │   Review Output      │   Diff View         │
│ Files       │                      │                     │
│             │ # Code Review        │ @@ -10,6 +10,8 @@  │
│ ● iris.rs M │                      │  impl ReviewMode {  │
│   state.rs M│ ## Security          │ +  pub from_ref     │
│   review.rs │                      │ +  pub to_ref       │
│             │ ⚠️ Potential SQL     │                     │
│ Reviewing:  │ injection on line 45 │ File: iris.rs      │
│ main..HEAD  │                      │ Hunk 2/5           │
│             │ ## Performance       │                     │
│ [f] From    │                      │ [[] Previous hunk   │
│ [t] To      │ ✓ Efficient use of   │ []] Next hunk       │
│ [r] Review  │ iterators            │ [n] Next file       │
└─────────────┴──────────────────────┴─────────────────────┘
```

### Left Panel: Changed Files

- Files with changes in selected range
- Directory tree structure
- Git status indicators
- Ref selection (from/to)

### Center Panel: Review Output

- Markdown-formatted review
- Categorized by dimension (Security, Performance, etc.)
- Severity indicators (✓, ⚠️, ✗)
- Line number references
- Scrollable multi-section output

### Right Panel: Diff View

- Unified diff for context
- Syntax-highlighted changes
- Hunk navigation
- Multi-file diff

## Essential Keybindings

### File List (Left Panel)

| Key                            | Action                             |
| ------------------------------ | ---------------------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>    | Select next file                   |
| <kbd>k</kbd> / <kbd>↑</kbd>    | Select previous file               |
| <kbd>h</kbd> / <kbd>←</kbd>    | Collapse directory                 |
| <kbd>l</kbd> / <kbd>→</kbd>    | Expand directory                   |
| <kbd>Enter</kbd>               | Load file diff (focus right panel) |
| <kbd>f</kbd>                   | Select "from" ref (base)           |
| <kbd>t</kbd>                   | Select "to" ref (target)           |
| <kbd>r</kbd>                   | Generate review                    |
| <kbd>g</kbd> / <kbd>Home</kbd> | Jump to first file                 |
| <kbd>G</kbd> / <kbd>End</kbd>  | Jump to last file                  |

### Review Output (Center Panel)

| Key                                 | Action                   |
| ----------------------------------- | ------------------------ |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down              |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up                |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down                |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up                  |
| <kbd>g</kbd> / <kbd>Home</kbd>      | Jump to top              |
| <kbd>G</kbd> / <kbd>End</kbd>       | Jump to bottom           |
| <kbd>r</kbd>                        | Regenerate review        |
| <kbd>Shift+R</kbd>                  | Reset (clear review)     |
| <kbd>y</kbd>                        | Copy review to clipboard |

### Diff View (Right Panel)

| Key                                 | Action                |
| ----------------------------------- | --------------------- |
| <kbd>j</kbd> / <kbd>↓</kbd>         | Scroll down           |
| <kbd>k</kbd> / <kbd>↑</kbd>         | Scroll up             |
| <kbd>[</kbd>                        | Jump to previous hunk |
| <kbd>]</kbd>                        | Jump to next hunk     |
| <kbd>n</kbd>                        | Jump to next file     |
| <kbd>p</kbd>                        | Jump to previous file |
| <kbd>Ctrl+d</kbd> / <kbd>PgDn</kbd> | Page down             |
| <kbd>Ctrl+u</kbd> / <kbd>PgUp</kbd> | Page up               |

## Ref Selection

Press <kbd>f</kbd> or <kbd>t</kbd> to select **from/to refs**:

```
┌─────────────────────────────────────────────┐
│  Select Base Branch (from)             [Esc] │
├─────────────────────────────────────────────┤
│  Type to filter...                           │
│                                              │
│  main                                    ◀  │
│  master                                      │
│  origin/main                                 │
│  develop                                     │
│  feature/emoji-selector                      │
│  origin/feature/emoji-selector               │
│                                              │
│  [j/k] Navigate  [Enter] Select              │
└─────────────────────────────────────────────┘
```

### Default Refs

- **From**: `main` (or `master`)
- **To**: `HEAD` (current state)

### Common Ranges

| From          | To       | Reviews                       |
| ------------- | -------- | ----------------------------- |
| `main`        | `HEAD`   | All changes on current branch |
| `v1.0.0`      | `v1.1.0` | Changes between releases      |
| `abc123f`     | `HEAD`   | Changes since specific commit |
| `origin/main` | `HEAD`   | Local changes not pushed      |

## Review Dimensions

Iris analyzes code across multiple dimensions:

### 🔒 Security

- SQL injection risks
- XSS vulnerabilities
- Authentication/authorization issues
- Cryptographic weaknesses
- Input validation gaps
- Secret exposure

### ⚡ Performance

- Algorithmic complexity (O(n²) loops, etc.)
- Memory leaks
- Inefficient queries
- Unnecessary allocations
- Hot path optimizations
- Caching opportunities

### ✨ Best Practices

- Code organization
- Naming conventions
- Error handling patterns
- Testing coverage
- Documentation quality
- SOLID principles

### 🐛 Potential Bugs

- Null/undefined dereferencing
- Off-by-one errors
- Race conditions
- Resource leaks
- Type mismatches
- Edge case handling

### 🧹 Code Quality

- Duplication (DRY violations)
- Complexity (cyclomatic, cognitive)
- Readability
- Maintainability
- Testability

### 📚 Documentation

- Missing docstrings
- Outdated comments
- Unclear naming
- API documentation
- README accuracy

## Review Output Format

```markdown
# Code Review

## Summary

Reviewed 3 files with 145 additions and 32 deletions.
Overall: ✓ Good with minor improvements suggested.

## Security

✓ No critical security issues found.

⚠️ Consider validating user input in `src/handlers/commit.rs:45`
Current implementation trusts all input from message editor.
Suggestion: Add length limits and sanitize special characters.

## Performance

✓ Efficient use of iterators throughout.

✗ Potential O(n²) issue in `src/studio/state.rs:123`
Nested loop scans all files for each commit.
Suggestion: Index files by commit ID for O(1) lookups.

## Best Practices

✓ Follows Rust idioms well.
✓ Good error handling with Result types.

⚠️ Consider extracting `sync_file_selection()` to a trait
This pattern is repeated in 3 different handlers.
Suggestion: Create a `FileSync` trait for reuse.

## Potential Bugs

✓ No obvious bugs detected.

## Code Quality

Overall quality: High

⚠️ Function `handle_commit_key` is complex (cyclomatic: 12)
Consider splitting into smaller functions.

## Documentation

⚠️ Missing docstring for `EmojiMode::Custom`
Add documentation explaining when to use this variant.

## Recommendations

1. Add input validation to message editor
2. Optimize file lookup in state management
3. Extract file sync logic to shared trait
4. Document EmojiMode variants

Estimated time to address: 2-3 hours
```

### Severity Indicators

- **✓ (Green)**: Good, no issues
- **⚠️ (Yellow)**: Warning, minor improvement
- **✗ (Red)**: Error, significant issue

## Workflow Examples

### Example 1: Pre-Commit Review

**Goal**: Check changes before committing

1. Make code changes
2. Switch to Review mode (<kbd>Shift+R</kbd>)
3. Default refs are `main..HEAD` (perfect for feature branch)
4. Press <kbd>r</kbd> to generate review
5. Read through each dimension
6. Press <kbd>/</kbd> to chat: "Explain the O(n²) issue you found"
7. Fix issues in your editor
8. Press <kbd>r</kbd> to review again
9. When ✓ across all dimensions, switch to Commit mode (<kbd>Shift+C</kbd>)

### Example 2: PR Preparation

**Goal**: Get feedback before creating pull request

1. Finish feature branch
2. Switch to Review mode
3. Press <kbd>f</kbd> to select from ref: `origin/main`
4. Press <kbd>t</kbd> to select to ref: `HEAD`
5. Press <kbd>r</kbd> to generate review
6. Address all ✗ and ⚠️ items
7. Press <kbd>y</kbd> to copy review to clipboard
8. Paste into PR description as "Self-Review" section

### Example 3: Release Audit

**Goal**: Review all changes between versions

1. Open Review mode
2. Press <kbd>f</kbd> → select `v1.0.0`
3. Press <kbd>t</kbd> → select `v1.1.0`
4. Press <kbd>r</kbd> to generate review
5. Focus on Security dimension (scroll to section)
6. Document any breaking changes found
7. Copy review with <kbd>y</kbd>
8. Use as release audit documentation

### Example 4: Learning from Reviews

**Goal**: Understand best practices by asking Iris

1. Generate review (<kbd>r</kbd>)
2. See warning: "Consider using iterators instead of for loops"
3. Press <kbd>/</kbd> to open chat
4. Ask: "Show me how to rewrite that loop with iterators"
5. Iris provides example code
6. Press <kbd>Shift+E</kbd> to switch to Explore mode
7. Navigate to the file
8. Press <kbd>w</kbd> on the loop to understand its history
9. Refactor based on learning

### Example 5: Security-Focused Review

**Goal**: Audit for security issues only

1. Generate review
2. Scroll to Security section with <kbd>j</kbd>/<kbd>k</kbd>
3. For each ⚠️ or ✗:
   - Note the line number
   - Press <kbd>Tab</kbd> to focus diff panel
   - Navigate to that line with <kbd>j</kbd>/<kbd>k</kbd>
   - Press <kbd>/</kbd> to ask: "How would you exploit this?"
4. Fix vulnerabilities
5. Press <kbd>r</kbd> to verify fixes

## Special Features

### Context-Aware Analysis

Iris reads project documentation to understand:

- **Architecture patterns**: From CLAUDE.md
- **Coding standards**: From README or CONTRIBUTING.md
- **Agent behavior**: From AGENTS.md

Reviews are customized to your project's standards.

### Diff-Integrated Review

Line number references in review correspond to diff view:

```
Review says:        Diff shows:
⚠️ Line 45          @@ -40,6 +40,10 @@
                     42  pub fn new() {
                     43    Self {
                     44      mode: Auto,
                     45      editing: false,  ← Line 45
```

Press <kbd>Tab</kbd> to jump between review and diff.

### Chat Integration

Ask follow-up questions about review findings:

```
You: Why is the O(n²) loop a problem here?

Iris: The nested loop in state.rs processes each commit
      against all files. For 100 commits × 500 files, that's
      50,000 iterations.

      With indexed lookup:
      1. Build HashMap<CommitId, Vec<FileId>>  // O(n)
      2. Look up files for each commit         // O(1)
      Total: O(n) instead of O(n²)

You: Show me the indexed version

Iris: [Provides code example]
```

### Incremental Review

After fixing issues:

1. Press <kbd>r</kbd> to regenerate
2. Iris focuses on **new issues only** (remembers context)
3. Confirms fixes: "✓ O(n²) loop now uses indexed lookup"

## Review Modes

### Quick Review (Default)

Fast analysis focusing on critical issues:

- Security vulnerabilities
- Obvious bugs
- Major performance problems

Takes 10-30 seconds.

### Deep Review (Future)

Comprehensive analysis including:

- Test coverage gaps
- Documentation completeness
- Architecture adherence
- Dependency analysis

Takes 1-3 minutes.

## Tips & Tricks

### 1. Review Before Committing

Make it a habit:

1. Write code
2. <kbd>Shift+R</kbd> → Review
3. Fix issues
4. <kbd>Shift+C</kbd> → Commit

### 2. Use Chat for Explanations

Don't guess what warnings mean:

- See ⚠️ → Press <kbd>/</kbd> → Ask "Explain this warning"
- Iris provides detailed context and examples

### 3. Compare with Upstream

Before merging to main:

- From: `origin/main`
- To: `HEAD`
- Reviews what will land in production

### 4. Save Reviews for PRs

Copy review (<kbd>y</kbd>) and paste into:

- PR description
- Commit message (for complex changes)
- Team wiki (as examples)

### 5. Focus on One Dimension

Use <kbd>j</kbd>/<kbd>k</kbd> to scroll directly to:

- Security (if handling user input)
- Performance (if optimizing hot paths)
- Best Practices (for mentorship/learning)

### 6. Iterative Improvement

Don't try to fix everything at once:

1. First pass: Fix ✗ (errors)
2. Second pass: Address ⚠️ (warnings)
3. Third pass: Polish ✓ sections

## Troubleshooting

### Review is empty

**Symptom**: Center panel shows "No review generated"

**Fix**:

1. Check that from/to refs are different
2. Ensure there are actual changes in range
3. Press <kbd>r</kbd> to manually trigger
4. Check status bar for errors

### Review takes too long

**Symptom**: Iris status shows "Thinking..." for >1 minute

**Cause**: Very large diff (1000+ lines)

**Fix**:

1. Narrow the ref range (fewer commits)
2. Review files individually (select in left panel)
3. Use chat instead: "Review the security of iris.rs"

### Line numbers don't match

**Symptom**: Review mentions line 45, but diff shows line 50

**Cause**: Line numbers are from **after** changes (in "to" ref)

**Fix**: Navigate diff to find context around that area.

### No security issues found but I'm suspicious

**Symptom**: Review says "✓ No security issues" but you're not convinced

**Fix**:

1. Press <kbd>/</kbd> to open chat
2. Ask specific questions: "Could this be vulnerable to XSS?"
3. Request focused analysis: "Review line 45 for SQL injection"

## Next Steps

- Use review findings to improve [Commit Messages](commit.md)
- Combine with [Explore Mode](explore.md) to understand flagged code
- Generate [PR Descriptions](pr.md) that include review summary
- Learn [Chat](../chat.md) for detailed review discussions
