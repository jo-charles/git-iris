//! LLM Service Layer
//!
//! Extracted from the monolithic `IrisAgent` to handle all LLM communication patterns
//! with unified streaming support and consistent error handling.

#![allow(clippy::unused_self)]

use anyhow::Result;
use futures::StreamExt;
use regex::Regex;
use rig::completion::Prompt;
use rig::prelude::*;
use rig::streaming::StreamingPrompt;
use std::sync::LazyLock;

use crate::agents::{
    core::AgentBackend,
    iris::StreamingCallback,
    status::{IRIS_STATUS, IrisPhase, IrisStatus, TokenMetrics},
};

// Token limit for all operations - Claude can handle much more than 8192 tokens
const DEFAULT_MAX_TOKENS: u64 = 8192;

// Compiled regex patterns for status extraction (performance optimization)
static STATUS_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"STATUS:\s*(.+)").expect("Failed to compile STATUS regex pattern"),
        Regex::new(r"NEXT_STATUS:\s*(.+)").expect("Failed to compile NEXT_STATUS regex pattern"),
        Regex::new(r"Current status:\s*(.+)")
            .expect("Failed to compile Current status regex pattern"),
        Regex::new(r"Status:\s*(.+)").expect("Failed to compile Status regex pattern"),
    ]
});

/// Model type selection for different complexity tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    /// Primary model for complex analysis, code generation, nuanced decisions
    Primary,
    /// Fast model for status updates, simple parsing, tool planning
    Fast,
}

impl ModelType {
    /// Determine model type based on operation characteristics
    pub fn from_operation(operation_type: &str, phase: &IrisPhase) -> Self {
        let op_lower = operation_type.to_lowercase();

        match op_lower.as_str() {
            // Fast operations - simple tasks that don't need deep reasoning
            "status" | "status_update" | "status_generation" | "parsing" | "parse"
            | "extraction" | "simple_planning" | "tool_planning" | "validation"
            | "format_check" | "summary" | "summarization" | "list" | "listing" | "enumerate" => {
                ModelType::Fast
            }

            // Analysis tasks - use fast model for simple analysis, primary for complex
            "analysis" => {
                // Use fast model for tool planning and simple context analysis
                match phase {
                    IrisPhase::Planning | IrisPhase::ToolExecution { .. } => ModelType::Fast,
                    _ => ModelType::Primary,
                }
            }

            // Generation tasks - use fast for simple, primary for complex
            "generation" => {
                // Use fast model for tool planning and status generation
                match phase {
                    IrisPhase::Planning | IrisPhase::Initializing => ModelType::Fast,
                    _ => ModelType::Primary,
                }
            }

            // Complex operations that always need primary model
            "commit_generation"
            | "commit_message_generation"
            | "code_review"
            | "review_generation"
            | "changelog_generation"
            | "pr_generation"
            | "pull_request_generation"
            | "synthesis"
            | "context_synthesis" => ModelType::Primary,

            // Phase-based fallback with more aggressive fast model usage
            _ => match phase {
                // Fast model for planning and initialization phases
                IrisPhase::Initializing
                | IrisPhase::Planning
                | IrisPhase::PlanExpansion
                | IrisPhase::Completed
                | IrisPhase::Error(_) => ModelType::Fast,

                // Fast model for tool execution unless it's complex synthesis
                IrisPhase::ToolExecution { .. } => {
                    if op_lower.contains("synthesis") || op_lower.contains("complex") {
                        ModelType::Primary
                    } else {
                        ModelType::Fast
                    }
                }

                // Primary model for final analysis and generation
                IrisPhase::Synthesis | IrisPhase::Analysis | IrisPhase::Generation => {
                    ModelType::Primary
                }
            },
        }
    }
}

/// LLM service for unified language model operations
#[derive(Clone)]
pub struct LLMService {
    backend: AgentBackend,
}

