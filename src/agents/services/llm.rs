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
    status::{IRIS_STATUS, IrisPhase, IrisStatus},
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
        }
    }

    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    #[must_use]
    pub fn with_phase(mut self, phase: IrisPhase) -> Self {
        self.phase = phase;
        self
    }

    #[must_use]
    pub fn with_operation(mut self, operation_type: String, context_hint: String) -> Self {
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

    /// Generate text with streaming support for real-time feedback
    pub async fn generate_streaming(
        &self,
        request: GenerationRequest,
        callback: &dyn StreamingCallback,
    ) -> Result<String> {
        self.generate_internal(request, Some(callback)).await
    }

    /// Analyze context using optimized analysis configuration
    pub async fn analyze(&self, prompt: &str) -> Result<String> {
        let system_prompt = "You are Iris, an expert AI assistant specializing in Git workflow automation and code analysis. \
                            Provide intelligent, structured analysis in the requested JSON format. \
                            You have deep understanding of software development patterns and can provide insightful analysis.";

        let request = GenerationRequest::new(system_prompt.to_string(), prompt.to_string())
            .with_temperature(0.3) // Lower temperature for consistent analysis
            .with_phase(IrisPhase::Analysis)
            .with_operation("analysis".to_string(), "analyzing context".to_string());

        self.generate(request).await
    }

    /// Internal generation pipeline with optional streaming
    async fn generate_internal(
        &self,
        request: GenerationRequest,
        callback: Option<&dyn StreamingCallback>,
    ) -> Result<String> {
        // Step 1: Sanitize inputs to prevent prompt injection
        let safe_operation_type = Self::sanitize_status_input(&request.operation_type);
        let safe_context_hint = Self::sanitize_status_input(&request.context_hint);

        // Step 2: Add dynamic status instruction to the system prompt
        let status_instruction = self.create_status_instruction(
            &request.phase,
            &safe_operation_type,
            &safe_context_hint,
        );
        let enhanced_system_prompt = format!("{}\n\n{}", request.system_prompt, status_instruction);

        // Step 3: Generate with the right pipeline based on streaming preference
        let mut full_response = String::new();

        match &self.backend {
            AgentBackend::OpenAI { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(&enhanced_system_prompt)
                    .temperature(f64::from(request.temperature))
                    .max_tokens(request.max_tokens)
                    .build();

                if let Some(callback) = callback {
                    // Streaming pipeline with real-time status updates
                    let mut stream = agent.stream_prompt(&request.user_prompt).await?;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(assistant_content) => {
                                if let rig::completion::AssistantContent::Text(text) =
                                    assistant_content
                                {
                                    if !text.text.is_empty() {
                                        callback.on_chunk(&text.text).await?;
                                        full_response.push_str(&text.text);

                                        // Real-time status extraction and update
                                        Self::check_and_update_streaming_status(
                                            &text.text, &request,
                                        );
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
                    callback.on_complete(&full_response).await?;
                } else {
                    // Non-streaming pipeline
                    full_response = agent.prompt(&request.user_prompt).await?;
                }
            }
            AgentBackend::Anthropic { client, model } => {
                let agent = client
                    .agent(model)
                    .preamble(&enhanced_system_prompt)
                    .temperature(f64::from(request.temperature))
                    .max_tokens(request.max_tokens)
                    .build();

                if let Some(callback) = callback {
                    // Streaming pipeline with real-time status updates
                    let mut stream = agent.stream_prompt(&request.user_prompt).await?;
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(assistant_content) => {
                                if let rig::completion::AssistantContent::Text(text) =
                                    assistant_content
                                {
                                    if !text.text.is_empty() {
                                        callback.on_chunk(&text.text).await?;
                                        full_response.push_str(&text.text);

                                        // Real-time status extraction and update
                                        Self::check_and_update_streaming_status(
                                            &text.text, &request,
                                        );
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
                    callback.on_complete(&full_response).await?;
                } else {
                    // Non-streaming pipeline
                    full_response = agent.prompt(&request.user_prompt).await?;
                }
            }
        }

        // Step 4: Final status extraction and update
        self.update_status_from_response(&full_response, &request);

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

    /// Create status instruction to add to prompts
    fn create_status_instruction(
        &self,
        phase: &IrisPhase,
        operation_type: &str,
        context_hint: &str,
    ) -> String {
        let phase_description = match phase {
            IrisPhase::Planning => "orchestrating cosmic alignment of code patterns",
            IrisPhase::ToolExecution { tool_name, .. } => {
                &format!("channeling the essence of {tool_name}")
            }
            IrisPhase::Analysis => "diving into the quantum depths of code reality",
            IrisPhase::Synthesis => "weaving interdimensional insights into coherent understanding",
            IrisPhase::Generation => "manifesting the perfect algorithmic expression",
            IrisPhase::PlanExpansion => {
                "recalibrating cosmic understanding through discovered wisdom"
            }
            _ => "traversing the digital astral plane",
        };

        format!(
            "Include: STATUS: [cosmic emoji] [mystical action 8-12 words]...\n\
            Context: {phase_description} for {operation_type}. {context_hint}\n\
            Style: Abstract, cosmic, avoid literal descriptions.\n\
            Examples: 🔮 Consulting the cosmic oracle... 🌌 Aligning celestial spheres..."
        )
    }

    /// Extract status message from LLM response that includes status in structured format
    fn extract_status_from_response(response: &str, phase: &IrisPhase) -> String {
        // Use precompiled regex patterns for better performance
        for re in STATUS_PATTERNS.iter() {
            if let Some(captures) = re.captures(response) {
                if let Some(status_match) = captures.get(1) {
                    let status = status_match.as_str().trim().to_string();
                    if !status.is_empty() && status.len() > 5 {
                        return status;
                    }
                }
            }
        }

        // Fallback to mystical cosmic message if no status found in response
        match phase {
            IrisPhase::Planning => "🔮 Consulting the cosmic commit oracle...".to_string(),
            IrisPhase::ToolExecution { tool_name, .. } => {
                format!("💎 Charging the {tool_name} crystals with cosmic energy...")
            }
            IrisPhase::Analysis => {
                "🔬 Analyzing code particles at the quantum level...".to_string()
            }
            IrisPhase::Synthesis => "🧙 Casting a spell for the perfect synthesis...".to_string(),
            IrisPhase::Generation => {
                "⭐ Gathering stardust for your stellar creation...".to_string()
            }
            IrisPhase::PlanExpansion => {
                "🧭 Calibrating the cosmic compass for true direction...".to_string()
            }
            _ => "🌊 Diving into the depths of the code ocean...".to_string(),
        }
    }

    /// Update status with a dynamic message extracted from the LLM response
    fn update_status_from_response(&self, response: &str, request: &GenerationRequest) {
        let message = Self::extract_status_from_response(response, &request.phase);
        crate::log_debug!(
            "🌟 LLM Service: Updating UI with dynamic status: '{}'",
            message
        );
        IRIS_STATUS.update(IrisStatus::dynamic(
            request.phase.clone(),
            message,
            request.current_step,
            request.total_steps,
        ));

        // Give time for the dynamic status to be visible
        std::thread::sleep(std::time::Duration::from_millis(1000));
        crate::log_debug!("🌟 LLM Service: Dynamic status display time completed");
    }

    /// Check streaming text for status updates and update UI in real-time
    fn check_and_update_streaming_status(chunk: &str, request: &GenerationRequest) {
        // Look for complete STATUS lines in the chunk
        if chunk.contains("STATUS:") {
            // Extract the status message using our regex patterns
            for re in STATUS_PATTERNS.iter() {
                if let Some(captures) = re.captures(chunk) {
                    if let Some(status_match) = captures.get(1) {
                        let status = status_match.as_str().trim().to_string();
                        if !status.is_empty() && status.len() > 5 {
                            crate::log_debug!(
                                "🌊 LLM Service: Real-time streaming status update: '{}'",
                                status
                            );
                            IRIS_STATUS.update(IrisStatus::dynamic(
                                request.phase.clone(),
                                status,
                                request.current_step,
                                request.total_steps,
                            ));
                            return;
                        }
                    }
                }
            }
        }
    }
}
