# Git-Iris 2.0: Agent-Based Architecture

## Overview

Git-Iris 2.0 reimagines our application as an agent-centric system built on the Rig framework, moving away from the current approach of "throwing a bunch of context at the LLM and hoping for the best." This document outlines our architectural vision, implementation strategy, and migration path.

## Core Vision

Rather than gathering all possible context upfront, Git-Iris 2.0 will employ specialized agents that dynamically explore a codebase to gather precisely the context needed for a given task. These agents will:

1. Start with minimal context (the diff)
2. Iteratively explore the codebase through tool calls
3. Make decisions about what additional information is relevant
4. Stop when they have sufficient context to complete the task

This approach offers several advantages:
- **Precision over bulk**: Only gather what's necessary
- **Improved performance**: Reduce token usage and processing time
- **Better user experience**: Faster responses with more relevant outputs
- **Scalability**: Handle larger codebases without overwhelming token limits
- **Adaptability**: Easily adjust exploration strategy based on task requirements

## Architecture

### System Components

```
┌─────────────────────────────────────────────┐
│                                             │
│                Git-Iris Core                │
│                                             │
└───────────────────┬─────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│                                             │
│              Agent Orchestrator             │
│                                             │
└───────┬───────────────┬───────────┬─────────┘
        │               │           │
        ▼               ▼           ▼
┌───────────────┐ ┌───────────┐ ┌───────────┐
│ Commit Agent  │ │ Review    │ │ Changelog │
│               │ │ Agent     │ │ Agent     │
└───────┬───────┘ └─────┬─────┘ └─────┬─────┘
        │               │             │
        │               │             │
┌───────▼───────────────▼─────────────▼─────┐
│                                           │
│                 Tool Registry             │
│                                           │
└───────────────────────────────────────────┘
```

### Key Components

1. **Git-Iris Core**: The main application shell, handling user interactions, configuration, and coordinating the various agents. Provides a unified interface for CLI and API interactions.

2. **Agent Orchestrator**: Manages agent lifecycle, initializes appropriate agents based on the task, and provides common infrastructure. Responsible for:
   - Agent instantiation and configuration
   - Resource allocation and monitoring
   - Inter-agent communication
   - Task delegation and result aggregation
   - Error handling and recovery strategies

3. **Specialized Agents**:
   - **Commit Agent**: Generates commit messages based on staged changes
   - **Review Agent**: Performs code reviews of staged changes
   - **Changelog Agent**: Generates changelogs and release notes
   - **Context Explorer Agent**: Explores and maps codebase structure
   - **Code Analyzer Agent**: Analyzes specific code pieces for details
   - **Structure Analyzer Agent**: Maps relationships and dependencies
   - **Meta Analyzer Agent**: Examines metadata and configuration

4. **Tool Registry**: Centralized registry of tools that agents can utilize to explore and understand the codebase. Features:
   - Dynamic tool discovery and registration
   - Tool versioning and compatibility management
   - Access control and security boundaries
   - Performance monitoring and optimization

### Agent Workflow

Each specialized agent follows this general process:

1. **Initialize** with minimal context (usually the diff or basic repository info)
2. **Explore** the codebase using tools to gather relevant information
3. **Analyze** the gathered information to determine what else is needed
4. **Generate** the desired output once sufficient context is obtained
5. **Refine** the output based on additional context if necessary

## Implementation Using Rig

We'll leverage the [Rig framework](https://docs.rs/rig-core) to implement our agent architecture. Rig provides the foundations for building LLM-powered agents in Rust with strong typing and performance.

### Agent Implementation

```rust
use rig::{
    agent::AgentBuilder,
    completion::Prompt,
    providers::llm_provider
};

// Common agent setup pattern
pub fn create_commit_agent(
    config: &Config,
    diff: &str,
) -> Agent<impl CompletionModel> {
    let provider = get_provider_from_config(config);
    let model = provider.completion_model(config.model_name());
    
    AgentBuilder::new(model)
        .preamble(COMMIT_AGENT_PROMPT)
        .context(format!("DIFF:\n{}", diff))
        .tool(FileReaderTool::new())
        .tool(DirectoryListTool::new())
        .tool(CodeSearchTool::new())
        .tool(GitHistoryTool::new())
        .temperature(config.temperature())
        .build()
}
```

