# Chat with Iris

Press <kbd>/</kbd> in any mode to open the **universal chat interface**. Chat with Iris about code, ask for changes to generated content, or get explanations—all while maintaining context across modes.

## What Makes Chat Special

### Universal & Persistent

- **Works everywhere**: Available in all modes
- **Persists across mode switches**: Start a conversation in Commit, continue in Review
- **Shared context**: Iris knows about all generated content (commits, reviews, PRs, etc.)

### Direct Content Updates

Iris has special tools to modify content directly:

- **`update_commit`** — Edit commit messages
- **`update_pr`** — Modify PR descriptions
- **`update_review`** — Change review content

No need to regenerate—just ask for changes.

## Opening Chat

| Key            | Action                             |
| -------------- | ---------------------------------- |
| <kbd>/</kbd>   | Open chat modal                    |
| <kbd>Esc</kbd> | Close chat (conversation persists) |

## Chat Interface

```
┌─────────────────────────────────────────────────┐
│  Chat with Iris                            [Esc] │
├─────────────────────────────────────────────────┤
│                                                 │
│  You:                                           │
│  Why did you choose ✨ for this commit?        │
│                                                 │
│  Iris:                                          │
│  I chose ✨ (sparkles) because this commit      │
│  introduces a new feature—the emoji selector.   │
│  It's a conventional gitmoji for feature        │
│  additions that enhance the user experience.    │
│                                                 │
│  You:                                           │
│  Make the commit message more concise           │
│                                                 │
│  Iris: ⠋ Thinking...                            │
│                                                 │
├─────────────────────────────────────────────────┤
│  > Your message here_                           │
└─────────────────────────────────────────────────┘
```

## Chat Controls

| Key                                                    | Action               |
| ------------------------------------------------------ | -------------------- |
| <kbd>Type</kbd>                                        | Enter your message   |
| <kbd>Enter</kbd>                                       | Send message to Iris |
| <kbd>Esc</kbd>                                         | Close chat modal     |
| <kbd>j</kbd>/<kbd>k</kbd> or <kbd>↓</kbd>/<kbd>↑</kbd> | Scroll chat history  |
| <kbd>Ctrl+d</kbd>/<kbd>Ctrl+u</kbd>                    | Page up/down         |

## What You Can Ask

### Ask About Generated Content

```
You: Why did you choose this title for the commit?
You: Explain the security issues you found
You: What makes this a breaking change?
```

Iris will reference the current mode's content to answer.

### Request Content Changes

```
You: Make the commit message shorter
You: Add more detail to the security section
You: Rewrite the PR description to be more technical
You: Use a different emoji
```

Iris will use her update tools to modify content directly.

### Get Code Explanations

```
You: What does the reducer pattern do here?
You: Why is this file tree component separate?
You: Explain the visual selection feature
```

Iris can access project documentation (README, CLAUDE.md, AGENTS.md) to give context-aware answers.

### Troubleshoot Issues

```
You: Why isn't the commit message showing?
You: How do I stage all files?
You: What do the scroll indicators mean?
```

Iris knows Studio's features and can guide you.

## Context Awareness

Chat automatically loads context from:

1. **Current mode's generated content**
   - Commit message (if in Commit mode)
   - Review output (if in Review mode)
   - PR description (if in PR mode)
   - Changelog (if in Changelog mode)
   - Release notes (if in Release mode)

2. **Project documentation**
   - README.md
   - CLAUDE.md (developer instructions)
   - AGENTS.md (agent behavior docs)

3. **Conversation history**
   - All previous messages in the session
   - Context builds throughout conversation

## Example Conversations

### Commit Mode: Refining Messages

```
You: The commit message is too long. Make it more concise.

Iris: I've updated the commit message to be more direct:
      ✨ Add emoji selector to commit UI

      Let me know if you'd like it even shorter!

You: Perfect! Now add that I also fixed a bug with the file tree.

Iris: Updated:
      ✨ Add emoji selector and fix file tree selection
```

Iris modifies the message directly using `update_commit`.

### Review Mode: Understanding Issues

```
You: Explain the performance issue you found in line 45

Iris: The issue on line 45 is an O(n²) nested loop:

      for commit in commits:
          for file in all_files:  # Scans all files for each commit
              ...

      This scales poorly with large repos. Consider indexing
      files by commit ID first, then doing O(1) lookups.

You: Can you add this optimization as a suggestion in the review?

Iris: Done! I've added a "Performance Optimization" section
      to the review with the indexed lookup pattern.
```

### PR Mode: Formatting Changes

```
You: Make the PR description more bullet-pointy, less prose

Iris: Updated the PR to use concise bullet lists:

      ## Changes
      - Added emoji selector modal
      - Fixed file tree selection sync
      - Improved keyboard navigation

      ## Technical Details
      - New `EmojiSelector` modal component
      - `sync_file_selection()` helper for tree<->diff sync

You: Great! Add a migration note about the new emoji feature.

Iris: Added:

      ## Migration
      - Emoji mode defaults to Auto (AI chooses)
      - Use `g` key to open selector in Commit mode
```

### Explore Mode: Code Questions

````
You: Why does this file use a reducer pattern?

Iris: The reducer pattern in `state.rs` provides:

      1. Predictable state transitions
      2. Pure functions for testing
      3. Clear event history for debugging
      4. Time-travel replay capability

      It's documented in CLAUDE.md under "Studio Architecture":

      > Pure function: (state, event) → (state, effects)
      > No I/O inside reducer—side effects returned as data

