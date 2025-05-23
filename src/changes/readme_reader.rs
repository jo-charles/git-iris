use crate::config::Config;
use crate::git::GitRepo;
use crate::llm;
use anyhow::{Context, Result};
use std::sync::Arc;

pub async fn get_readme_summary(
    git_repo: Arc<GitRepo>,
    commit_ish: &str,
    config: &Config,
    provider_name: &str,
) -> Result<Option<String>> {
    match git_repo
        .get_readme_at_commit(commit_ish)
        .context("Failed to get README at specified commit")?
    {
        Some(readme_content) => {
            let summary = summarize_readme(config, provider_name, &readme_content).await?;
            Ok(Some(summary))
        }
        _ => Ok(None),
    }
}

async fn summarize_readme(
    config: &Config,
    provider_name: &str,
    readme_content: &str,
) -> Result<String> {
    let system_prompt = "You are an AI assistant tasked with summarizing README files for software projects. \
        Please provide a concise summary of the key points in the README, focusing on the following aspects:
        1. The project's main purpose and goals
        2. Key features and functionality
        3. Technologies or frameworks used
        4. Installation or setup instructions (if notable)
        5. Usage examples or quick start guide
        6. Any crucial information for users or contributors
        7. The style and vibe of the project (e.g., professional, casual, fun)

        Keep the summary informative yet brief, highlighting the most important aspects of the project.";

    let user_prompt = format!(
        "Please summarize the following README content, adhering to the guidelines provided:\n\n{readme_content}"
    );

    llm::get_message(config, provider_name, system_prompt, &user_prompt)
        .await
        .context("Failed to generate README summary")
}
