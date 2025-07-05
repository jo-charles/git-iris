# Git-Iris Agent Framework Documentation

This document serves as a comprehensive guide to the intelligent agent framework we've built for Git-Iris. This is designed to help future development sessions understand the architecture, capabilities, and design philosophy.

## 🤖 Meet Iris - The Unified AI Agent

Iris is the central AI agent that powers all Git-Iris operations. Unlike traditional static tools, Iris is truly agentic - she plans her own tasks, uses tools intelligently, takes notes, manages context, and adapts her approach based on what she learns.

### Core Philosophy: LLM-Driven Everything

**Key Principle**: The LLM (Iris) makes all the intelligent decisions. We avoid deterministic heuristics and instead let Iris:
- Decide which tools to use and when
- Manage her own context and summarization
- Take notes and build knowledge as she works
- Plan and adapt her approach dynamically
- Handle complex workflows through intelligent orchestration

## 🏗️ Architecture Overview

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Iris Agent    │◄──►│  Tool Registry  │◄──►│ Status Tracker │
│  (LLM Brain)    │    │   (Abilities)   │    │ (Progress UI)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Context Mgmt   │    │ Workspace Tool  │    │  CLI Spinner    │
│ (Smart Sizing)  │    │ (Notes/Tasks)   │    │ (Real-time UI)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Key Files & Modules

- **`src/agents/iris.rs`** - The main Iris agent implementation
- **`src/agents/core.rs`** - Agent backend abstraction and context
- **`src/agents/tools/`** - Clean, organized tool implementations:
  - `mod.rs` - Tool trait and registry
  - `git.rs` - Git repository operations
  - `file_analyzer.rs` - File content analysis
  - `code_search.rs` - Code pattern searching
  - `workspace.rs` - Iris's personal workspace
- **`src/agents/status.rs`** - Status tracking and progress display
- **`src/agents/integration.rs`** - CLI integration layer
- **`src/ui.rs`** - Beautiful spinner and status UI

## 🧠 Iris Capabilities

### 1. Adaptive Planning & Execution
Iris doesn't follow predetermined scripts. Instead, she:
- **Plans intelligently**: Creates initial tool usage plans based on the task
- **Expands dynamically**: Adds more tools as she discovers new context
- **Adapts in real-time**: Modifies her approach based on what she learns

```rust
// Example: Iris decides her own tool sequence
async fn plan_tool_usage_for_context_analysis(&self, context: &AgentContext) -> Result<Vec<ToolPlan>>
async fn expand_plan_based_on_context(&self, context: &AgentContext, current_results: &[ToolResult], remaining_plan: &[ToolPlan]) -> Result<Vec<ToolPlan>>
```

### 2. Intelligent Context Management
One of our key innovations - Iris manages her own context to avoid token limits:

```rust
async fn manage_review_context(&self, context: &CommitContext) -> Result<String> {
    // If context is small enough, use it directly
    if context_size < 8000 {
        return Ok(full_context);
    }
    
    // Otherwise, ask Iris to intelligently summarize
    let smart_context_prompt = format!(
        "Create a focused summary that preserves all critical information..."
    );
    
    self.analyze_with_backend(&smart_context_prompt).await
}
```

**What Iris preserves:**
- Security-critical changes
- Complex logic requiring careful review
- Performance-sensitive code
- Error handling patterns
- API changes or breaking changes
- Critical diff sections (exact code)

**What Iris summarizes:**
- Repetitive patterns
- Simple refactoring
- Formatting changes
- Non-critical utility functions

### 3. Self-Managing Workspace
Iris has her own workspace tool for taking notes and managing tasks:

```rust
// Iris can create her own tasks
workspace.add_task("Analyze security implications of auth changes", "high")

// Take notes as she works
workspace.add_note("Found potential SQL injection in user input validation", ["security", "critical"])

// Update task progress
workspace.update_task("task_1", Some("completed"), Some("Implemented rate limiting"))
```

This allows Iris to:
- Keep track of complex analysis workflows
- Remember insights across tool calls
- Plan multi-step operations
- Build institutional knowledge

### 4. Beautiful Status Display
Real-time progress feedback with clean, futuristic design:

```
⠋ Iris is performing deep analysis...  
⠙ Iris is synthesizing findings...      
⠹ Iris is generating recommendations...  
⠸ ◎ Analysis complete                   
```

- **Braille pattern spinners** for clean, professional look
- **Real-time status updates** showing Iris's thinking process
- **Phase-aware progress** (Planning → Analysis → Synthesis → Generation)
- **Integrated with CLI** using existing `indicatif` infrastructure

## 🛠️ Tool Ecosystem

### Core Tools Available to Iris

Tools are now cleanly organized in `src/agents/tools/` with each tool in its own module:

#### 1. Git Tool (`src/agents/tools/git.rs`)
```rust
// Operations Iris can perform
"diff"   -> Get staged file changes with full context
"status" -> Repository status and unstaged files  
"log"    -> Recent commit history
"files"  -> List of changed files
```

#### 2. File Analyzer Tool (`src/agents/tools/file_analyzer.rs`)
```rust
// Iris can analyze files in detail
analyzer.analyze(path, include_content: true)
// Returns: file type, line count, content, language-specific insights
```

#### 3. Code Search Tool (`src/agents/tools/code_search.rs`)
```rust
// Search for patterns, functions, classes
search.find("function", query, file_pattern, max_results)
```

#### 4. Workspace Tool (`src/agents/tools/workspace.rs`)
```rust
// Iris's personal productivity system
workspace.add_note(content, tags)
workspace.add_task(description, priority)
workspace.update_task(id, status, note)
workspace.get_summary()
```