/// Request for LLM generation with all necessary parameters
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
    pub max_tokens: u64,
    pub phase: IrisPhase,
    pub operation_type: String,
    pub context_hint: String,
    pub current_step: usize,
    pub total_steps: Option<usize>,
    pub model_type: ModelType,
}

impl GenerationRequest {
    pub fn new(system_prompt: String, user_prompt: String) -> Self {
        Self {
            system_prompt,
            user_prompt,
            temperature: 0.7,
            max_tokens: DEFAULT_MAX_TOKENS,
            phase: IrisPhase::Generation,
            operation_type: "generation".to_string(),
            context_hint: "processing request".to_string(),
            current_step: 1,
            total_steps: None,
            model_type: ModelType::from_operation("generation", &IrisPhase::Generation),
        }
    }

    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    #[must_use]
    pub fn with_phase(mut self, phase: IrisPhase) -> Self {
        self.model_type = ModelType::from_operation(&self.operation_type, &phase);
        self.phase = phase;
        self
    }

    #[must_use]
    pub fn with_operation(mut self, operation_type: String, context_hint: String) -> Self {
        self.model_type = ModelType::from_operation(&operation_type, &self.phase);
        self.operation_type = operation_type;
        self.context_hint = context_hint;
        self
    }

    #[must_use]
    pub fn with_context(mut self, context: &str) -> Self {
        self.context_hint = context.to_string();
        self
    }

    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u64) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    #[must_use]
    pub fn with_model_type(mut self, model_type: ModelType) -> Self {
        self.model_type = model_type;
        self
    }

    #[must_use]
    pub fn with_streaming_callback(self, _callback: &dyn StreamingCallback) -> Self {
        // For now, we'll just return self as streaming callback is handled in the service layer
        self
    }

    #[must_use]
    pub fn with_progress(mut self, current_step: usize, total_steps: Option<usize>) -> Self {
        self.current_step = current_step;
        self.total_steps = total_steps;
        self
    }

    /// Builder pattern for creating generation requests
    pub fn builder() -> GenerationRequestBuilder {
        GenerationRequestBuilder::new()
    }
}

/// Builder for creating generation requests
pub struct GenerationRequestBuilder {
    system_prompt: Option<String>,
    user_prompt: Option<String>,
    temperature: f32,
    phase: IrisPhase,
    operation_type: String,
    context_hint: String,
}

impl Default for GenerationRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationRequestBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: None,
            user_prompt: None,
            temperature: 0.7,
            phase: IrisPhase::Generation,
            operation_type: "generation".to_string(),
            context_hint: "processing request".to_string(),
        }
    }

    #[must_use]
    pub fn system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    #[must_use]
    pub fn user_prompt(mut self, prompt: String) -> Self {
        self.user_prompt = Some(prompt);
        self
    }

    #[must_use]
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    #[must_use]
    pub fn phase(mut self, phase: IrisPhase) -> Self {
        self.phase = phase;
        self
    }

    #[must_use]
    pub fn with_context(mut self, context: &str) -> Self {
        self.context_hint = context.to_string();
        self
    }

    #[must_use]
    pub fn context_hint(mut self, hint: &str) -> Self {
        self.context_hint = hint.to_string();
        self
    }

    #[must_use]
    pub fn operation_type(mut self, operation: &str) -> Self {
        self.operation_type = operation.to_string();
        self
    }

    #[must_use]
    pub fn current_step(mut self, step: usize) -> Self {
        // For now, we'll just store it in the context hint
        self.context_hint = format!("step {step}");
        self
    }

    #[must_use]
    pub fn total_steps(mut self, total: Option<usize>) -> Self {
        // For now, we'll just store it in the context hint
        if let Some(total) = total {
            self.context_hint = format!("{} of {}", self.context_hint, total);
        }
        self
    }

    #[must_use]
    pub fn with_streaming_callback(self, _callback: &dyn StreamingCallback) -> Self {
        // For now, we'll just return self as streaming is handled elsewhere
        self
    }

    pub fn build(self) -> Result<GenerationRequest> {
        let system_prompt = self
            .system_prompt
            .ok_or_else(|| anyhow::anyhow!("System prompt is required for generation request"))?;
        let user_prompt = self
            .user_prompt
            .ok_or_else(|| anyhow::anyhow!("User prompt is required for generation request"))?;

        Ok(GenerationRequest::new(system_prompt, user_prompt)
            .with_temperature(self.temperature)
            .with_phase(self.phase)
            .with_operation(self.operation_type, self.context_hint))
    }
}

