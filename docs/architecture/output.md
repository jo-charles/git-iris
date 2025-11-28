# Structured Output & Validation

Git-Iris enforces structured output from LLMs using JSON schemas and robust validation. This ensures predictable, parseable responses even when LLMs produce malformed or verbose output.

**Source:** `src/agents/output_validator.rs`, `src/agents/iris.rs`

## Design Philosophy

### Type-Safe Responses

All Iris responses map to Rust types:

```rust
pub enum StructuredResponse {
    CommitMessage(GeneratedMessage),       // Strict JSON schema
    PullRequest(MarkdownPullRequest),      // Markdown wrapper
    Changelog(MarkdownChangelog),          // Markdown wrapper
    ReleaseNotes(MarkdownReleaseNotes),    // Markdown wrapper
    MarkdownReview(MarkdownReview),        // Markdown wrapper
    SemanticBlame(String),                 // Plain text
    PlainText(String),                     // Fallback
}
```

**Two patterns:**

1. **Strict JSON** — `GeneratedMessage` with specific fields
2. **Markdown wrappers** — Single `content: String` field for LLM flexibility

### Schema-Driven Validation

JSON schemas are generated from Rust types using `schemars`:

```rust
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GeneratedMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    pub title: String,
    pub message: String,
}
```

The schema is injected into the prompt:

```json
{
  "type": "object",
  "required": ["title", "message"],
  "properties": {
    "emoji": { "type": ["string", "null"] },
    "title": { "type": "string" },
    "message": { "type": "string" }
  }
}
```

Iris must return JSON matching this schema.

## Validation Pipeline

When Iris completes a task, the response goes through a multi-stage pipeline:

````
┌──────────────────────────────────────────────────────────────┐
│ 1. Raw LLM Response                                          │
│    "Here's the commit message:\n```json\n{...}\n```"         │
└──────────────────────────────────────────────────────────────┘
                          ▼
┌──────────────────────────────────────────────────────────────┐
│ 2. JSON Extraction                                           │
│    extract_json_from_response()                              │
│    • Try parsing entire response                             │
│    • Check for markdown code blocks                          │
│    • Find JSON object by brace matching                      │
└──────────────────────────────────────────────────────────────┘
                          ▼
┌──────────────────────────────────────────────────────────────┐
│ 3. Sanitization                                              │
│    sanitize_json_response()                                  │
│    • Escape literal newlines in strings                      │
│    • Escape control characters                               │
│    • Fix malformed escape sequences                          │
└──────────────────────────────────────────────────────────────┘
                          ▼
┌──────────────────────────────────────────────────────────────┐
│ 4. Schema Validation & Recovery                              │
│    validate_and_parse()                                      │
│    • Try direct parsing                                      │
│    • On failure: recover_missing_fields()                    │
│    • On failure: recover_type_mismatches()                   │
│    • On failure: extract_required_fields()                   │
└──────────────────────────────────────────────────────────────┘
                          ▼
┌──────────────────────────────────────────────────────────────┐
│ 5. Type-Safe Result                                          │
│    GeneratedMessage { emoji, title, message }                │
└──────────────────────────────────────────────────────────────┘
````

## JSON Extraction

### Problem

LLMs often return JSON with extra text:

````
I've analyzed the changes. Here's the commit message:

```json
{
  "emoji": "✨",
  "title": "Add parallel analysis",
  "message": "Implements concurrent subagent processing..."
}
````

This enables large changesets to be analyzed efficiently.

````

### Solution

`extract_json_from_response()` handles multiple formats:

```rust
fn extract_json_from_response(response: &str) -> Result<String> {
    // 1. Try parsing entire response as JSON
    if response.trim().starts_with('{') {
        if let Ok(_) = serde_json::from_str::<Value>(response) {
            return Ok(response.to_string());
        }
    }

    // 2. Look for markdown code blocks
    if let Some(start) = response.find("```json") {
        let content_start = start + "```json".len();
        let json_end = response[content_start..].find("\n```")
            .unwrap_or(response.len() - content_start);
        let json = &response[content_start..content_start + json_end];
        return Ok(json.trim().to_string());
    }

    // 3. Find JSON by brace matching
    let mut brace_count = 0;
    let mut json_start = None;

    for (i, ch) in response.char_indices() {
        match ch {
            '{' => {
                if brace_count == 0 { json_start = Some(i); }
                brace_count += 1;
            }
            '}' => {
                brace_count -= 1;
                if brace_count == 0 && json_start.is_some() {
                    let json = &response[json_start.unwrap()..i + 1];
                    // Validate it's actually JSON
                    serde_json::from_str::<Value>(json)?;
                    return Ok(json.to_string());
                }
            }
            _ => {}
        }
    }

    Err(anyhow::anyhow!("No valid JSON found in response"))
}
````

