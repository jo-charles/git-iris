# Reducer Pattern

**Deep dive into Iris Studio's pure reducer architecture.**

## What is a Reducer?

A **reducer** is a pure function that takes the current state and an event, then returns the new state and any side effects:

```
(state, event) → (state', effects)
```

**Key properties:**

- **Pure** — No I/O, no randomness, no hidden state
- **Predictable** — Same inputs always produce same outputs
- **Testable** — Trivial to unit test
- **Traceable** — Can log every state transition

## Why Use a Reducer?

Traditional imperative UI code scatters state mutations everywhere:

```rust
// BAD: State mutations scattered throughout handlers
fn handle_key(&mut self, key: Key) {
    if key == 'g' {
        self.generating = true;
        self.spawn_agent();  // Side effect!
        self.status = "Thinking...";
    }
}
```

**Problems:**

- Hard to test (needs mocking)
- Hard to trace (what changed when?)
- Hard to debug (where did this state come from?)
- Race conditions with async code

With a reducer, everything flows through one function:

```rust
// GOOD: Single source of truth
fn reduce(state: &mut State, event: Event) -> Vec<Effect> {
    match event {
        Event::GenerateCommit { .. } => {
            state.generating = true;
            state.status = "Thinking...";
            vec![Effect::SpawnAgent { task: Commit }]
        }
    }
}
```

**Benefits:**

- **Single place** to look for state changes
- **Pure logic** separate from I/O
- **Easy testing** — no mocking needed
- **Audit trail** — log every `(state, event, effects)` triple

## The Reducer Function

Located in `src/studio/reducer/mod.rs`:

```rust
pub fn reduce(
    state: &mut StudioState,
    event: StudioEvent,
    history: &mut History,
) -> Vec<SideEffect>
```

**Parameters:**

- `state` — Mutable reference to application state
- `event` — The event to process
- `history` — Mutable reference to event history

**Returns:**

- `Vec<SideEffect>` — Side effects to execute after state update

### Why Mutate State Directly?

Some reducer patterns return a new state (pure functional style). We mutate in-place for performance:

```rust
// Pure functional (expensive for large state)
fn reduce(state: State, event: Event) -> (State, Vec<Effect>) {
    let mut new_state = state.clone();  // Full clone!
    // ... mutations ...
    (new_state, effects)
}

// In-place mutation (what we use)
fn reduce(state: &mut State, event: Event) -> Vec<Effect> {
    // Direct mutations
    state.mode = Mode::Commit;
    effects
}
```

We still get **predictability** because:

1. All mutations happen in one function
2. We log the before/after state in history
3. We can replay events to reconstruct state

## Event Processing Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Event Loop                             │
│                                                             │
│  1. Pop event from queue                                    │
│         │                                                   │
│         ▼                                                   │
│  2. Call reduce(state, event, history)                      │
│         │                                                   │
│         ├──▶ Match event variant                            │
│         ├──▶ Update state fields                            │
│         ├──▶ Record to history                              │
│         └──▶ Build effect list                              │
│         │                                                   │
│         ▼                                                   │
│  3. Return effects                                          │
│         │                                                   │
│         ▼                                                   │
│  4. Execute effects (app/mod.rs)                            │
│         │                                                   │
│         ├──▶ SpawnAgent → tokio::spawn                      │
│         ├──▶ GitStage → git add                             │
│         ├──▶ LoadData → async load                          │
│         └──▶ Effects emit new events                        │
│                   │                                         │
│                   └──▶ Back to event queue                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Anatomy of an Event Handler

Let's trace `GenerateCommit`:

```rust
StudioEvent::GenerateCommit {
    instructions,
    preset,
    use_gitmoji,
} => {
    // 1. Update UI state
    state.modes.commit.generating = true;
    state.set_iris_thinking("Generating commit message...");

    // 2. Record to history
    history.record_agent_start(TaskType::Commit);

    // 3. Build effect
    effects.push(SideEffect::SpawnAgent {
        task: AgentTask::Commit {
            instructions,
            preset,
            use_gitmoji,
        },
    });
}
```

**Notice:**

- State mutations happen **immediately**
- No async/await (pure, synchronous)
- Effect describes what to do, doesn't do it
- History records the transition

## Side Effects

**Effects are data** describing I/O operations:

```rust
pub enum SideEffect {
    SpawnAgent { task: AgentTask },
    LoadData { data_type, from_ref, to_ref },
    GitStage(PathBuf),
    GitUnstage(PathBuf),
    SaveSettings,
    RefreshGitStatus,
    CopyToClipboard(String),
    ExecuteCommit { message },
    Quit,
}
```

**Why return effects instead of executing directly?**

1. **Testability** — Test logic without I/O
2. **Traceability** — Log all effects
3. **Batching** — Combine multiple effects
4. **Ordering** — Control execution order

### Effect Execution

After reducer returns, `StudioApp::execute_effects()` runs:

```rust
fn execute_effects(&mut self, effects: Vec<SideEffect>) {
    for effect in effects {
        match effect {
            SideEffect::SpawnAgent { task } => {
                self.spawn_agent_task(task);
            }
            SideEffect::GitStage(path) => {
                if let Some(svc) = &self.commit_service {
                    svc.stage_file(&path)?;
                    // Emit new event
                    self.push_event(StudioEvent::FileStaged(path));
                }
            }
            // ...
        }
    }
}
```

**Notice:** Effects can trigger new events, which feed back into the reducer.

## State Structure

`StudioState` holds all application state:

```rust
pub struct StudioState {
    // Repository & git
    pub repo: Option<Arc<GitRepo>>,
    pub git_status: GitStatus,

    // Configuration
    pub config: Config,

    // Navigation
    pub active_mode: Mode,
    pub focused_panel: PanelId,

    // Mode-specific state
    pub modes: ModeStates,

    // Overlays
    pub modal: Option<Modal>,
    pub chat_state: ChatState,

    // Notifications
    pub notifications: VecDeque<Notification>,

    // Agent status
    pub iris_status: IrisStatus,

    // UI state
    pub dirty: bool,
    pub last_render: Instant,
}
```

### Mode-Specific State

Each mode has its own state struct:

```rust
pub struct ModeStates {
    pub explore: ExploreMode,
    pub commit: CommitMode,
    pub review: ReviewMode,
    pub pr: PRMode,
    pub changelog: ChangelogMode,
    pub release_notes: ReleaseNotesMode,
}

pub struct CommitMode {
    pub messages: Vec<GeneratedMessage>,
    pub current_index: usize,
    pub generating: bool,
    pub message_editor: MessageEditorState,
    pub diff_view: DiffViewState,
    pub file_tree: FileTreeState,
}
```

**Why separate?** Each mode has unique state. Keeps `StudioState` organized.

## History Recording

The reducer records all significant events to `History`:

```rust
// Agent started
history.record_agent_start(TaskType::Commit);

// Agent completed
history.record_agent_complete(TaskType::Commit, success: true);

// Mode switched
history.record_mode_switch(old_mode, new_mode);

// Content generated
history.record_commit_message(message);
history.record_review_content(content);

// Chat message
history.add_chat_message(role, message, mode, context);
```

**Why in reducer?** Guarantees every state change is recorded, no missed events.

## Common Reducer Patterns

### Pattern 1: Simple State Update

```rust
StudioEvent::FocusNext => {
    state.focus_next_panel();
    // No effects needed
}
```

### Pattern 2: State Update + Effect

```rust
StudioEvent::RefreshGitStatus => {
    state.set_iris_thinking("Refreshing git status...");
    effects.push(SideEffect::RefreshGitStatus);
}
```

### Pattern 3: Conditional Logic

```rust
StudioEvent::StageFile(path) => {
    if state.git_status.modified_files.contains(&path) {
        effects.push(SideEffect::GitStage(path));
    } else {
        state.notify(Notification::warning("File not modified"));
    }
}
```

### Pattern 4: Mode-Specific Behavior

```rust
StudioEvent::GenerateReview { from_ref, to_ref } => {
    match state.active_mode {
        Mode::Review => {
            state.modes.review.generating = true;
            effects.push(SideEffect::SpawnAgent {
                task: AgentTask::Review { from_ref, to_ref },
            });
        }
        _ => {
            // Wrong mode, ignore or warn
        }
    }
}
```

### Pattern 5: Multi-Step Updates

```rust
StudioEvent::SwitchMode(new_mode) => {
    let old_mode = state.active_mode;

    // 1. Record to history
    history.record_mode_switch(old_mode, new_mode);

    // 2. Update state
    state.switch_mode(new_mode);

    // 3. Trigger data load for new mode
    match new_mode {
        Mode::Commit => {
            effects.push(SideEffect::LoadData {
                data_type: DataType::CommitDiff,
                from_ref: None,
                to_ref: None,
            });
        }
        // ... other modes
    }
}
```

## Async Event Loop

**Problem:** Reducer is synchronous, but LLM calls and git ops are async.

**Solution:** Effects spawn async tasks that send events back via channel:

```
Reducer (sync)
    │
    └──▶ Effect: SpawnAgent
            │
            └──▶ Executor (sync)
                    │
                    └──▶ tokio::spawn (async)
                            │
                            ├──▶ Call LLM API
                            ├──▶ Wait for response
                            └──▶ Send AgentComplete event via channel
                                    │
                                    └──▶ Event loop receives
                                            │
                                            └──▶ Back to reducer
```

**Key insight:** Async work happens **outside** the reducer. Results come back as events.

## Testing the Reducer

Pure functions are trivial to test:

```rust
#[test]
fn test_generate_commit_starts_agent() {
    let mut state = test_state();
    let mut history = History::new();

    let event = StudioEvent::GenerateCommit {
        instructions: None,
        preset: "default".into(),
        use_gitmoji: true,
    };

    let effects = reduce(&mut state, event, &mut history);

    // Assert state changes
    assert!(state.modes.commit.generating);
    assert!(matches!(state.iris_status, IrisStatus::Thinking { .. }));

    // Assert effects
    assert_eq!(effects.len(), 1);
    assert!(matches!(effects[0], SideEffect::SpawnAgent { .. }));

    // Assert history
    assert_eq!(history.events.len(), 1);
}
```