impl LLMService {
    pub fn new(backend: AgentBackend) -> Self {
        Self { backend }
    }

    /// Generate text with LLM - unified pipeline for all agent operations
    pub async fn generate(&self, request: GenerationRequest) -> Result<String> {
        self.generate_internal(request, None).await
    }

    /// Generate text using the fast model (optimized for speed over quality)
    pub async fn fast_generate(&self, request: GenerationRequest) -> Result<String> {
        let fast_request = GenerationRequest {
            model_type: ModelType::Fast,
            ..request
        };
        self.generate_internal(fast_request, None).await
    }

    /// Generate text with streaming support for real-time feedback
    pub async fn generate_streaming(
        &self,
        request: GenerationRequest,
        callback: &dyn StreamingCallback,
    ) -> Result<String> {
        self.generate_internal(request, Some(callback)).await
    }

    /// Analyze context using optimized analysis configuration (uses primary model)
    pub async fn analyze(&self, prompt: &str) -> Result<String> {
        let system_prompt = "You are Iris, an expert AI assistant specializing in Git workflow automation and code analysis. \
                            Provide intelligent, structured analysis in the requested JSON format. \
                            You have deep understanding of software development patterns and can provide insightful analysis.";

        let request = GenerationRequest::new(system_prompt.to_string(), prompt.to_string())
            .with_temperature(0.3) // Lower temperature for consistent analysis
            .with_phase(IrisPhase::Analysis)
            .with_operation("analysis".to_string(), "gathering context".to_string());

        self.generate(request).await
    }

    /// Fast analysis using the fast model for simple parsing/extraction tasks
    pub async fn fast_analyze(&self, prompt: &str) -> Result<String> {
        let system_prompt = "You are Iris, a helpful AI assistant. Provide concise, structured responses in the requested format.";

        let request = GenerationRequest::new(system_prompt.to_string(), prompt.to_string())
            .with_temperature(0.1) // Very low temperature for consistent parsing
            .with_phase(IrisPhase::Planning)
            .with_operation("parsing".to_string(), "extracting information".to_string())
            .with_model_type(ModelType::Fast);

        self.generate(request).await
    }