## Sanitization

### Problem

Some providers (Anthropic) occasionally send **literal control characters** in JSON strings:

```json
{
  "title": "Fix authentication bug",
  "message": "The OAuth flow was broken
because of invalid state handling"
}
```

This violates strict JSON parsing rules.

### Solution

`sanitize_json_response()` escapes control characters **only inside string literals**:

```rust
fn sanitize_json_response(raw: &str) -> Cow<'_, str> {
    // Fast path: check if sanitization is needed
    let needs_sanitization = /* scan for control chars in strings */;

    if !needs_sanitization {
        return Cow::Borrowed(raw);  // No allocation
    }

    // Slow path: escape control characters
    let mut sanitized = String::with_capacity(raw.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in raw.chars() {
        if in_string {
            match ch {
                '\n' => sanitized.push_str("\\n"),
                '\r' => sanitized.push_str("\\r"),
                '\t' => sanitized.push_str("\\t"),
                c if c.is_control() => {
                    write!(&mut sanitized, "\\u{:04X}", u32::from(c));
                }
                _ => sanitized.push(ch),
            }
        } else {
            sanitized.push(ch);
            if ch == '"' { in_string = true; }
        }
    }

    Cow::Owned(sanitized)
}
```

**Result:**

```json
{
  "title": "Fix authentication bug",
  "message": "The OAuth flow was broken\nbecause of invalid state handling"
}
```

## Schema Validation & Recovery

### ValidationResult

Validation returns metadata about recovery attempts:

```rust
pub struct ValidationResult<T> {
    pub value: Option<T>,          // Parsed value if successful
    pub warnings: Vec<String>,     // Non-fatal issues
    pub recovered: bool,           // Whether recovery was needed
}
```

### Recovery Strategies

#### 1. Missing Required Fields

If a required field is missing, add a default:

```rust
fn recover_missing_fields(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>
) {
    let required = schema["required"].as_array()?;

    for field in required {
        let field_name = field.as_str()?;
        if !obj.contains_key(field_name) {
            // Add default based on type
            let field_schema = schema["properties"][field_name];
            let default_value = match field_schema["type"].as_str()? {
                "string" => Value::String("".to_string()),
                "number" => Value::Number(0.into()),
                "boolean" => Value::Bool(false),
                "array" => Value::Array(vec![]),
                "object" => Value::Object(Map::new()),
                _ => Value::Null,
            };

            obj.insert(field_name.to_string(), default_value);
            warnings.push(format!("Added missing field: {}", field_name));
        }
    }
}
```

#### 2. Type Mismatches

If a field has the wrong type, attempt coercion:

```rust
fn recover_type_mismatches(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>
) {
    for (field_name, field_value) in obj.iter_mut() {
        let expected_type = schema["properties"][field_name]["type"].as_str()?;

        match (expected_type, field_value) {
            // String expected, got number
            ("string", Value::Number(n)) => {
                *field_value = Value::String(n.to_string());
                warnings.push(format!("Coerced {} to string", field_name));
            }
            // Array expected, got string
            ("array", Value::String(s)) => {
                *field_value = Value::Array(vec![Value::String(s.clone())]);
                warnings.push(format!("Wrapped {} in array", field_name));
            }
            // Null not allowed, got null
            ("string", Value::Null) => {
                *field_value = Value::String("".to_string());
                warnings.push(format!("Replaced null {} with empty string", field_name));
            }
            _ => {}
        }
    }
}
```

#### 3. Null to Defaults

Replace nulls with appropriate defaults:

```rust
fn recover_null_to_defaults(
    obj: &mut Map<String, Value>,
    schema: &Value,
    warnings: &mut Vec<String>
) {
    for (field_name, field_value) in obj.iter_mut() {
        if field_value.is_null() {
            let field_schema = schema["properties"][field_name];
            let field_type = field_schema["type"].as_str()?;

            if !field_type.contains("null") {
                *field_value = default_for_type(field_type);
                warnings.push(format!("Replaced null {} with default", field_name));
            }
        }
    }
}
```

#### 4. Extract Required Only

As a last resort, extract just the required fields:

```rust
fn extract_required_fields(json: &Value, schema: &Value) -> Value {
    let required = schema["required"].as_array()?;
    let mut result = Map::new();

    for field in required {
        let field_name = field.as_str()?;
        if let Some(value) = json[field_name].clone() {
            result.insert(field_name.to_string(), value);
        }
    }

    Value::Object(result)
}
```

### Full Validation Flow

```rust
pub fn validate_and_parse<T>(json_str: &str) -> Result<ValidationResult<T>>
where
    T: JsonSchema + DeserializeOwned,
{
    let mut warnings = Vec::new();

    // Try direct parsing
    match serde_json::from_str::<T>(json_str) {
        Ok(value) => return Ok(ValidationResult::success(value)),
        Err(e) => warnings.push(format!("Initial parse failed: {}", e)),
    }

    // Parse as generic Value for recovery
    let mut json_value: Value = serde_json::from_str(json_str)?;
    let schema = schema_for!(T);

    // Apply recovery strategies
    if let Some(obj) = json_value.as_object_mut() {
        recover_missing_fields(obj, &schema_value, &mut warnings);
        recover_type_mismatches(obj, &schema_value, &mut warnings);
        recover_null_to_defaults(obj, &schema_value, &mut warnings);
    }

    // Try parsing again
    match serde_json::from_value::<T>(json_value.clone()) {
        Ok(value) => Ok(ValidationResult::recovered(value, warnings)),
        Err(e) => {
            // Final attempt: extract required fields only
            let minimal = extract_required_fields(&json_value, &schema_value);
            match serde_json::from_value::<T>(minimal) {
                Ok(value) => {
                    warnings.push(format!("Extracted required fields only: {}", e));
                    Ok(ValidationResult::recovered(value, warnings))
                }
                Err(final_e) => Err(anyhow::anyhow!(
                    "Failed to parse JSON even after recovery: {}", final_e
                ))
            }
        }
    }
}
```

## Type Examples

### Strict JSON: GeneratedMessage

**Schema:**

```json
{
  "type": "object",
  "required": ["title", "message"],
  "properties": {
    "emoji": { "type": ["string", "null"] },
    "title": { "type": "string", "maxLength": 72 },
    "message": { "type": "string" }
  }
}
```

**Rust type:**

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GeneratedMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    pub title: String,
    pub message: String,
}
```

**Example output:**

```json
{
  "emoji": "✨",
  "title": "Add parallel analysis for large changesets",
  "message": "Implements concurrent subagent processing for analyzing large changesets. Each subagent runs independently with its own context window, preventing token limit errors."
}
```

### Markdown Wrapper: MarkdownPullRequest

**Schema:**

```json
{
  "type": "object",
  "required": ["content"],
  "properties": {
    "content": { "type": "string" }
  }
}
```

**Rust type:**

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MarkdownPullRequest {
    pub content: String,
}

impl MarkdownPullRequest {
    pub fn raw_content(&self) -> &str {
        &self.content
    }
}
```

**Example output:**

```json
{
  "content": "## Summary\n\nAdds parallel analysis capability using concurrent subagents...\n\n## Changes\n\n- New `ParallelAnalyze` tool...\n- Subagent runner with provider abstraction...\n\n## Test Plan\n\n- [ ] Test with large changesets (>20 files)\n- [ ] Verify concurrent execution\n"
}
```

The LLM structures the markdown as it sees fit — Git-Iris just wraps it.

## Streaming vs. Structured

Git-Iris supports two execution modes:

### 1. Structured Mode (Default)

**Use case:** CLI, background tasks

**Flow:**

- Iris calls tools and gathers context
- Returns complete JSON response
- Goes through full validation pipeline
- Guaranteed to match schema

**Limitations:**

- No real-time feedback
- Slower perceived latency

### 2. Streaming Mode (Studio TUI)

**Use case:** Real-time TUI updates

**Flow:**

