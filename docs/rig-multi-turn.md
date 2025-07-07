
## The Definitive Guide to Building Multi-Turn Agents in Rig

This document provides a detailed walkthrough for building a sophisticated, multi-turn agent using the Rig framework in Rust. We will cover how to create custom tools, handle streaming responses, manage multi-step reasoning (tool use), and abstract away specific Large Language Model (LLM) providers to create flexible, powerful agents.

-----

### Part 1: The Foundation - Abstracting LLM Providers

A core strength of Rig is its ability to abstract different LLM providers behind a consistent interface. This lets you build your application logic once and switch between providers like OpenAI, Anthropic, or Gemini with minimal code changes.

The key to this abstraction is the `ProviderClient` trait and the `DynClientBuilder`.

**Workflow:**

1.  **Instantiate `DynClientBuilder`:** This builder acts as a factory for any supported provider client.
2.  **Request a Client:** Use methods like `.agent()` on the builder, specifying the provider's name (e.g., `"openai"`, `"anthropic"`) and the desired model as strings.
3.  **Build the Agent:** The builder returns a dynamically-typed `AgentBuilder`, which you can then configure with a preamble, tools, and other settings.

**Code Example: Creating Agents for Different Providers**

The following example demonstrates how to use `DynClientBuilder` to create agents for both OpenAI and Anthropic. This approach allows you to select a provider dynamically, for instance, based on configuration or user input.

```rust
use rig::{
    client::builder::DynClientBuilder,
    completion::Prompt,
    providers::anthropic::CLAUDE_3_7_SONNET,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let multi_client = DynClientBuilder::new();

    // 1. Set up an OpenAI agent
    // This requires the OPENAI_API_KEY environment variable.
    let openai_agent_builder = multi_client.agent("openai", "gpt-4o").unwrap();
    let agent_openai = openai_agent_builder
        .preamble("You are a helpful assistant powered by OpenAI.")
        .build();

    // 2. Set up an Anthropic agent
    // This requires the ANTHROPIC_API_KEY environment variable.
    [cite_start]let anthropic_agent_builder = multi_client.agent("anthropic", CLAUDE_3_7_SONNET).unwrap(); [cite: 685]
    let agent_anthropic = anthropic_agent_builder
        .preamble("You are a helpful assistant powered by Anthropic.")
        .max_tokens(1024) // Some providers require certain parameters.
        .build();

    println!("Sending prompt: 'Hello world!'");

    let res_openai = agent_openai.prompt("Hello world!").await.unwrap();
    println!("Response from OpenAI (gpt-4o): {res_openai}");

    let res_anthropic = agent_anthropic.prompt("Hello world!").await.unwrap();
    println!("Response from Anthropic (Claude 3.7 Sonnet): {res_anthropic}");

    Ok(())
}
```

-----

### Part 2: Creating Custom Tools with `#[rig_tool]`

Tools are functions that an agent can call to interact with external systems or perform specific computations. Rig simplifies tool creation with the `#[rig_tool]` procedural macro, which automatically generates the necessary `Tool` trait implementation from a standard Rust function.

**Workflow:**

1.  **Define a Function:** Write a regular `sync` or `async` Rust function that takes arguments and returns a `Result<T, rig::tool::ToolError>`. The success type `T` must be serializable.
2.  **Add the `#[rig_tool]` Attribute:**
      * `description`: Provide a high-level description of what the tool does. This is **critical** for the LLM to understand when to use the tool.
      * `params(...)`: Describe each parameter. This helps the LLM provide the correct arguments.
      * `required(...)`: List the parameters that are mandatory for the tool call. OpenAI's strict function calling mode requires this.

**Code Example: A Simple Calculator Tool**

Here, we define a function `calculator` and use `#[rig_tool]` to turn it into a `Tool` that our agent can use.

```rust
use rig::tool::Tool;
use rig_derive::rig_tool;

// The rig_tool macro generates a struct named `Calculator` that implements the Tool trait.
#[rig_tool(
    description = "Perform basic arithmetic operations.",
    params(
        x = "The first number in the calculation.",
        y = "The second number in thecalculation.",
        operation = "The operation to perform (e.g., add, subtract)."
    ),
    required(x, y, operation)
)]
fn calculator(x: f32, y: f32, operation: String) -> Result<f32, rig::tool::ToolError> {
    match operation.as_str() {
        "add" => Ok(x + y),
        [cite_start]"subtract" => Ok(x - y), [cite: 178]
        "multiply" => Ok(x * y),
        "divide" => {
            if y == 0.0 {
                Err(rig::tool::ToolError::ToolCallError(
                    "Division by zero is not allowed.".into(),
                ))
            } else {
                Ok(x / y)
            }
        }
        _ => Err(rig::tool::ToolError::ToolCallError(
            format!("Unknown operation: {}", operation).into(),
        )),
    }
}

// You can now attach this to an agent like this:
// let agent = client
//     .agent("gpt-4o")
//     .tool(Calculator) // The generated struct name is PascalCase of the function name
//     .build();
```