    /// Internal generation pipeline with optional streaming
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::as_conversions
    )]
    #[allow(unused_assignments)]
    #[allow(clippy::too_many_lines)]
    async fn generate_internal(
        &self,
        request: GenerationRequest,
        callback: Option<&dyn StreamingCallback>,
    ) -> Result<String> {
        // Track call metrics
        let call_start = std::time::Instant::now();

        // Determine selected model ahead of time for logging
        let (selected_model, model_type_str) = match &self.backend {
            AgentBackend::OpenAI {
                model, fast_model, ..
            }
            | AgentBackend::Anthropic {
                model, fast_model, ..
            } => match request.model_type {
                ModelType::Primary => (model.as_str(), "Primary"),
                ModelType::Fast => (fast_model.as_str(), "Fast"),
            },
        };

        crate::log_debug!(
            "🚀 LLM Call Started | Model: {} ({}) | Operation: {} | Phase: {:?}",
            selected_model,
            model_type_str,
            request.operation_type,
            request.phase
        );

        // Step 1: Sanitize inputs to prevent prompt injection
        let safe_operation_type = Self::sanitize_status_input(&request.operation_type);
        let safe_context_hint = Self::sanitize_status_input(&request.context_hint);

        // Step 2: Add status instruction for LLM-generated progress messages
        let status_instruction = format!(
            "Include: STATUS: [progress message under 70 characters]...\n\
            Context: {} for {}\n\
            Style: Professional with subtle personality - clever but not mystical.\n\
            Examples: 🔍 Dissecting your code... 🧠 Connecting the dots... 🎯 Zeroing in on the perfect solution...",
            &safe_operation_type, &safe_context_hint
        );
        let enhanced_system_prompt = format!("{}\n\n{}", request.system_prompt, status_instruction);

        crate::log_debug!("🌟 LLM Service: Added status instruction for dynamic messages");

        // Step 3: Generate with the right pipeline based on streaming preference
        let mut full_response = String::new();
        let mut final_token_metrics = TokenMetrics::default();

        match &self.backend {
            AgentBackend::OpenAI {
                client,
                model,
                fast_model,
            } => {
                // Select model based on complexity
                let selected_model = match request.model_type {
                    ModelType::Primary => model,
                    ModelType::Fast => fast_model,
                };

                let agent = client
                    .agent(selected_model)
                    .preamble(&enhanced_system_prompt)
                    .temperature(f64::from(request.temperature))
                    .max_tokens(request.max_tokens)
                    .build();

                if let Some(callback) = callback {
                    // Streaming pipeline - stable status updates only
                    let mut stream = agent.stream_prompt(&request.user_prompt).await?;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(assistant_content) => {
                                if let rig::completion::AssistantContent::Text(text) =
                                    assistant_content
                                {
                                    if !text.text.is_empty() {
                                        // Create token metrics for streaming
                                        let token_metrics = TokenMetrics {
                                            input_tokens: 0, // Would need actual API token count
                                            output_tokens: full_response.len() as u32
                                                + text.text.len() as u32,
                                            total_tokens: full_response.len() as u32
                                                + text.text.len() as u32,
                                            tokens_per_second: 0.0, // Would be calculated by callback
                                            estimated_remaining: None,
                                        };
                                        callback
                                            .on_chunk(&text.text, Some(token_metrics.clone()))
                                            .await?;
                                        full_response.push_str(&text.text);

                                        // Skip real-time status extraction to avoid chaos
                                    }
                                }
                            }
                            Err(e) => {
                                let anyhow_error = anyhow::anyhow!("Streaming error: {}", e);
                                callback.on_error(&anyhow_error).await?;
                                return Err(e.into());
                            }
                        }
                    }
                    // Create final token metrics
                    final_token_metrics = TokenMetrics {
                        input_tokens: 0, // Would need actual API token count
                        output_tokens: full_response.len() as u32,
                        total_tokens: full_response.len() as u32,
                        tokens_per_second: 0.0,
                        estimated_remaining: None,
                    };
                    callback
                        .on_complete(&full_response, final_token_metrics.clone())
                        .await?;
                } else {
                    // Non-streaming pipeline
                    full_response = agent.prompt(&request.user_prompt).await?;
                    // Estimate token metrics for non-streaming
                    final_token_metrics = TokenMetrics {
                        input_tokens: (request.system_prompt.len() + request.user_prompt.len())
                            as u32
                            / 4, // Rough estimate: 4 chars per token
                        output_tokens: full_response.len() as u32 / 4,
                        total_tokens: (request.system_prompt.len()
                            + request.user_prompt.len()
                            + full_response.len()) as u32
                            / 4,
                        tokens_per_second: 0.0,
                        estimated_remaining: None,
                    };
                }
            }
            AgentBackend::Anthropic {
                client,
                model,
                fast_model,
            } => {
                // Select model based on complexity
                let selected_model = match request.model_type {
                    ModelType::Primary => model,
                    ModelType::Fast => fast_model,
                };

                let agent = client
                    .agent(selected_model)
                    .preamble(&enhanced_system_prompt)
                    .temperature(f64::from(request.temperature))
                    .max_tokens(request.max_tokens)
                    .build();

                if let Some(callback) = callback {
                    // Streaming pipeline - stable status updates only
                    let mut stream = agent.stream_prompt(&request.user_prompt).await?;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(assistant_content) => {
                                if let rig::completion::AssistantContent::Text(text) =
                                    assistant_content
                                {
                                    if !text.text.is_empty() {
                                        // Create token metrics for streaming
                                        let token_metrics = TokenMetrics {
                                            input_tokens: 0, // Would need actual API token count
                                            output_tokens: full_response.len() as u32
                                                + text.text.len() as u32,
                                            total_tokens: full_response.len() as u32
                                                + text.text.len() as u32,
                                            tokens_per_second: 0.0, // Would be calculated by callback
                                            estimated_remaining: None,
                                        };
                                        callback
                                            .on_chunk(&text.text, Some(token_metrics.clone()))
                                            .await?;
                                        full_response.push_str(&text.text);

                                        // Skip real-time status extraction to avoid chaos
                                    }
                                }
                            }
                            Err(e) => {
                                let anyhow_error = anyhow::anyhow!("Streaming error: {}", e);
                                callback.on_error(&anyhow_error).await?;
                                return Err(e.into());
                            }
                        }
                    }
                    // Create final token metrics
                    final_token_metrics = TokenMetrics {
                        input_tokens: 0, // Would need actual API token count
                        output_tokens: full_response.len() as u32,
                        total_tokens: full_response.len() as u32,
                        tokens_per_second: 0.0,
                        estimated_remaining: None,
                    };
                    callback
                        .on_complete(&full_response, final_token_metrics.clone())
                        .await?;
                } else {
                    // Non-streaming pipeline
                    full_response = agent.prompt(&request.user_prompt).await?;
                    // Estimate token metrics for non-streaming
                    final_token_metrics = TokenMetrics {
                        input_tokens: (request.system_prompt.len() + request.user_prompt.len())
                            as u32
                            / 4,
                        output_tokens: full_response.len() as u32 / 4,
                        total_tokens: (request.system_prompt.len()
                            + request.user_prompt.len()
                            + full_response.len()) as u32
                            / 4,
                        tokens_per_second: 0.0,
                        estimated_remaining: None,
                    };
                }
            }
        }

        // Calculate call duration and performance metrics
        let call_duration = call_start.elapsed();
        let tokens_per_second = if call_duration.as_secs_f32() > 0.0 {
            final_token_metrics.output_tokens as f32 / call_duration.as_secs_f32()
        } else {
            0.0
        };

        // Comprehensive logging of LLM call completion
        crate::log_debug!(
            "✅ LLM Call Completed | Model: {} ({}) | Duration: {:.2}s | Tokens: in={} out={} total={} | Speed: {:.1}t/s | Operation: {} | Response length: {} chars",
            selected_model,
            model_type_str,
            call_duration.as_secs_f32(),
            final_token_metrics.input_tokens,
            final_token_metrics.output_tokens,
            final_token_metrics.total_tokens,
            tokens_per_second,
            request.operation_type,
            full_response.len()
        );

        // Log cost estimation for major providers
        let estimated_cost = estimate_call_cost(selected_model, &final_token_metrics);
        if estimated_cost > 0.0 {
            crate::log_debug!(
                "💰 Estimated cost: ${:.4} | Model: {} | Operation: {}",
                estimated_cost,
                selected_model,
                request.operation_type
            );
        }

        // Step 4: Extract status for non-streaming requests
        if callback.is_none() {
            crate::log_debug!("🌟 LLM Service: Extracting status from non-streaming response");
            self.update_status_from_response(&full_response, &request);
        } else {
            crate::log_debug!("🌟 LLM Service: Skipping status extraction for streaming request");
        }

        Ok(full_response.trim().to_string())
    }

    /// Sanitize status input to prevent prompt injection
    fn sanitize_status_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .take(100) // Limit to 100 characters
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Extract status message from LLM response that includes status in structured format
    fn extract_status_from_response(response: &str, phase: &IrisPhase) -> String {
        crate::log_debug!(
            "🔍 LLM Service: Searching for status patterns in response snippet: '{}'",
            &response.chars().take(200).collect::<String>()
        );

        // Use precompiled regex patterns for better performance
        for (i, re) in STATUS_PATTERNS.iter().enumerate() {
            if let Some(captures) = re.captures(response) {
                if let Some(status_match) = captures.get(1) {
                    let status = status_match.as_str().trim().to_string();
                    if !status.is_empty() && status.len() > 5 {
                        crate::log_debug!(
                            "🎯 LLM Service: Found status with pattern {}: '{}'",
                            i,
                            status
                        );
                        return status;
                    }
                }
            }
        }

        crate::log_debug!(
            "⚠️ LLM Service: No status patterns found, using fallback for phase: {:?}",
            phase
        );

        // Fallback to engaging progress message if no status found in response
        match phase {
            IrisPhase::Planning => "🧠 Planning approach...".to_string(),
            IrisPhase::ToolExecution { tool_name, .. } => {
                format!("🔧 Running {tool_name}...")
            }
            IrisPhase::Analysis => "🔍 Analyzing...".to_string(),
            IrisPhase::Synthesis => "🧩 Connecting insights...".to_string(),
            IrisPhase::Generation => "🎯 Generating...".to_string(),
            IrisPhase::PlanExpansion => "📈 Expanding analysis...".to_string(),
            _ => "⚙️ Processing...".to_string(),
        }
    }

    /// Update status with a dynamic message extracted from the LLM response
    fn update_status_from_response(&self, response: &str, request: &GenerationRequest) {
        crate::log_debug!(
            "🌟 LLM Service: Extracting status from response of {} chars",
            response.len()
        );
        let message = Self::extract_status_from_response(response, &request.phase);
        crate::log_debug!(
            "🌟 LLM Service: Extracted status message: '{}' for phase: {:?}",
            message,
            request.phase
        );

        IRIS_STATUS.update(IrisStatus::dynamic(
            request.phase.clone(),
            message.clone(),
            request.current_step,
            request.total_steps,
        ));

        crate::log_debug!("🌟 LLM Service: Updated status display with: '{}'", message);

        // Brief pause for display, then move on
        std::thread::sleep(std::time::Duration::from_millis(500));
        crate::log_debug!("🌟 LLM Service: Status display pause completed");
    }
}

