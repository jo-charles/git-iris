use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time::timeout;

use crate::agents::{
    core::{AgentContext, TaskResult},
    registry::AgentRegistry,
};

/// Task execution engine for managing agent workloads
pub struct AgentExecutor {
    registry: Arc<AgentRegistry>,
    task_queue: Arc<RwLock<Vec<PendingTask>>>,
    max_concurrent_tasks: usize,
    task_timeout: Duration,
    running_tasks: Arc<RwLock<HashMap<String, RunningTask>>>,
    // Statistics tracking
    stats: Arc<RwLock<ExecutionStats>>,
}

/// Internal statistics tracking
#[derive(Debug, Clone, Default)]
struct ExecutionStats {
    total_tasks_executed: u64,
    successful_tasks: u64,
    failed_tasks: u64,
    execution_times: Vec<Duration>,
    tasks_by_type: HashMap<String, u64>,
    tasks_by_agent: HashMap<String, u64>,
}

/// A task waiting to be executed
#[derive(Debug, Clone)]
pub struct PendingTask {
    pub id: String,
    pub task_type: String,
    pub context: AgentContext,
    pub params: HashMap<String, serde_json::Value>,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub max_retries: u32,
}

/// A currently executing task
#[derive(Debug)]
pub struct RunningTask {
    pub id: String,
    pub task_type: String,
    pub agent_id: String,
    pub started_at: Instant,
    pub cancel_sender: mpsc::Sender<()>,
}

/// Task execution request
#[derive(Debug, Clone)]
pub struct TaskRequest {
    pub task_type: String,
    pub params: HashMap<String, serde_json::Value>,
    pub priority: TaskPriority,
    pub timeout: Option<Duration>,
    pub max_retries: u32,
}

/// Task execution result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub task_id: String,
    pub result: TaskResult,
    pub agent_id: String,
    pub execution_time: Duration,
    pub retry_count: u32,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TaskPriority {
    Low = 1,
    #[default]
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    pub total_tasks_executed: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time: Duration,
    pub tasks_by_type: HashMap<String, u64>,
    pub tasks_by_agent: HashMap<String, u64>,
    pub current_queue_size: usize,
    pub current_running_tasks: usize,
}