### Tool Implementation

Each tool will implement the `Tool` trait from Rig:

```rust
use rig::tool::{Tool, ToolError};

pub struct FileReaderTool {
    // Implementation details
}

impl Tool for FileReaderTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Reads the contents of a specified file"
    }
    
    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file, relative to the repository root"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn run(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        // Implementation
    }
}
```

### Core Tools

We'll implement these foundational tools:

1. **FileReader**: Read specific files from the repository
   - Path validation and security boundaries
   - Smart chunking for large files
   - Syntax highlighting and metadata enrichment

2. **DirectoryLister**: List contents of directories
   - Filtering by file type, name patterns
   - Recursive exploration with depth control
   - Directory structure visualization

3. **CodeSearcher**: Search for patterns or symbols in the codebase
   - Regex and semantic search capabilities
   - Language-aware context extraction
   - Token-aware result pagination

4. **DiffAnalyzer**: Analyze changes in a specific file
   - Line-by-line diff visualization
   - Semantic change categorization
   - Impact analysis on dependencies

5. **GitHistoryReader**: Examine commit history for relevant information
   - Author and time-range filtering
   - Commit message analysis
   - Code evolution tracking

6. **RepositoryMetaReader**: Access metadata about the repository
   - Configuration file analysis
   - Dependency management
   - Project structure insights

### Task-Specific Tools

These specialized tools will be made available based on the task:

1. **CommitAnalyzer**: Analyze commit messages for patterns (for commit agent)
   - Conventional commit parsing
   - Historical style matching
   - Quality assessment

2. **CodeQualityAnalyzer**: Identify code quality issues (for review agent)
   - Pattern-based anti-pattern detection
   - Complexity analysis
   - Style consistency checking

3. **BreakingChangeDetector**: Detect API changes (for changelog agent)
   - Interface signature comparison
   - Public API tracking
   - Backward compatibility assessment

4. **DependencyAnalyzer**: Analyze project dependencies (for changelog/review agents)
   - Dependency graph visualization
   - Version compatibility checking
   - Security vulnerability scanning

## MCP Integration

The Model Context Protocol (MCP) integration will be re-architected to align with our agent-based approach. Each MCP tool will be mapped to a specialized agent:

```
MCP Tool                  Git-Iris Agent
-----------------------------------------
git_iris_commit       →   Commit Agent
git_iris_code_review  →   Review Agent
git_iris_changelog    →   Changelog Agent
git_iris_release_notes→   Changelog Agent
```

The MCP server will:
1. Receive tool calls from clients
2. Initialize the appropriate agent
3. Provide the agent with necessary context
4. Return the agent's output to the client
5. Support streaming responses for long-running tasks

## Performance Optimizations

To ensure responsiveness:

1. **Parallel Tool Execution**: When possible, execute multiple tool calls in parallel
   - Independent tool call batching
   - Priority-based execution queuing
   - Resource-aware scheduling

2. **Context Caching**: Cache results of expensive tool calls for reuse
   - Time-based cache invalidation
   - Dependency-aware cache refreshing
   - Shared cache across agent instances

3. **Progressive Generation**: Stream results as they become available
   - Incremental context assembly
   - Early output generation
   - Continuous refinement

4. **Adaptive Exploration**: Use heuristics to limit unnecessary exploration
   - Relevance scoring for exploration paths
   - Diminishing returns detection
   - Exploration depth budgeting

5. **Token Budget Management**: Track token usage and optimize queries
   - Dynamic context prioritization
   - Compression techniques for context
   - Important information highlighting

## Metrics and Monitoring

To ensure our agent-based system delivers on its promises, we'll implement comprehensive metrics and monitoring:

### Key Metrics to Track