- Iris streams text as it's generated
- TUI displays chunks immediately
- No JSON validation (markdown only)
- Aggregated text converted to structured response at end

**Limitations:**

- Only works for markdown types
- No schema validation until stream completes

**Implementation:**

```rust
pub async fn execute_task_streaming<F>(
    &mut self,
    capability: &str,
    user_prompt: &str,
    mut on_chunk: F,
) -> Result<StructuredResponse>
where
    F: FnMut(&str, &str) + Send,
{
    let agent = self.build_agent()?;
    let mut stream = agent.stream_prompt(&full_prompt).multi_turn(50).await;

    let mut aggregated_text = String::new();

    while let Some(item) = stream.next().await {
        if let StreamedAssistantContent::Text(text) = item {
            aggregated_text.push_str(&text.text);
            on_chunk(&text.text, &aggregated_text);  // TUI update
        }
    }

    // Convert to structured response
    Ok(StructuredResponse::MarkdownReview(MarkdownReview {
        content: aggregated_text,
    }))
}
```

## Error Reporting

Validation errors provide rich context:

```rust
// Extraction failure
debug::debug_json_parse_error("No valid JSON found in response");

// Sanitization applied
debug::debug_context_management(
    "Sanitized JSON response",
    &format!("{} → {} characters", original_len, sanitized_len)
);

// Recovery applied
debug::debug_context_management(
    "JSON recovery applied",
    &format!("{} issues fixed", warnings.len())
);
for warning in &warnings {
    debug::debug_warning(warning);
}

// Final failure
Err(anyhow::anyhow!("Failed to parse JSON even after recovery: {}", e))
```

Enable with `--debug` for detailed diagnostics.

## Best Practices

### For Capability Authors

✅ **DO:**

- Use strict JSON for structured data (commits, metadata)
- Use markdown wrappers for flexible content (reviews, PRs)
- Mark optional fields with `Option<T>`
- Provide clear schema in prompts

❌ **DON'T:**

- Over-constrain markdown structure
- Require nested objects unless necessary
- Use complex enums in JSON (LLMs struggle with variants)

### For Type Designers

✅ **DO:**

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyType {
    pub required_field: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<String>,

    #[serde(default)]
    pub with_default: Vec<String>,
}
```

❌ **DON'T:**

```rust
pub struct MyType {
    // Too complex for LLM to generate reliably
    pub status: Result<Status, ErrorDetails>,

    // Deeply nested
    pub metadata: HashMap<String, HashMap<String, Vec<Attribute>>>,
}
```

### For Validation

✅ **DO:**

- Log warnings from recovery
- Provide defaults for missing fields
- Attempt type coercion
- Extract required fields as last resort

❌ **DON'T:**

- Fail on first error
- Assume LLM output is perfect
- Discard valuable but malformed data

## Testing Validation

### Test Extraction

````rust
#[test]
fn extracts_json_from_markdown() {
    let response = r#"
Here's the result:

```json
{"title": "Test", "message": "Body"}
````

    "#;

    let json = extract_json_from_response(response).unwrap();
    assert_eq!(json, r#"{"title": "Test", "message": "Body"}"#);

}

````

### Test Sanitization

```rust
#[test]
fn sanitizes_literal_newlines() {
    let raw = r#"{"message": "Line 1
Line 2"}"#;

    let sanitized = sanitize_json_response(raw);
    assert_eq!(sanitized.as_ref(), r#"{"message": "Line 1\nLine 2"}"#);

    let parsed: serde_json::Value = serde_json::from_str(sanitized.as_ref()).unwrap();
    assert_eq!(parsed["message"], "Line 1\nLine 2");
}
````

### Test Recovery

```rust
#[test]
fn recovers_missing_required_field() {
    let json = r#"{"title": "Test"}"#;  // Missing "message"

    let result = validate_and_parse::<GeneratedMessage>(json).unwrap();

    assert!(result.recovered);
    assert_eq!(result.value.unwrap().message, "");
    assert!(result.warnings.iter().any(|w| w.contains("missing field")));
}
```

## Next Steps

- [Agent System](./agent.md) — How structured output fits into execution
- [Capabilities](./capabilities.md) — Defining output types in TOML
- [Context Strategy](./context.md) — How Iris gathers information before generating output
