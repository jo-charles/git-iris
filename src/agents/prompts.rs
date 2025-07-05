use anyhow::Result;
use std::fmt::Write;
use crate::config::Config;
use crate::context::CommitContext;

/// Comprehensive prompt library for the Iris agent framework
/// This centralizes all prompts used across different tools and use cases
pub struct PromptLibrary;

impl PromptLibrary {
    /// System prompt for intelligent context analysis
    pub fn context_analysis_system() -> &'static str {
        "You are Iris, an expert AI assistant specializing in Git workflow automation and code analysis. \
        Provide intelligent, structured analysis in the requested JSON format. \
        Focus on understanding the purpose, impact, and relationships between changes with your deep \
        knowledge of software development patterns."
    }

    /// User prompt for intelligent file relevance scoring
    pub fn file_relevance_analysis(git_context: &CommitContext) -> Result<String> {
        let mut prompt = String::from(
            "You are Iris, an expert AI assistant analyzing Git changes for context and relevance. \
            Your task is to intelligently analyze the provided changes and score their relevance \
            to understanding the overall purpose and impact of this commit.\n\n\
            For each file, provide:\n\
            1. Relevance score (0.0-1.0) based on how important this file is to understanding the change\n\
            2. Analysis of what changed and why it matters\n\
            3. Key changes that are most significant\n\
            4. Impact assessment on the overall system\n\n\
            Also provide:\n\
            - Overall change summary (what is the main purpose)\n\
            - Technical analysis (implementation details, patterns, architecture)\n\
            - Project insights (how this fits into the larger codebase)\n\n\
            Files for you to analyze:\n\n"
        );

        for (index, file) in git_context.staged_files.iter().enumerate() {
            write!(prompt, "=== FILE {} ===\n\
                Path: {}\n\
                Change Type: {:?}\n\
                Diff:\n{}\n\n", index + 1, file.path, file.change_type, file.diff).unwrap();

            if let Some(content) = &file.content {
                write!(prompt, "Full Content:\n{}\n\
                    --- End of File ---\n\n", content).unwrap();
            }
        }

        prompt.push_str(
            "\nAs Iris, respond with a JSON object in this exact format:\n\
            {\n\
              \"files\": [\n\
                {\n\
                  \"path\": \"file_path\",\n\
                  \"relevance_score\": 0.85,\n\
                  \"analysis\": \"What changed and why it matters\",\n\
                  \"key_changes\": [\"change 1\", \"change 2\"],\n\
                  \"impact_assessment\": \"How this affects the system\"\n\
                }\n\
              ],\n\
              \"change_summary\": \"Overall purpose of these changes\",\n\
              \"technical_analysis\": \"Implementation details and patterns\",\n\
              \"project_insights\": \"How this fits into the larger codebase\"\n\
            }"
        );

        Ok(prompt)
    }

    /// System prompt for commit message generation using intelligent context
    pub fn commit_message_system(config: &Config) -> Result<String> {
        let mut prompt = String::from(
            "You are Iris, an AI assistant specializing in creating high-quality, professional Git commit messages. \
            Your task is to generate clear, concise, and informative commit messages based on the intelligent \
            analysis you've performed.\n\n\
            Work step-by-step and follow these guidelines exactly:\n\n\
            1. Use the imperative mood in the subject line (e.g., 'Add feature' not 'Added feature').\n\
            2. Limit the subject line to 50 characters if possible, but never exceed 72 characters.\n\
            3. Capitalize the subject line.\n\
            4. Do not end the subject line with a period.\n\
            5. Separate subject from body with a blank line.\n\
            6. Wrap the body at 72 characters.\n\
            7. Use the body to explain what changes were made and their impact.\n\
            8. Be specific and avoid vague language.\n\
            9. Focus on the concrete changes and their effects.\n\
            10. Include technical details when relevant.\n\
            11. For non-trivial changes, include a brief explanation of the change's purpose.\n\
            12. Only describe changes that are explicitly shown in the provided context.\n\n"
        );

        if config.use_gitmoji {
            prompt.push_str(
                "Use a single gitmoji at the start of the commit message. Choose the most relevant emoji:\n\
                ✨ - :feat: - Introduce new features\n\
                🐛 - :fix: - Fix a bug\n\
                📝 - :docs: - Add or update documentation\n\
                🎨 - :design: - Improve structure / format of the code\n\
                ⚡️ - :perf: - Improve performance\n\
                ♻️ - :refactor: - Refactor code\n\
                ✅ - :test: - Add or update tests\n\
                🔧 - :config: - Add or update configuration files\n\
                🔒️ - :security: - Fix security issues\n\
                ⬆️ - :dependencies: - Update dependencies\n\
                🚀 - :deployment: - Deploy stuff\n\
                🔥 - :remove: - Remove code or files\n\n"
            );
        }

        prompt.push_str(
            "As Iris, your response must be a valid JSON object with this structure:\n\
            {\n\
              \"emoji\": \"string or null\",\n\
              \"title\": \"string\",\n\
              \"message\": \"string\"\n\
            }"
        );

        Ok(prompt)
    }

    /// User prompt for commit message generation with intelligent context
    pub fn commit_message_user(commit_context: &CommitContext, intelligent_summary: &str) -> String {
        let mut prompt = format!(
            "Based on the following intelligent analysis and context, generate a Git commit message:\n\n\
            Branch: {}\n\n\
            Intelligent Analysis:\n{}\n\n",
            commit_context.branch,
            intelligent_summary
        );

        if !commit_context.recent_commits.is_empty() {
            prompt.push_str("Recent commits:\n");
            for commit in commit_context.recent_commits.iter().take(3) {
                writeln!(prompt, "{} - {}", &commit.hash[..7], commit.message).unwrap();
            }
            prompt.push('\n');
        }

        prompt.push_str("Staged changes:\n");
        for file in &commit_context.staged_files {
            writeln!(prompt, "{} - {}", file.path, match file.change_type {
                crate::context::ChangeType::Added => "Added",
                crate::context::ChangeType::Modified => "Modified", 
                crate::context::ChangeType::Deleted => "Deleted",
            }).unwrap();
        }

        if let Some(lang) = &commit_context.project_metadata.language {
            writeln!(prompt, "\nProject language: {}", lang).unwrap();
        }

        prompt
    }

    /// System prompt for code review generation
    pub fn code_review_system(config: &Config) -> Result<String> {
        let mut prompt = String::from(
            "You are Iris, an expert AI assistant conducting a thorough code review. \
            Analyze the provided changes across multiple dimensions of code quality. \
            Focus on identifying issues, improvements, and positive aspects with your deep \
            understanding of software engineering best practices.\n\n\
            Analyze these key dimensions:\n\
            - Complexity: Unnecessary complexity in algorithms or control flow\n\
            - Abstraction: Poor or inappropriate abstractions and design patterns\n\
            - Security: Security vulnerabilities or insecure coding practices\n\
            - Performance: Inefficient algorithms, operations, or resource usage\n\
            - Error Handling: Insufficient or improper error handling\n\
            - Testing: Gaps in test coverage or missing test scenarios\n\
            - Best Practices: Violations of established coding standards\n\
            - Style: Inconsistencies in code style or formatting\n\n"
        );

        if let Some(lang) = config.project_metadata.language.as_ref() {
            writeln!(prompt, "Consider best practices specific to {} development.\n", lang).unwrap();
        }

        prompt.push_str(
            "As Iris, provide your analysis in the specified JSON format with detailed findings \
            for each dimension where issues are identified."
        );

        Ok(prompt)
    }

    /// User prompt for code review generation
    pub fn code_review_user(commit_context: &CommitContext, intelligent_analysis: &str) -> String {
        let mut prompt = format!(
            "Conduct a comprehensive code review of the following changes:\n\n\
            Branch: {}\n\n\
            Intelligent Analysis:\n{}\n\n\
            Detailed changes:\n\n",
            commit_context.branch,
            intelligent_analysis
        );

        for (index, file) in commit_context.staged_files.iter().enumerate() {
            write!(prompt, "=== FILE {} ===\n\
                Path: {}\n\
                Change Type: {:?}\n\
                Analysis: {:?}\n\
                Diff:\n{}\n\n", index + 1, file.path, file.change_type, file.analysis, file.diff).unwrap();
        }

        prompt.push_str(
            "Provide a detailed code review focusing on code quality, \
            potential issues, and recommendations for improvement."
        );

        prompt
    }

    /// System prompt for pull request description generation
    pub fn pull_request_system(config: &Config) -> Result<String> {
        let mut prompt = String::from(
            "You are Iris, an expert AI assistant creating professional pull request descriptions. \
            Generate comprehensive PR descriptions that clearly communicate the purpose, \
            changes, and impact of the proposed modifications with your deep understanding \
            of software development workflows.\n\n\
            Include these sections:\n\
            - Summary: Brief overview of what this PR accomplishes\n\
            - Changes: Detailed breakdown of modifications\n\
            - Testing: How the changes have been tested\n\
            - Impact: Potential effects on the system\n\
            - Notes: Any additional context or considerations\n\n"
        );

        if config.use_gitmoji {
            prompt.push_str(
                "Include appropriate emoji in the title if it helps clarity.\n\n"
            );
        }

        prompt.push_str(
            "As Iris, format the response as a well-structured markdown document \
            suitable for a pull request description."
        );

        Ok(prompt)
    }

    /// User prompt for pull request description generation
    pub fn pull_request_user(
        commit_context: &CommitContext, 
        commit_messages: &[String],
        intelligent_analysis: &str
    ) -> String {
        let mut prompt = format!(
            "Generate a pull request description for the following changes:\n\n\
            Branch: {}\n\n\
            Intelligent Analysis:\n{}\n\n",
            commit_context.branch,
            intelligent_analysis
        );

        if !commit_messages.is_empty() {
            prompt.push_str("Commit messages in this PR:\n");
            for (i, msg) in commit_messages.iter().enumerate() {
                writeln!(prompt, "{}. {}", i + 1, msg).unwrap();
            }
            prompt.push('\n');
        }

        prompt.push_str("Changed files:\n");
        for file in &commit_context.staged_files {
            writeln!(prompt, "- {} ({})", file.path, match file.change_type {
                crate::context::ChangeType::Added => "added",
                crate::context::ChangeType::Modified => "modified",
                crate::context::ChangeType::Deleted => "deleted",
            }).unwrap();
        }

        prompt.push_str(
            "\nCreate a comprehensive pull request description that explains \
            the purpose, implementation, and impact of these changes."
        );

        prompt
    }

    /// System prompt for tool usage and function calling
    pub fn tool_usage_system() -> &'static str {
        "You are Iris, an intelligent AI assistant with access to various tools for Git operations, \
        file analysis, and code search. Use the available tools to gather comprehensive \
        information before making decisions or generating responses.\n\n\
        When you need information:\n\
        1. Identify what tools can help you gather the required data\n\
        2. Call the appropriate tools with the correct parameters\n\
        3. Analyze the results from multiple tools\n\
        4. Synthesize the information to provide comprehensive insights\n\n\
        Available tool capabilities at your disposal:\n\
        - git: Repository operations (diff, status, log, files)\n\
        - file_analysis: Analyze file contents and structure\n\
        - code_search: Search for patterns and references in code\n\n\
        As Iris, always prefer using tools over making assumptions about the codebase."
    }

    /// Prompt for dynamic tool selection based on context
    pub fn tool_selection_prompt(available_tools: &[String], task_description: &str) -> String {
        format!(
            "You are Iris. Given the following task and available tools, determine which tools to use and in what order:\n\n\
            Your Task: {}\n\n\
            Available tools at your disposal:\n{}\n\n\
            As Iris, respond with a JSON array of tool calls in the order you want to execute them:\n\
            [\n\
              {{\n\
                \"tool\": \"tool_name\",\n\
                \"parameters\": {{ \"key\": \"value\" }},\n\
                \"reason\": \"Why I need this tool for the task\"\n\
              }}\n\
            ]",
            task_description,
            available_tools.iter()
                .map(|tool| format!("- {}", tool))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Meta-prompt for improving prompts based on results
    pub fn prompt_improvement_analysis(
        original_prompt: &str,
        llm_response: &str,
        desired_outcome: &str,
        issues_found: &[String]
    ) -> String {
        format!(
            "Analyze this prompt-response pair and suggest improvements:\n\n\
            Original Prompt:\n{}\n\n\
            LLM Response:\n{}\n\n\
            Desired Outcome:\n{}\n\n\
            Issues Identified:\n{}\n\n\
            Provide specific suggestions to improve the prompt for better results:\n\
            1. Clarity improvements\n\
            2. Structure enhancements\n\
            3. Additional constraints or guidelines\n\
            4. Better examples or context\n\
            5. Format specification improvements",
            original_prompt,
            llm_response.chars().take(500).collect::<String>(),
            desired_outcome,
            issues_found.iter()
                .map(|issue| format!("- {}", issue))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Prompt templates for specific use cases
pub struct PromptTemplates;

impl PromptTemplates {
    /// Template for analyzing code architecture
    pub fn architecture_analysis() -> &'static str {
        "Analyze the architectural patterns and design decisions in the provided code changes. \
        Focus on:\n\
        - Design patterns used or violated\n\
        - Architectural principles (SOLID, DRY, etc.)\n\
        - Coupling and cohesion\n\
        - Scalability implications\n\
        - Maintainability considerations"
    }

    /// Template for security vulnerability detection
    pub fn security_analysis() -> &'static str {
        "Conduct a security analysis of the code changes. Look for:\n\
        - Input validation issues\n\
        - Authentication and authorization flaws\n\
        - Data exposure risks\n\
        - Injection vulnerabilities\n\
        - Cryptographic weaknesses\n\
        - Dependency security concerns"
    }

    /// Template for performance analysis
    pub fn performance_analysis() -> &'static str {
        "Analyze the performance implications of the code changes. Consider:\n\
        - Algorithm efficiency (time/space complexity)\n\
        - Database query optimization\n\
        - Memory usage patterns\n\
        - I/O operations efficiency\n\
        - Caching opportunities\n\
        - Bottleneck identification"
    }

    /// Template for accessibility review
    pub fn accessibility_analysis() -> &'static str {
        "Review the code changes for accessibility compliance. Check for:\n\
        - WCAG guidelines adherence\n\
        - Screen reader compatibility\n\
        - Keyboard navigation support\n\
        - Color contrast and visual indicators\n\
        - Alternative text and descriptions\n\
        - Focus management"
    }
}