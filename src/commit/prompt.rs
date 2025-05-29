use super::review::{GeneratedReview, QualityDimension};
use super::types::GeneratedMessage;
use crate::common::get_combined_instructions;
use crate::config::Config;
use crate::context::{ChangeType, CommitContext, ProjectMetadata, RecentCommit, StagedFile};
use crate::gitmoji::{apply_gitmoji, get_gitmoji_list};

use super::relevance::RelevanceScorer;
use crate::log_debug;
use std::collections::HashMap;
use std::fmt::Write;

pub fn create_system_prompt(config: &Config) -> anyhow::Result<String> {
    let commit_schema = schemars::schema_for!(GeneratedMessage);
    let commit_schema_str = serde_json::to_string_pretty(&commit_schema)?;

    let mut prompt = String::from(
        "You are an AI assistant specializing in creating high-quality, professional Git commit messages. \
        Your task is to generate clear, concise, and informative commit messages based solely on the provided context.
        
        Work step-by-step and follow these guidelines exactly:

        1. Use the imperative mood in the subject line (e.g., 'Add feature' not 'Added feature').
        2. Limit the subject line to 50 characters if possible, but never exceed 72 characters.
        3. Capitalize the subject line.
        4. Do not end the subject line with a period.
        5. Separate subject from body with a blank line.
        6. Wrap the body at 72 characters.
        7. Use the body to explain what changes were made and their impact, and how they were implemented.
        8. Be specific and avoid vague language.
        9. Focus on the concrete changes and their effects, not assumptions about intent.
        10. If the changes are part of a larger feature or fix, state this fact if evident from the context.
        11. For non-trivial changes, include a brief explanation of the change's purpose if clearly indicated in the context.
        12. Do not include a conclusion or end summary section.
        13. Avoid common cliché words (like 'enhance', 'streamline', 'leverage', etc) and phrases.
        14. Don't mention filenames in the subject line unless absolutely necessary.
        15. Only describe changes that are explicitly shown in the provided context.
        16. If the purpose or impact of a change is not clear from the context, focus on describing the change itself without inferring intent.
        17. Do not use phrases like 'seems to', 'appears to', or 'might be' - only state what is certain based on the context.
        18. If there's not enough information to create a complete, authoritative message, state only what can be confidently determined from the context.
        19. NO YAPPING!

        Be sure to quote newlines and any other control characters in your response.

        The message should be based entirely on the information provided in the context,
        without any speculation or assumptions.
      ");

    prompt.push_str(get_combined_instructions(config).as_str());

    // Check if using conventional commits preset - if so, explicitly disable gitmoji
    let is_conventional = config.instruction_preset == "conventional";

    if config.use_gitmoji && !is_conventional {
        prompt.push_str(
            "\n\nUse a single gitmoji at the start of the commit message. \
          Choose the most relevant emoji from the following list:\n\n",
        );
        prompt.push_str(&get_gitmoji_list());
    } else if is_conventional {
        prompt.push_str(
            "\n\nIMPORTANT: This is using Conventional Commits format. \
          DO NOT include any emojis in the commit message. \
          Set the emoji field to null in your response.",
        );
    }

    prompt.push_str("
        Your response must be a valid JSON object with the following structure:

        {
          \"emoji\": \"string or null\",
          \"title\": \"string\",
          \"message\": \"string\"
        }

        Follow these steps to generate the commit message:

        1. Analyze the provided context, including staged changes, recent commits, and project metadata.
        2. Identify the main purpose of the commit based on the changes.
        3. Create a concise and descriptive title (subject line) for the commit.
        4. If using emojis (false unless stated below), select the most appropriate one for the commit type.
        5. Write a detailed message body explaining the changes, their impact, and any other relevant information.
        6. Ensure the message adheres to the guidelines above, and follows all of the additional instructions provided.
        7. Construct the final JSON object with the emoji (if applicable), title, and message.

         Here's a minimal example of the expected output format:

        {
          \"emoji\": \"✨\",
          \"title\": \"Add user authentication feature\",
          \"message\": \"Implement user authentication using JWT tokens\\n\\n- Add login and registration endpoints\\n- Create middleware for token verification\\n- Update user model to include password hashing\\n- Add unit tests for authentication functions\"
        }

        Ensure that your response is a valid JSON object matching this structure. Include an empty string for the emoji if not using one.
        "
    );

    prompt.push_str(&commit_schema_str);

    Ok(prompt)
}

pub fn create_user_prompt(context: &CommitContext) -> String {
    let scorer = RelevanceScorer::new();
    let relevance_scores = scorer.score(context);
    let detailed_changes = format_detailed_changes(&context.staged_files, &relevance_scores);

    let prompt = format!(
        "Based on the following context, generate a Git commit message:\n\n\
        Branch: {}\n\n\
        Recent commits:\n{}\n\n\
        Staged changes:\n{}\n\n\
        Project metadata:\n{}\n\n\
        Detailed changes:\n{}",
        context.branch,
        format_recent_commits(&context.recent_commits),
        format_staged_files(&context.staged_files, &relevance_scores),
        format_project_metadata(&context.project_metadata),
        detailed_changes
    );

    log_debug!(
        "Generated commit prompt for {} files ({} added, {} modified, {} deleted)",
        context.staged_files.len(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Added))
            .count(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Modified))
            .count(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Deleted))
            .count()
    );

    prompt
}