1. **Performance Metrics**:
   - Average response time per agent type
   - Token usage per agent interaction
   - Tool call frequency and latency
   - Cache hit/miss ratio for context requests
   - Memory usage and garbage collection patterns

2. **Quality Metrics**:
   - Output quality scoring (via separate evaluation)
   - User satisfaction ratings
   - Error rates and recovery success
   - Context relevance assessment
   - Hallucination frequency detection

3. **Exploration Metrics**:
   - Depth of exploration per task
   - Files accessed vs. files available ratio
   - Tool call patterns and efficiency
   - Path redundancy in exploration
   - Time spent in exploration vs. generation

### Monitoring Infrastructure

We'll implement:
1. **Telemetry Pipeline**: Collect detailed usage statistics
2. **Performance Dashboard**: Real-time visualization of system performance
3. **A/B Testing Framework**: Compare different agent configurations
4. **Anomaly Detection**: Identify and alert on unusual behavior patterns
5. **User Feedback Loop**: Capture and incorporate user satisfaction data

## Technical Debt Considerations

As we transition to an agent-based architecture, we need to be mindful of potential technical debt:

1. **Legacy System Compatibility**: Ensure backward compatibility during migration
2. **Prompt Engineering Maintenance**: Develop systems to maintain and version agent prompts
3. **Tool Proliferation**: Avoid creating redundant or overlapping tools
4. **Provider Lock-in**: Maintain abstraction layers for LLM provider flexibility
5. **Testing Complexity**: Develop strategies for testing non-deterministic agent behavior
6. **Error Handling Consistency**: Establish patterns for error recovery across components
7. **Documentation Overhead**: Keep documentation in sync with rapidly evolving components

We'll address these concerns through:
- Regular architecture reviews
- Comprehensive test coverage
- Careful documentation of design decisions
- Planned refactoring cycles
- Clear component boundaries and interfaces
- Automated testing and validation pipelines

## Implementation Roadmap

The detailed implementation plan for Git-Iris 2.0 is maintained in a separate document: [Implementation Checklist](implementation-checklist.md).

The implementation is divided into four main phases:

1. **Foundation**: Building the core architecture and fundamental tools
   - Tool registry implementation
   - Agent orchestrator framework
   - Basic tool set development
   - Initial agent prototypes

2. **Agent Development**: Implementing specialized agents with advanced capabilities
   - Agent communication protocols
   - Specialized agent implementation
   - Advanced tool development
   - Agent collaboration mechanisms

3. **MCP & User Experience**: Reimplementing the MCP integration and enhancing UX
   - MCP server redesign
   - CLI improvements
   - Performance optimizations
   - User feedback integration

4. **Testing & Refinement**: Comprehensive testing and final refinements
   - Test suite development
   - Documentation updates
   - Performance tuning
   - Final release preparation

For detailed tasks, status tracking, and implementation notes, refer to the implementation checklist document.

## Transitioning from llms to Rig

Git-Iris currently utilizes the `llms` crate as a flexible abstraction layer for interfacing with various LLM providers. Moving to Rig represents a significant architecture change that offers several advantages while requiring careful migration.

### Rig vs. llms Comparison

| Feature | llms | Rig |
|---------|------|-----|
| Provider Support | OpenAI, Anthropic, Google, etc. | OpenAI, Anthropic, Cohere, Perplexity |
| Type Safety | Basic | Enhanced with stronger type guarantees |
| Agent Support | Limited, requires custom implementation | First-class, built-in agent framework |
| RAG Capabilities | Not included, requires separate implementation | Integrated vector stores and embedding support |
| Streaming | Basic support | Native, first-class streaming support |
| Tool Integration | Not included | Built-in tool framework |
| Error Handling | Basic | Comprehensive with recovery strategies |
| Caching | Not included | Built-in context caching |

### Migration Strategy

#### 1. Provider Adapter Layer

We'll begin by creating a compatibility layer that adapts our existing provider configurations to Rig's provider system:

```rust
pub fn adapt_provider(provider_config: &ProviderConfig) -> Box<dyn RigCompatibleProvider> {
    match provider_config.provider_type() {
        ProviderType::OpenAI => {
            let client = openai::Client::new(provider_config.api_key());
            Box::new(OpenAIAdapter { client })
        },
        ProviderType::Anthropic => {
            let client = anthropic::Client::new(provider_config.api_key());
            Box::new(AnthropicAdapter { client })
        },
        // Other providers...
    }
}
```

#### 2. Gradual Feature Migration

We'll migrate features from the current architecture to Rig in this order:

1. **Basic Completion API**: Ensure we can get equivalent results with Rig
2. **Provider Authentication**: Migrate all provider auth patterns
3. **Tool Implementation**: Recreate our current tools using Rig's tool framework
4. **Agent Implementation**: Build specialized agents using Rig's agent system

#### 3. Dual-Mode Operation

During transition, we'll support both systems simultaneously:

```rust
pub enum CompletionBackend {
    Legacy(llms::CompletionProvider),
    Rig(rig::Agent<impl CompletionModel>)
}

impl CompletionBackend {
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        match self {
            Self::Legacy(provider) => provider.complete(prompt).await,
            Self::Rig(agent) => {
                let response = agent.prompt(prompt).await?;
                Ok(response.to_string())
            }
        }
    }
}
```

#### 4. Testing & Validation

For each migrated component:
- Create parity tests to ensure consistency
- Benchmark performance differences
- Validate output quality between implementations
- Implement canary deployments to detect issues early

#### 5. Dependency Updates

We'll need to carefully manage our dependencies during transition:
- Add Rig as a dependency
- Maintain llms until migration is complete
- Eventually remove llms dependency
- Update other dependent crates
- Ensure compatibility with the broader ecosystem

### Key Rig Features to Leverage

In our transition, we'll focus on these key Rig advantages:

1. **Native Agent Framework**: Rig's agent system eliminates our need to build custom agent infrastructure, providing a solid foundation that handles:
   - Tool discovery and execution
   - Context management
   - Agent state tracking
   - Reasoning step tracking

2. **Built-in RAG Support**: Rig provides integrated vector database and embedding functionality that we can leverage for:
   - Semantic code search
   - Relevant file discovery
   - Similar code pattern identification
   - Intelligent context prioritization

3. **Provider Simplification**: Rig offers a cleaner API for working with LLM providers:
   - Consistent error handling
   - Simplified authentication
   - Standardized parameter passing
   - Automatic model detection
   - Capability-based provider selection

4. **Dynamic Context Management**: Rig's ability to handle dynamic contexts will allow us to:
   - Progressively load relevant files
   - Automatically manage token limits
   - Intelligently prioritize context elements
   - Track context utilization

## Migration Strategy

We'll implement a phased migration approach:

1. **Dual-Mode Operation**: Support both current and agent-based approaches with a configuration toggle
2. **Feature Parity**: Ensure all existing functionality works with the new approach
3. **Gradual Transition**: Move features one at a time to the agent-based approach
4. **Performance Benchmarking**: Compare results and performance between approaches
5. **User Feedback Collection**: Gather input on quality and performance differences

## Expected Outcomes

The agent-based architecture is expected to deliver:

1. **More Relevant Outputs**: By focusing on what matters for the task
2. **Faster Performance**: By reducing unnecessary context gathering
3. **Higher Quality**: Better understanding of codebase relationships
4. **Improved Scalability**: Better handling of large repositories
5. **More Maintainable Codebase**: Clearer separation of concerns
6. **Enhanced Extensibility**: Easier to add new capabilities
7. **Finer Control**: More precise configuration options for advanced users

## Conclusion

Git-Iris 2.0's agent-based architecture represents a significant advancement in how AI interacts with codebases. By moving from bulk context loading to intelligent exploration, we'll deliver a more responsive, accurate, and powerful developer tool.

This approach aligns with the cutting edge of AI agent research while maintaining Git-Iris's commitment to performance, privacy, and user experience. The implementation checklist provides a clear path forward, with measurable milestones to track our progress toward this vision. 