impl AgentExecutor {
    /// Create a new executor with the given registry
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            registry,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(300), // 5 minutes default
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
        }
    }

    /// Configure executor settings
    #[must_use]
    pub fn with_max_concurrent_tasks(mut self, max: usize) -> Self {
        self.max_concurrent_tasks = max;
        self
    }

    #[must_use]
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.task_timeout = timeout;
        self
    }

    /// Submit a task for execution
    pub async fn submit_task(&self, request: TaskRequest, context: AgentContext) -> Result<String> {
        let task_id = uuid::Uuid::new_v4().to_string();

        let pending_task = PendingTask {
            id: task_id.clone(),
            task_type: request.task_type,
            context,
            params: request.params,
            priority: request.priority,
            created_at: Instant::now(),
            timeout: request.timeout.or(Some(self.task_timeout)),
            retry_count: 0,
            max_retries: request.max_retries,
        };

        // Add to queue in priority order
        {
            let mut queue = self.task_queue.write().await;
            queue.push(pending_task);

            // Sort by priority (highest first) and creation time (oldest first)
            queue.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| a.created_at.cmp(&b.created_at))
            });
        }

        // Try to process queue immediately
        self.process_queue().await?;

        Ok(task_id)
    }

    /// Execute a task immediately (bypassing the queue)
    pub async fn execute_task_immediately(
        &self,
        request: TaskRequest,
        context: AgentContext,
    ) -> Result<ExecutionResult> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();

        // Find appropriate agent
        let agent = self
            .registry
            .find_agent_for_task(&request.task_type)
            .await
            .ok_or_else(|| {
                anyhow::anyhow!("No agent found for task type: {}", request.task_type)
            })?;

        let agent_id = agent.id().to_string();

        // Execute task with timeout
        let task_timeout = request.timeout.unwrap_or(self.task_timeout);
        let result = timeout(
            task_timeout,
            agent.execute_task(&request.task_type, &context, &request.params),
        )
        .await;

        let execution_time = start_time.elapsed();

        let execution_result = match result {
            Ok(Ok(task_result)) => {
                // Record successful execution
                self.record_task_execution(
                    &request.task_type,
                    &agent_id,
                    execution_time,
                    task_result.success,
                )
                .await;

                ExecutionResult {
                    task_id,
                    result: task_result,
                    agent_id,
                    execution_time,
                    retry_count: 0,
                    completed_at: chrono::Utc::now(),
                }
            }
            Ok(Err(e)) => {
                // Record failed execution
                self.record_task_execution(&request.task_type, &agent_id, execution_time, false)
                    .await;

                ExecutionResult {
                    task_id,
                    result: TaskResult::failure(format!("Task execution failed: {e}")),
                    agent_id,
                    execution_time,
                    retry_count: 0,
                    completed_at: chrono::Utc::now(),
                }
            }
            Err(_) => {
                // Record timeout as failure
                self.record_task_execution(&request.task_type, &agent_id, execution_time, false)
                    .await;

                ExecutionResult {
                    task_id,
                    result: TaskResult::failure("Task execution timed out".to_string()),
                    agent_id,
                    execution_time,
                    retry_count: 0,
                    completed_at: chrono::Utc::now(),
                }
            }
        };

        Ok(execution_result)
    }

    /// Process the task queue
    async fn process_queue(&self) -> Result<()> {
        let current_running = self.running_tasks.read().await.len();

        if current_running >= self.max_concurrent_tasks {
            return Ok(()); // Queue will be processed when tasks complete
        }

        // For now, just log that we would process the queue
        // Actual implementation would handle async task execution
        tracing::info!(
            "Would process task queue (current running: {})",
            current_running
        );
        Ok(())
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: &str) -> Result<bool> {
        let cancel_sender = {
            let running = self.running_tasks.read().await;
            running.get(task_id).map(|task| task.cancel_sender.clone())
        };

        if let Some(sender) = cancel_sender {
            sender.send(()).await?;
            Ok(true)
        } else {
            // Try to remove from queue
            let mut queue = self.task_queue.write().await;
            let initial_len = queue.len();
            queue.retain(|t| t.id != task_id);
            Ok(queue.len() < initial_len)
        }
    }

    /// Get current execution statistics
    pub async fn get_statistics(&self) -> ExecutionStatistics {
        let queue = self.task_queue.read().await;
        let running = self.running_tasks.read().await;
        let stats = self.stats.read().await;

        // Calculate average execution time
        let average_execution_time = if stats.execution_times.is_empty() {
            Duration::from_secs(0)
        } else {
            let total_ms: u64 = stats
                .execution_times
                .iter()
                .map(|d| d.as_millis() as u64)
                .sum();
            Duration::from_millis(total_ms / stats.execution_times.len() as u64)
        };

        ExecutionStatistics {
            total_tasks_executed: stats.total_tasks_executed,
            successful_tasks: stats.successful_tasks,
            failed_tasks: stats.failed_tasks,
            average_execution_time,
            tasks_by_type: stats.tasks_by_type.clone(),
            tasks_by_agent: stats.tasks_by_agent.clone(),
            current_queue_size: queue.len(),
            current_running_tasks: running.len(),
        }
    }

    /// Record task execution statistics
    async fn record_task_execution(
        &self,
        task_type: &str,
        agent_id: &str,
        execution_time: Duration,
        success: bool,
    ) {
        let mut stats = self.stats.write().await;

        stats.total_tasks_executed += 1;
        if success {
            stats.successful_tasks += 1;
        } else {
            stats.failed_tasks += 1;
        }

        stats.execution_times.push(execution_time);
        // Keep only the last 1000 execution times to avoid memory growth
        if stats.execution_times.len() > 1000 {
            stats.execution_times.drain(0..500);
        }

        *stats
            .tasks_by_type
            .entry(task_type.to_string())
            .or_insert(0) += 1;
        *stats
            .tasks_by_agent
            .entry(agent_id.to_string())
            .or_insert(0) += 1;
    }

    /// Get information about running tasks
    pub async fn get_running_tasks(&self) -> Vec<TaskInfo> {
        let running = self.running_tasks.read().await;

        running
            .values()
            .map(|task| TaskInfo {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                agent_id: task.agent_id.clone(),
                started_at: task.started_at,
                running_time: task.started_at.elapsed(),
            })
            .collect()
    }

    /// Get information about queued tasks
    pub async fn get_queued_tasks(&self) -> Vec<QueuedTaskInfo> {
        let queue = self.task_queue.read().await;

        queue
            .iter()
            .map(|task| QueuedTaskInfo {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                priority: task.priority,
                created_at: task.created_at,
                waiting_time: task.created_at.elapsed(),
                retry_count: task.retry_count,
            })
            .collect()
    }

    /// Wait for all tasks to complete
    pub async fn wait_for_completion(&self, timeout_duration: Option<Duration>) -> Result<()> {
        let timeout_instant = timeout_duration.map(|d| Instant::now() + d);

        loop {
            let (queue_empty, running_empty) = {
                let queue = self.task_queue.read().await;
                let running = self.running_tasks.read().await;
                (queue.is_empty(), running.is_empty())
            };

            if queue_empty && running_empty {
                break;
            }

            if let Some(timeout_instant) = timeout_instant {
                if Instant::now() >= timeout_instant {
                    return Err(anyhow::anyhow!("Timeout waiting for task completion"));
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Shutdown the executor and cancel all tasks
    pub async fn shutdown(&self) -> Result<()> {
        // Cancel all running tasks
        let running_task_ids: Vec<String> = {
            let running = self.running_tasks.read().await;
            running.keys().cloned().collect()
        };

        for task_id in running_task_ids {
            let _ = self.cancel_task(&task_id).await;
        }

        // Clear the queue
        {
            let mut queue = self.task_queue.write().await;
            queue.clear();
        }

        // Wait for running tasks to complete (with timeout)
        let _ = timeout(Duration::from_secs(30), self.wait_for_completion(None)).await;

        Ok(())
    }
}

/// Information about a running task
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub task_type: String,
    pub agent_id: String,
    pub started_at: Instant,
    pub running_time: Duration,
}

/// Information about a queued task
#[derive(Debug, Clone)]
pub struct QueuedTaskInfo {
    pub id: String,
    pub task_type: String,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub waiting_time: Duration,
    pub retry_count: u32,
}

impl TaskRequest {
    pub fn new(task_type: String) -> Self {
        Self {
            task_type,
            params: HashMap::new(),
            priority: TaskPriority::Normal,
            timeout: None,
            max_retries: 0,
        }
    }

    #[must_use]
    pub fn with_params(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.params = params;
        self
    }

    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}