fn format_recent_commits(commits: &[RecentCommit]) -> String {
    commits
        .iter()
        .map(|commit| format!("{} - {}", &commit.hash[..7], commit.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_staged_files(files: &[StagedFile], relevance_scores: &HashMap<String, f32>) -> String {
    files
        .iter()
        .map(|file| {
            let relevance = relevance_scores.get(&file.path).unwrap_or(&0.0);
            format!(
                "{} ({:.2}) - {}",
                file.path,
                relevance,
                format_change_type(&file.change_type)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_project_metadata(metadata: &ProjectMetadata) -> String {
    format!(
        "Language: {}\nFramework: {}\nDependencies: {}",
        metadata.language.as_deref().unwrap_or("None"),
        metadata.framework.as_deref().unwrap_or("None"),
        metadata.dependencies.join(", ")
    )
}

fn format_detailed_changes(
    files: &[StagedFile],
    relevance_scores: &HashMap<String, f32>,
) -> String {
    let mut all_sections = Vec::new();

    // Add a statistical summary at the top
    let added_count = files
        .iter()
        .filter(|f| matches!(f.change_type, ChangeType::Added))
        .count();
    let modified_count = files
        .iter()
        .filter(|f| matches!(f.change_type, ChangeType::Modified))
        .count();
    let deleted_count = files
        .iter()
        .filter(|f| matches!(f.change_type, ChangeType::Deleted))
        .count();

    let summary = format!(
        "CHANGE SUMMARY:\n- {} file(s) added\n- {} file(s) modified\n- {} file(s) deleted\n- {} total file(s) changed",
        added_count,
        modified_count,
        deleted_count,
        files.len()
    );
    all_sections.push(summary);

    // First section: File summaries with diffs
    let diff_section = files
        .iter()
        .map(|file| {
            let relevance = relevance_scores.get(&file.path).unwrap_or(&0.0);
            // Add emoji indicators for change types
            let change_indicator = match file.change_type {
                ChangeType::Added => "➕",
                ChangeType::Modified => "✏️",
                ChangeType::Deleted => "🗑️",
            };

            format!(
                "{} File: {} (Relevance: {:.2})\nChange Type: {}\nAnalysis:\n{}\n\nDiff:\n{}",
                change_indicator,
                file.path,
                relevance,
                format_change_type(&file.change_type),
                file.analysis.join("\n"),
                file.diff
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    all_sections.push(format!(
        "=== DIFFS ({} files) ===\n\n{}",
        files.len(),
        diff_section
    ));

    // Second section: Full file contents (only for added/modified files)
    let content_files: Vec<_> = files
        .iter()
        .filter(|file| file.change_type != ChangeType::Deleted && file.content.is_some())
        .collect();

    if !content_files.is_empty() {
        let content_section = content_files
            .iter()
            .map(|file| {
                let change_indicator = match file.change_type {
                    ChangeType::Added => "➕",
                    ChangeType::Modified => "✏️",
                    ChangeType::Deleted => "",
                };

                format!(
                    "{} File: {}\nFull File Content:\n{}\n\n--- End of File ---",
                    change_indicator,
                    file.path,
                    file.content
                        .as_ref()
                        .expect("File content should be present for added/modified files")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        all_sections.push(format!(
            "=== FULL FILE CONTENTS ({} files) ===\n\n{}",
            content_files.len(),
            content_section
        ));
    }

    all_sections.join("\n\n====================\n\n")
}

fn format_change_type(change_type: &ChangeType) -> &'static str {
    match change_type {
        ChangeType::Added => "Added",
        ChangeType::Modified => "Modified",
        ChangeType::Deleted => "Deleted",
    }
}

pub fn process_commit_message(message: String, use_gitmoji: bool) -> String {
    if use_gitmoji {
        apply_gitmoji(&message)
    } else {
        message
    }
}

/// Creates a system prompt for code review generation
#[allow(clippy::too_many_lines)]
pub fn create_review_system_prompt(config: &Config) -> anyhow::Result<String> {
    let review_schema = schemars::schema_for!(GeneratedReview);
    let review_schema_str = serde_json::to_string_pretty(&review_schema)?;

    let mut prompt = String::from(
        "You are an AI assistant specializing in code reviews. \
        Your task is to provide a comprehensive, professional, and constructive review of the code changes provided.

        Work step-by-step and follow these guidelines exactly:

        1. Analyze the code changes carefully, focusing on:
           - Code quality and readability
           - Potential bugs or errors
           - Architecture and design patterns
           - Performance implications
           - Security considerations
           - Maintainability and testability

        2. Provide constructive feedback:
           - Be specific and actionable in your suggestions
           - Point out both strengths and areas for improvement
           - Explain why certain patterns or practices are problematic
           - Suggest alternative approaches when appropriate

        3. Focus on substantive issues:
           - Prioritize significant issues over minor stylistic concerns
           - Consider the context of the codebase and changes
           - Note potential edge cases or scenarios that might not be handled

        4. Be professional and constructive:
           - Frame feedback positively and constructively
           - Focus on the code, not the developer
           - Acknowledge good practices and improvements

        5. Analyze the following specific dimensions of code quality:
        ");

    // Add each dimension's description
    for dimension in QualityDimension::all() {
        prompt.push_str(dimension.description());
    }

    prompt.push_str(
        "
        For each dimension, identify specific issues with:
        - A severity level (Critical, High, Medium, Low)
        - Line number references or specific location in the code
        - Explanation of why it's problematic
        - Concrete recommendation for improvement

        Your review should be based entirely on the information provided in the context, without any speculation or assumptions.
      ");

    prompt.push_str(get_combined_instructions(config).as_str());

    prompt.push_str(
        "
        Your response must be a valid JSON object with the following structure:

        {
          \"summary\": \"A brief summary of the changes and their quality\",
          \"code_quality\": \"An assessment of the overall code quality\",
          \"suggestions\": [\"Suggestion 1\", \"Suggestion 2\", ...],
          \"issues\": [\"Issue 1\", \"Issue 2\", ...],
          \"positive_aspects\": [\"Positive aspect 1\", \"Positive aspect 2\", ...],",
    );

    // Add each dimension to the JSON schema
    let mut is_first = true;
    for dimension in QualityDimension::all() {
        let dim_name = match dimension {
            QualityDimension::Complexity => "complexity",
            QualityDimension::Abstraction => "abstraction",
            QualityDimension::Deletion => "deletion",
            QualityDimension::Hallucination => "hallucination",
            QualityDimension::Style => "style",
            QualityDimension::Security => "security",
            QualityDimension::Performance => "performance",
            QualityDimension::Duplication => "duplication",
            QualityDimension::ErrorHandling => "error_handling",
            QualityDimension::Testing => "testing",
            QualityDimension::BestPractices => "best_practices",
        };

        if is_first {
            is_first = false;
            write!(
                &mut prompt,
                "
          \"{dim_name}\": {{
            \"issues_found\": true/false,
            \"issues\": [
              {{
                \"description\": \"Brief description\",
                \"severity\": \"Critical/High/Medium/Low\",
                \"location\": \"filename.rs:line_numbers or path/to/file.rs:lines_range\",
                \"explanation\": \"Detailed explanation of the issue\",
                \"recommendation\": \"Specific suggestion for improvement\"
              }},
              ...
            ]
          }}"
            )
            .expect("write to string should not fail");
        } else {
            write!(
                &mut prompt,
                ",
          \"{dim_name}\": {{ ... similar structure ... }}"
            )
            .expect("write to string should not fail");
        }
    }

    prompt.push_str("
        }

        Follow these steps to generate the code review:

        1. Analyze the provided context, including staged changes and project metadata.
        2. Evaluate the code quality, looking for potential issues, improvements, and good practices.
        3. Create a concise summary of the changes and their quality.
        4. Analyze each of the code quality dimensions.
        5. For each dimension with issues, list them with appropriate severity, location, explanation, and recommendation.
        6. Provide overall suggestions for improvements.
        7. Identify specific issues found across all dimensions.
        8. Acknowledge positive aspects and good practices in the code.
        9. Construct the final JSON object with all components.

        Note: It's expected that not all dimensions will have issues. For dimensions without issues, set 'issues_found' to false and provide an empty issues array.

        Here's a minimal example of the expected output format (showing only two dimensions for brevity):

        {
          \"summary\": \"The changes implement a new authentication system with good separation of concerns, but lacks proper error handling in several places.\",
          \"code_quality\": \"The code is generally well-structured with clear naming conventions. The architecture follows established patterns, but there are some inconsistencies in error handling approaches.\",
          \"suggestions\": [\"Consider implementing a consistent error handling strategy across all authentication operations\", \"Add unit tests for edge cases in the token validation logic\"],
          \"issues\": [\"Missing error handling in the user registration flow\", \"Potential race condition in token refresh mechanism\"],
          \"positive_aspects\": [\"Good separation of concerns with clear service boundaries\", \"Consistent naming conventions throughout the added components\"],
          \"complexity\": {
            \"issues_found\": true,
            \"issues\": [
              {
                \"description\": \"Complex authentication flow with excessive nesting\",
                \"severity\": \"Medium\",
                \"location\": \"src/auth/auth_service.rs:45-67\",
                \"explanation\": \"The authentication validation contains 5 levels of nesting, making it difficult to follow the logic flow.\",
                \"recommendation\": \"Extract validation steps into separate functions and use early returns to reduce nesting\"
              }
            ]
          },
          \"error_handling\": {
            \"issues_found\": true,
            \"issues\": [
              {
                \"description\": \"Missing error handling in token refresh\",
                \"severity\": \"High\",
                \"location\": \"src/auth/auth_service.rs:102-120\",
                \"explanation\": \"The token refresh function doesn't properly handle network timeouts, potentially leaving users in an inconsistent state.\",
                \"recommendation\": \"Add explicit error handling for network timeouts with appropriate user feedback\"
              }
            ]
          },
          ... (other dimensions would be included with empty issues arrays if no issues found)
        }

        Ensure that your response is a valid JSON object matching this structure.
        "
    );

    prompt.push_str(&review_schema_str);

    Ok(prompt)
}

/// Creates a user prompt for code review generation
pub fn create_review_user_prompt(context: &CommitContext) -> String {
    let scorer = RelevanceScorer::new();
    let relevance_scores = scorer.score(context);
    let detailed_changes = format_detailed_changes(&context.staged_files, &relevance_scores);

    let prompt = format!(
        "Based on the following context, generate a code review:\n\n\
        Branch: {}\n\n\
        Recent commits:\n{}\n\n\
        Staged changes:\n{}\n\n\
        Project metadata:\n{}\n\n\
        Detailed changes:\n{}",
        context.branch,
        format_recent_commits(&context.recent_commits),
        format_staged_files(&context.staged_files, &relevance_scores),
        format_project_metadata(&context.project_metadata),
        detailed_changes
    );

    log_debug!(
        "Generated review prompt for {} files ({} added, {} modified, {} deleted)",
        context.staged_files.len(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Added))
            .count(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Modified))
            .count(),
        context
            .staged_files
            .iter()
            .filter(|f| matches!(f.change_type, ChangeType::Deleted))
            .count()
    );

    prompt
}