-----

### Part 3: Building a Multi-Turn Agent

A multi-turn agent can engage in a sequence of reasoning steps, often involving multiple tool calls, before arriving at a final answer. For example, to solve `(5 + 3) * 2`, the agent must first call the `add` tool and then use its result to call the `multiply` tool.

**Workflow:**

1.  **Build the Agent:** Use `AgentBuilder` to define the agent's preamble and attach all necessary tools using the `.tool()` method.
2.  **Enable Multi-Turn:** When prompting the agent, chain the `.multi_turn(depth)` method. The `depth` parameter specifies the maximum number of sequential tool-call-and-response loops the agent can perform. This prevents infinite loops.
3.  **Await the Final Result:** The `.await` on the prompt request will handle the entire multi-turn conversation transparently and return only the final text response from the LLM.

**Code Example: A Multi-Turn Calculator**

This example shows an agent that can solve a multi-step arithmetic problem. It uses the `.multi_turn()` method to allow the agent to first add `3 + 5` and then use that result to perform the final division.

```rust
use rig::prelude::*;
use rig::{
    completion::{Prompt, ToolDefinition},
    providers::anthropic,
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Tools (Add, Subtract, Multiply, Divide) are defined here...
// For brevity, see the full tool definitions in the "Custom Tools" section above.
# #[derive(Deserialize)]
# struct OperationArgs { x: i32, y: i32 }
# #[derive(Debug, thiserror::Error)]
# #[error("Math error")]
# struct MathError;
# #[derive(Deserialize, Serialize)]
# struct Add;
# impl Tool for Add {
#     const NAME: &'static str = "add";
#     type Error = MathError;
#     type Args = OperationArgs;
#     type Output = i32;
#     async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name": "add", "description": "Add two numbers", "parameters": {}})).unwrap() }
#     async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> { Ok(args.x + args.y) }
# }
# #[derive(Deserialize, Serialize)]
# struct Subtract;
# impl Tool for Subtract {
#     const NAME: &'static str = "subtract";
#     type Error = MathError;
#     type Args = OperationArgs;
#     type Output = i32;
#     async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name": "subtract", "description": "Subtract two numbers", "parameters": {}})).unwrap() }
#     async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> { Ok(args.x - args.y) }
# }
# struct Multiply;
# impl Tool for Multiply {
#     const NAME: &'static str = "multiply";
#     type Error = MathError;
#     type Args = OperationArgs;
#     type Output = i32;
#     async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name": "multiply", "description": "Multiply two numbers", "parameters": {}})).unwrap() }
#     async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> { Ok(args.x * args.y) }
# }
# struct Divide;
# impl Tool for Divide {
#     const NAME: &'static str = "divide";
#     type Error = MathError;
#     type Args = OperationArgs;
#     type Output = i32;
#     async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name": "divide", "description": "Divide two numbers", "parameters": {}})).unwrap() }
#     async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> { Ok(args.x / args.y) }
# }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = anthropic::Client::from_env();

    let agent = client
        .agent(anthropic::CLAUDE_3_5_SONNET)
        .preamble(
            "You are an assistant that helps with arithmetic. \
             Select the appropriate tool for the request. \
             [cite_start]This is very important: never perform the operation yourself. [cite: 1295]"
        )
        .tool(Add)
        .tool(Subtract)
        .tool(Multiply)
        .tool(Divide)
        .build();

    // The agent will first call add(3, 5), get 8, then call divide(8, 9)
    let result = agent
        .prompt("Calculate (3 + 5) / 9. Describe the result to me.")
        .multi_turn(10) // Allow up to 10 reasoning steps
        [cite_start].await?; [cite: 1296]

    println!("\nFinal Agent Response: {result}");

    Ok(())
}
```

-----

### Part 4: Implementing Streaming

Streaming allows you to process the LLM's response as it's being generated, which is essential for creating responsive, real-time user interfaces.

When streaming, the agent's response is broken into chunks. These chunks can be either text deltas or complete tool calls. Your application code is responsible for handling this stream.

**Workflow:**

1.  **Initiate the Stream:** Instead of `.prompt(..).await`, call `.stream_prompt(...)` or `.stream_chat(...)` on your agent. This returns a `StreamingCompletionResponse`.
2.  **Process the Stream:** The `StreamingCompletionResponse` is a `Stream`. Loop over it using `while let Some(chunk) = stream.next().await`.
3.  **Handle Chunks:** Each `chunk` is a `Result<AssistantContent, ...>`.
      * If it's `Ok(AssistantContent::Text(text))`, append the text to your display.
      * If it's `Ok(AssistantContent::ToolCall(tool_call))`, the model wants to use a tool. You must now execute this tool.
4.  **Execute Tools and Continue the Conversation:**
      * Once the first part of the stream finishes, collect all the `ToolCall`s.
      * Execute them using your `ToolSet`.
      * Create a new `UserContent::tool_result` message containing the output of the tool call.
      * Start a *new* streaming request to the agent, providing the original chat history plus the assistant's tool call message and the new user tool result message.
      * Process this new stream to get the final text response.