You: Show me how to add a new event type

Iris: To add a new event:

      1. Add variant to `StudioEvent` enum in `events.rs`:
         ```rust
         pub enum StudioEvent {
             YourNewEvent { data: String },
         }
         ```

      2. Handle in reducer (`reducer.rs`):
         ```rust
         match event {
             StudioEvent::YourNewEvent { data } => {
                 state.your_field = data;
                 (state, vec![])
             }
         }
         ```

      3. Dispatch from handler or async task
````

## Direct Content Updates

Iris can modify generated content **without regeneration**:

### Update Commit Message

```
You: Change the emoji to 🐛

Iris: ✓ Updated commit message emoji to 🐛
```

Behind the scenes: `update_commit(new_message)`

### Update Review Content

```
You: Remove the style nitpicks section

Iris: ✓ Removed "Style & Formatting" section from review
```

Behind the scenes: `update_review(filtered_content)`

### Update PR Description

```
You: Add a "Breaking Changes" section

Iris: ✓ Added Breaking Changes section:

      ## Breaking Changes
      - Emoji selector modal requires new keybinding (g)
      - EmojiMode enum replaces boolean use_gitmoji flag
```

Behind the scenes: `update_pr(new_description)`

## Markdown Rendering

Iris's responses support **rich markdown**:

- **Bold** and _italic_ text
- `inline code`
- `code blocks with syntax highlighting`
- > Blockquotes
- Lists (bulleted and numbered)
- Links (displayed, not clickable in TUI)

Code blocks in chat match Studio's syntax highlighting theme.

## Chat Status Indicators

### Iris Is Thinking

```
Iris: ⠋ Thinking...
```

Animated Braille spinner while Iris processes your request.

### Error State

```
Iris: ✗ Error: Could not update commit message
      (No message currently generated)
```

Red text for errors with explanation.

### Tool Use

```
Iris: 🔧 Using update_commit tool...
      ✓ Commit message updated
```

Shows which tools Iris is calling (if visible in debug mode).

## Privacy & Persistence

### What Persists

- **Message history**: Survives modal close, mode switches
- **Context**: All generated content from current session

### What Doesn't Persist

- **Chat state**: Cleared when you quit Studio
- **API calls**: Each message is a new request (no server-side storage)

### Data Flow

1. You type message → Sent to LLM with context
2. LLM responds → Displayed in chat
3. If tool call needed → Iris executes locally
4. Result → Shown in chat

No data leaves your machine except LLM API calls.

## Advanced Usage

### Multi-Turn Conversations

Chat maintains context across turns:

````
You: What's the biggest file in the diff?

Iris: `src/studio/state.rs` at 1,183 lines is the largest
      file in this diff.

You: Show me its main struct

Iris: The main struct is `StudioState`:
      ```rust
      pub struct StudioState {
          pub repo: Option<Arc<GitRepo>>,
          pub git_status: GitStatus,
          pub config: Config,
          pub active_mode: Mode,
          pub focused_panel: PanelId,
          pub modes: ModeStates,
          // ... 14 fields total
      }
      ```

You: Why is repo an Option?

Iris: It's `Option<Arc<GitRepo>>` because Studio can run:
      1. Outside a git repository (demo mode)
      2. With repository errors (permissions, corruption)

      The `Arc` allows sharing across threads for async tasks.
````

### Combining Chat with Tools

Use chat to understand, then use Studio tools to act:

1. **Ask in chat**: "Which files should I review first?"
2. **Iris suggests**: File list based on change significance
3. **You navigate**: Use <kbd>j</kbd>/<kbd>k</kbd> to select file
4. **You view**: <kbd>Enter</kbd> to load in diff panel

### Iterative Refinement

```
You: Generate a commit message
[Iris generates]

You: Too formal. Make it casual.
[Iris updates]

You: Add an emoji
[Iris updates with emoji]

You: Perfect!
```

Each iteration refines without full regeneration.

## Limitations

### What Chat Can't Do

- **Navigate UI**: Can't move focus or select files for you
- **Execute git commands**: Can't commit, push, or stage files
- **Open external tools**: Can't launch editors or browsers
- **Modify files**: Can only update Studio-generated content

### What Chat _Can_ Do

- **Read generated content**: Commits, reviews, PRs, changelogs, release notes
- **Update generated content**: Direct modification via tools
- **Access project docs**: README, CLAUDE.md, AGENTS.md
- **Explain code**: Based on visible context and documentation
- **Guide usage**: Studio features, keybindings, workflows

## Tips for Effective Chat

### Be Specific

❌ "Make it better"
✅ "Make the commit message more concise and add technical details"

### Ask One Thing at a Time

❌ "Fix the emoji, shorten the message, and add a footer"
✅ "Change the emoji to 🐛" → wait → "Now shorten the message"

### Reference Context

✅ "In the review, explain the security issue on line 45"
✅ "Why did you categorize the database change as 'Changed' not 'Added'?"

### Use Chat for Questions, Hotkeys for Actions

- **Chat**: "Why is this file tree organized this way?"
- **Hotkey**: <kbd>r</kbd> to regenerate, <kbd>e</kbd> to edit

## Next Steps

- Learn mode-specific workflows:
  - [Commit Mode](modes/commit.md) — Refine messages with chat
  - [Review Mode](modes/review.md) — Understand review findings
  - [Explore Mode](modes/explore.md) — Ask about code history
- Master [Navigation Patterns](navigation.md) to move quickly
- Read [Studio Overview](index.md) for global features