/// Estimate the cost of an LLM call based on model and token usage
/// Returns estimated cost in USD, or 0.0 if model pricing is unknown
fn estimate_call_cost(model: &str, tokens: &TokenMetrics) -> f64 {
    // Pricing per 1M tokens (as of 2024) - these are approximate and change frequently
    let (input_cost_per_million, output_cost_per_million) = match model.to_lowercase().as_str() {
        // Anthropic Claude models
        "claude-sonnet-4-20250514" | "claude-3-5-sonnet-20241022" => (3.0, 15.0),
        "claude-3-5-haiku-latest" | "claude-3-5-haiku-20241022" => (1.0, 5.0),
        "claude-3-opus-20240229" => (15.0, 75.0),

        // OpenAI models
        "gpt-4o" | "gpt-4o-2024-08-06" => (2.5, 10.0),
        "gpt-4o-mini" | "gpt-4o-mini-2024-07-18" => (0.15, 0.6),
        "gpt-4-turbo" => (10.0, 30.0),
        "gpt-3.5-turbo" => (0.5, 1.5),

        // Google models (Gemini)
        "gemini-2.5-pro-preview-06-05" | "gemini-pro" => (1.25, 5.0),
        "gemini-flash" => (0.075, 0.3),

        // Unknown models
        _ => return 0.0,
    };

    let input_cost = (f64::from(tokens.input_tokens) / 1_000_000.0) * input_cost_per_million;
    let output_cost = (f64::from(tokens.output_tokens) / 1_000_000.0) * output_cost_per_million;

    input_cost + output_cost
}