**Code Example: A Streaming, Multi-Turn Agent**

This example shows the client-side logic required to handle a streaming response that includes tool calls. The `multi_turn_prompt` function encapsulates the logic for handling the conversation loop.

```rust
use rig::prelude::*;
use rig::{
    agent::Agent,
    completion::{CompletionModel, Message, ToolDefinition},
    message::{AssistantContent, UserContent, Text, ToolResultContent},
    streaming::{StreamingCompletion, stream_to_stdout},
    tool::Tool, OneOrMany
};
use futures::{Stream, stream, StreamExt};
use serde_json::json;
use std::pin::Pin;

// Define errors and tools (Adder, etc.) as in previous examples...
# #[derive(serde::Deserialize)] struct OperationArgs { x: i32, y: i32 }
# #[derive(Debug, thiserror::Error)] #[error("...")] struct MathError;
# #[derive(serde::Deserialize, serde::Serialize)] struct Adder;
# impl Tool for Adder { const NAME: &'static str = "add"; type Error = MathError; type Args = OperationArgs; type Output = i32; async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name":"add"})).unwrap() } async fn call(&self, args: Self::Args) -> Result<i32, MathError> { Ok(args.x + args.y) } }
# #[derive(serde::Deserialize, serde::Serialize)] struct Subtract;
# impl Tool for Subtract { const NAME: &'static str = "subtract"; type Error = MathError; type Args = OperationArgs; type Output = i32; async fn definition(&self, _: String) -> ToolDefinition { serde_json::from_value(json!({"name":"subtract"})).unwrap() } async fn call(&self, args: Self::Args) -> Result<i32, MathError> { Ok(args.x - args.y) } }

type StreamingError = Box<dyn std::error::Error + Send + Sync>;
type StreamingResult = Pin<Box<dyn Stream<Item = Result<Text, StreamingError>> + Send>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = rig::providers::anthropic::Client::from_env();
    let agent = client
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .preamble("You are a calculator. You must use tools to answer. Never do math yourself.")
        .tool(Adder)
        .tool(Subtract)
        .build();

    let mut stream = multi_turn_streaming_prompt(
        agent,
        "Calculate 2 - 5 and tell me about the result.",
        Vec::new(),
    ).await;

    // Custom helper to print stream to stdout
    print!("Final Response: ");
    while let Some(content) = stream.next().await {
        match content {
            Ok(Text { text }) => {
                print!("{text}");
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            Err(err) => eprintln!("\nStream Error: {err}"),
        }
    }
    println!();

    Ok(())
}

/// This function encapsulates the multi-turn logic for streaming with tool calls.
async fn multi_turn_streaming_prompt<M>(
    agent: Agent<M>,
    prompt: impl Into<Message> + Send,
    mut chat_history: Vec<Message>,
) -> StreamingResult
where
    M: CompletionModel + 'static,
    M::StreamingResponse: std::marker::Send,
{
    let prompt_message: Message = prompt.into();

    Box::pin(async_stream::stream! {
        // Start with the initial user prompt
        let mut current_prompt = prompt_message;
        
        // Loop for multi-turn conversation
        loop {
            // Add the current prompt to history before sending
            chat_history.push(current_prompt.clone());

            let mut stream = agent
                .stream_chat(current_prompt, chat_history.clone())
                .await
                .unwrap();

            let mut tool_calls = vec![];
            
            // Process the stream from the LLM
            while let Some(content_result) = stream.next().await {
                match content_result {
                    Ok(AssistantContent::Text(text)) => {
                        // Yield text chunks to the consumer of this stream
                        yield Ok(text);
                    },
                    Ok(AssistantContent::ToolCall(tool_call)) => {
                        // Collect tool calls to be executed after the stream part is done
                        tool_calls.push(tool_call);
                    },
                    Err(e) => {
                        yield Err(Box::new(e) as StreamingError);
                        return;
                    }
                }
            }

            // If there are no more tool calls, the conversation is over.
            if tool_calls.is_empty() {
                break;
            }

            // Add the assistant's decision to call tools to the chat history
            let tool_call_message = Message::Assistant {
                id: None,
                content: OneOrMany::many(tool_calls.clone().into_iter().map(AssistantContent::ToolCall).collect()).unwrap(),
            };
            chat_history.push(tool_call_message);

            // Execute the collected tool calls
            let mut tool_results = vec![];
            for tool_call in tool_calls {
                let result = agent
                    .tools
                    .call(&tool_call.function.name, tool_call.function.arguments.to_string())
                    .await
                    .unwrap(); // Simplified error handling
                
                let tool_result_content = UserContent::tool_result(tool_call.id, OneOrMany::one(result.into()));
                tool_results.push(tool_result_content);
            }
            
            // Create a new prompt containing the results of the tool calls
            current_prompt = Message::User {
                content: OneOrMany::many(tool_results).unwrap(),
            };

            // The loop continues, sending the tool results back to the agent
        }
    })
}

```