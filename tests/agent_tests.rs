use git_iris::{
    agents::{
        core::{AgentContext, TaskResult},
        executor::{TaskPriority, TaskRequest},
        tools::{AgentTool, GitTool, create_default_tool_registry},
    },
    config::{Config, PerformanceConfig},
    git::GitRepo,
};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_context() -> (AgentContext, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize a git repo in the temp directory
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to initialize git repo");

    let config = Config::default();
    let git_repo = GitRepo::new(&repo_path).expect("Failed to create GitRepo");

    let context = AgentContext::new(config, git_repo);
    (context, temp_dir)
}

#[tokio::test]
async fn test_tool_registry_creation() {
    let registry = create_default_tool_registry();

    assert!(!registry.list_tools().is_empty());
    assert!(registry.list_capabilities().contains(&"git".to_string()));
    assert!(
        registry
            .list_capabilities()
            .contains(&"file_analysis".to_string())
    );
}

#[tokio::test]
async fn test_git_tool_execution() {
    let (context, _temp_dir) = create_test_context();
    let git_tool = GitTool::new();

    let mut params = HashMap::new();
    params.insert(
        "operation".to_string(),
        serde_json::Value::String("status".to_string()),
    );

    let result = git_tool.execute(&context, &params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_task_result_creation() {
    let result = TaskResult::success("Test completed".to_string());
    assert!(result.success);
    assert_eq!(result.message, "Test completed");
    assert!((result.confidence - 1.0).abs() < f32::EPSILON);

    let failure = TaskResult::failure("Test failed".to_string());
    assert!(!failure.success);
    assert_eq!(failure.message, "Test failed");
    assert!((failure.confidence - 0.0).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_task_request_builder() {
    let mut params = HashMap::new();
    params.insert(
        "test".to_string(),
        serde_json::Value::String("value".to_string()),
    );

    let request = TaskRequest::new("test_task".to_string())
        .with_params(params)
        .with_priority(TaskPriority::High)
        .with_timeout(std::time::Duration::from_secs(60))
        .with_retries(3);

    assert_eq!(request.task_type, "test_task");
    assert_eq!(request.priority, TaskPriority::High);
    assert_eq!(request.max_retries, 3);
    assert!(request.timeout.is_some());
}

#[tokio::test]
async fn test_agent_context_session_data() {
    let (context, _temp_dir) = create_test_context();

    let key = "test_key".to_string();
    let value = serde_json::json!({"data": "test_value"});

    context.set_session_data(key.clone(), value.clone()).await;
    let retrieved = context.get_session_data(&key).await;

    assert_eq!(retrieved, Some(value));
}

#[test]
fn test_task_priority_ordering() {
    assert!(TaskPriority::Critical > TaskPriority::High);
    assert!(TaskPriority::High > TaskPriority::Normal);
    assert!(TaskPriority::Normal > TaskPriority::Low);
}

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    assert_eq!(config.max_concurrent_tasks, Some(5));
    assert_eq!(config.default_timeout_seconds, Some(300));
    assert!(config.use_agent_framework);
}
