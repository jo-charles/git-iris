//! Tests for the MCP integration

#[cfg(test)]
mod tests {
    use git_iris::config::Config;
    use git_iris::git::GitRepo;
    use git_iris::mcp::tools::GitIrisTools;
    use git_iris::mcp::tools::ReleaseNotesTool;
    use git_iris::mcp::tools::utils::GitIrisTool;
    use rmcp::model::{Content, RawContent};
    use serde_json::{Map, Value, json};
    use std::sync::Arc;

    // Helper function to create a Map<String, Value> from a JSON object
    fn create_params_map(json_value: &Value) -> Map<String, Value> {
        json_value
            .as_object()
            .expect("JSON value must be an object")
            .clone()
    }

    // Helper function to extract text from Content
    fn get_text_from_content(content: &Content) -> String {
        match &content.raw {
            RawContent::Text(text_content) => text_content.text.clone(),
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    #[ignore = "Run manually when testing MCP functionality"]
    async fn test_release_notes_tool_direct() {
        // Initialize dependencies
        let git_repo = match GitRepo::new_from_url(None) {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                eprintln!("Error creating git repo for test: {e}");
                return;
            }
        };

        let config = Config::default();

        // Create the release notes tool instance
        let tool = ReleaseNotesTool {
            from: "HEAD~5".to_string(), // Last 5 commits
            to: "HEAD".to_string(),
            detail_level: "minimal".to_string(),
            custom_instructions: "Keep it brief".to_string(),
            repository: String::new(),
            version_name: String::new(),
        };

        // Execute the tool directly
        match tool.execute(git_repo, config).await {
            Ok(result) => {
                let content = &result.content[0];
                let text = get_text_from_content(content);

                println!(
                    "Release notes content (first 200 chars): {}",
                    text.chars().take(200).collect::<String>()
                );

                assert!(!text.is_empty(), "Release notes should not be empty");
            }
            Err(e) => {
                panic!("Tool execution failed: {e}");
            }
        }
    }

    #[tokio::test]
    #[ignore = "Run manually when testing MCP functionality"]
    async fn test_tools_conversion() {
        // Create parameters for the release notes tool
        let args = create_params_map(&json!({
            "from": "HEAD~5",
            "to": "HEAD",
            "detail_level": "minimal",
            "custom_instructions": "Keep it brief",
            "version_name": ""
        }));

        // Add tool name to parameters for GitIrisTools::try_from
        let mut params = args.clone();
        params.insert(
            "name".to_string(),
            Value::String("git_iris_release_notes".to_string()),
        );

        // Convert to our GitIrisTools enum
        let tool_params =
            GitIrisTools::try_from(params).expect("Failed to convert parameters to GitIrisTools");

        // Verify the tool was created correctly
        match tool_params {
            GitIrisTools::ReleaseNotesTool(tool) => {
                assert_eq!(tool.from, "HEAD~5", "From field not set correctly");
                assert_eq!(tool.to, "HEAD", "To field not set correctly");
                assert_eq!(
                    tool.detail_level, "minimal",
                    "Detail level not set correctly"
                );
                assert_eq!(
                    tool.custom_instructions, "Keep it brief",
                    "Custom instructions not set correctly"
                );
                assert_eq!(tool.version_name, "", "Version name not set correctly");
            }
            GitIrisTools::ChangelogTool(_) => {
                // Not testing this variant in this test
                println!("ChangelogTool variant not being tested in this test");
            }
            GitIrisTools::CommitTool(_) => {
                // Not testing this variant in this test
                println!("CommitTool variant not being tested in this test");
            }
            GitIrisTools::CodeReviewTool(_) => {
                // Not testing this variant in this test
                println!("CodeReviewTool variant not being tested in this test");
            }
        }
    }
}