#### Tool Organization Benefits
- **Clean separation** - Each tool is self-contained in its own file
- **Easy to extend** - Adding new tools is straightforward
- **Clear documentation** - Each tool has comprehensive docs
- **Modular design** - Tools can be developed and tested independently

### Tool Selection Philosophy
Tools are **capabilities**, not rigid scripts. Iris decides:
- Which tools to use for each task
- What parameters to pass
- When to use multiple tools in sequence
- How to combine results from different tools

## 📊 Status & Progress Tracking

### Real-Time Status System
```rust
// Iris updates her status throughout execution
iris_status_planning!();        // "📋 Iris is planning her approach..."
iris_status_tool!("git", "Getting repository context");  // "🔧 Iris is analyzing git repository..."
iris_status_analysis!();        // "🔬 Iris is performing deep analysis..."
iris_status_synthesis!();       // "🧬 Iris is synthesizing findings..."
iris_status_generation!();      // "✨ Iris is generating response..."
iris_status_completed!();       // "✅ Analysis complete"
```

### Phase-Based Progress
- **Initializing**: Iris awakens and prepares
- **Planning**: Creates initial tool usage plan
- **Tool Execution**: Executes tools with real-time updates
- **Plan Expansion**: Decides if more tools are needed
- **Analysis**: Deep analysis of gathered context
- **Synthesis**: Combines insights into coherent understanding
- **Generation**: Creates final output
- **Completed**: Task finished successfully

## 🔄 Workflow Examples

### Code Review Workflow
```
1. 📋 Iris plans review approach
2. 🔧 Analyzes git diff and repository context  
3. 📄 Examines specific files needing attention
4. 🧠 Builds understanding through intelligent context management
5. 📝 Takes notes on critical findings
6. 🔬 Performs deep analysis across quality dimensions
7. ✨ Generates comprehensive review with recommendations
```

### Commit Message Generation
```
1. 📋 Plans context gathering strategy
2. 🔧 Gets staged changes and file analysis
3. 🔍 Searches for related code patterns if needed
4. 📝 Takes notes on change significance  
5. 🧬 Synthesizes understanding of change impact
6. ✨ Generates contextual commit message
```

## 🎯 Key Design Decisions

### 1. LLM-First Architecture
- **No deterministic heuristics** - Iris makes all decisions
- **Context-aware tool selection** - Tools chosen based on situation
- **Adaptive behavior** - Changes approach based on findings

### 2. Agent Personification
- Iris is consistently referred to as "she" and "Iris"
- All prompts use "You are Iris" language
- Status messages show "Iris is..." for personal connection
- Creates a cohesive AI assistant experience

### 3. Intelligent Defaults with Flexibility
- Smart context management prevents token limit issues
- Beautiful UI provides feedback without being noisy
- Agent mode is opt-in via `--agent` flag
- Graceful fallbacks to traditional Git-Iris behavior

### 4. Self-Improving System
- Iris learns and takes notes as she works
- Workspace tool allows building institutional knowledge
- Future sessions can benefit from previous learnings
- Adaptive planning improves with experience

## 🚀 CLI Integration

### Agent Mode Activation
```bash
# Enable agent mode with --agent flag
git-iris gen --agent                    # AI commit message
git-iris review --agent --print         # AI code review  
git-iris pr --agent                     # AI PR description
git-iris changelog --agent --from v1.0  # AI changelog
```

### Status Display Integration
- **Seamless integration** with existing CLI infrastructure
- **Progressive enhancement** - works with existing commands
- **Clean visual design** using braille patterns
- **Real-time updates** without overwhelming output

## 🔧 Future Enhancements

### Planned Improvements
1. **MCP Server Integration** - Allow external tools to connect to Iris
2. **Project Metadata Integration** - Learn project-specific patterns
3. **Persistent Knowledge Base** - Remember insights across sessions
4. **Advanced Tool Orchestration** - Complex multi-tool workflows
5. **Custom Tool Development** - Easy addition of new capabilities

### Architectural Flexibility
The framework is designed to grow:
- **Modular tool system** - Easy to add new capabilities
- **Provider-agnostic backend** - Works with any LLM provider  
- **Extensible status system** - New phases and updates
- **Clean abstraction layers** - Separate concerns properly

## 💡 Development Philosophy

### Core Principles
1. **LLM Intelligence First** - Let the AI make decisions
2. **Beautiful User Experience** - Clean, informative, delightful
3. **Adaptive & Learning** - Improve with experience
4. **Modular Architecture** - Easy to extend and modify
5. **Fail Gracefully** - Fallback to working alternatives

### Code Quality
- **Comprehensive error handling** with graceful degradation
- **Extensive logging** for debugging and understanding
- **Clean abstractions** separating concerns
- **Thread-safe design** for concurrent operations
- **Performance conscious** with intelligent optimizations

---

## 🎉 What We've Built

This agent framework represents a significant evolution in how AI tools work. Instead of static, predetermined sequences, we have:

- **A truly intelligent agent** that plans and adapts
- **Beautiful, real-time feedback** showing the agent's thinking
- **Sophisticated context management** preventing token issues
- **Self-improving workspace** for building knowledge
- **Seamless CLI integration** that enhances existing workflows

Iris isn't just a tool - she's an AI assistant that understands Git workflows, learns from experience, and provides intelligent, contextual help exactly when you need it.

The future of Git-Iris is agentic, adaptive, and absolutely delightful to use. 🚀