**No mocking, no async, no I/O.** Just pure logic.

## Debugging Tips

### 1. Log Every Event

```rust
pub fn reduce(state: &mut State, event: Event, history: &mut History) -> Vec<Effect> {
    eprintln!("[REDUCE] {:?}", event);
    // ... reducer logic ...
    eprintln!("[EFFECTS] {:?}", effects);
    effects
}
```

### 2. Snapshot State Before/After

```rust
let before = format!("{:?}", state);
let effects = reduce(state, event, history);
let after = format!("{:?}", state);
eprintln!("BEFORE: {}\nAFTER: {}", before, after);
```

### 3. Check History

```rust
// In test or at runtime
for event in &history.events {
    println!("{:?} at {:?}", event.event, event.timestamp);
}
```

### 4. Assert Effect Ordering

```rust
let effects = reduce(&mut state, event, &mut history);
assert!(effects[0].is_spawn_agent());
assert!(effects[1].is_load_data());
```

## Performance Considerations

**Reducer is fast** — Simple pattern matching and field assignments.

**Avoid expensive operations** in reducer:

- No network calls
- No file I/O
- No heavy computation

If you need to compute something expensive, return it as an effect:

```rust
// BAD: Expensive work in reducer
StudioEvent::AnalyzeCode => {
    let analysis = expensive_analysis(&state);  // Blocks reducer!
    state.analysis = analysis;
}

// GOOD: Spawn async task
StudioEvent::AnalyzeCode => {
    effects.push(SideEffect::SpawnAnalysis);
}
```

## Advanced Patterns

### Optimistic Updates

Update state immediately, rollback if operation fails:

```rust
StudioEvent::StageFile(path) => {
    // Optimistic: assume success
    state.git_status.staged_files.push(path.clone());
    state.dirty = true;

    effects.push(SideEffect::GitStage(path));
}

StudioEvent::FileStageFailed(path) => {
    // Rollback on error
    state.git_status.staged_files.retain(|p| p != &path);
    state.notify(Notification::error("Failed to stage file"));
}
```

### Batched Effects

Combine multiple effects into one:

```rust
StudioEvent::RefreshAll => {
    effects.push(SideEffect::RefreshGitStatus);
    effects.push(SideEffect::LoadData { ... });
    effects.push(SideEffect::LoadData { ... });
}
```

Executor can batch git operations for efficiency.

### Derived State

Compute state from other state (like React's useMemo):

```rust
// In state/mod.rs
impl StudioState {
    pub fn has_uncommitted_changes(&self) -> bool {
        self.git_status.staged_count > 0 ||
        self.git_status.modified_count > 0
    }
}
```

Don't store derived state, compute on demand.

## Common Pitfalls

### ❌ I/O in Reducer

```rust
// WRONG
StudioEvent::SaveSettings => {
    self.config.save_to_file()?;  // I/O!
}
```

```rust
// RIGHT
StudioEvent::SaveSettings => {
    effects.push(SideEffect::SaveSettings);
}
```

### ❌ Async in Reducer

```rust
// WRONG
StudioEvent::GenerateCommit => {
    let result = agent.generate().await?;  // async!
    state.message = result;
}
```

```rust
// RIGHT
StudioEvent::GenerateCommit => {
    state.generating = true;
    effects.push(SideEffect::SpawnAgent { ... });
}
```

### ❌ Side Effects Without Events

```rust
// WRONG - invisible state change
fn execute_effect(effect: Effect) {
    match effect {
        Effect::GitStage(path) => {
            git_add(&path);
            self.state.staged_files.push(path);  // Hidden mutation!
        }
    }
}
```

```rust
// RIGHT - emit event for reducer
fn execute_effect(effect: Effect) {
    match effect {
        Effect::GitStage(path) => {
            git_add(&path);
            self.push_event(StudioEvent::FileStaged(path));  // Explicit!
        }
    }
}
```

## Comparison to Other Patterns

### Traditional MVC

```
Controller ──▶ Model (scattered mutations)
               │
               └──▶ View (reads model)
```

**Problems:** Hard to trace, mutations everywhere.

### MVVM / Data Binding

```
ViewModel ◀──▶ View (two-way binding)
```

**Problems:** Complex dependency tracking, hard to debug.

### Reducer Pattern

```
Event ──▶ Reducer ──▶ State ──▶ View
           │
           └──▶ Effects ──▶ Async ──▶ New Events
```

**Benefits:** Unidirectional flow, explicit state changes, traceable.

## Further Reading

- [Redux documentation](https://redux.js.org/) (web equivalent)
- [Elm architecture](https://guide.elm-lang.org/architecture/) (original pattern)
- [Flux architecture](https://facebookarchive.github.io/flux/) (Facebook's pattern)

## Summary

**Reducer = single source of truth for state transitions**

- Pure function: `(state, event) → (state', effects)`
- All mutations in one place
- Side effects are explicit data
- Easy to test, trace, and debug
- Async work happens outside, results come back as events

**When in doubt:** If it's not a field assignment or simple logic, it should be an effect.
