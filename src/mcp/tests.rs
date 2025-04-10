//! Tests for the MCP integration

#[cfg(test)]
mod tests {
    use crate::git::GitRepo;
    use crate::config::Config;
    use crate::mcp::tools::GitIrisToolbox;
    use crate::mcp::tools::releasenotes::ReleaseNotesRequest;
    use std::sync::Arc;
    use std::borrow::Cow;
    use rmcp::{ServerHandler, RoleServer};
    use rmcp::model::CallToolRequestParam;
    use rmcp::service::RequestContext;
    use serde_json::json;
    use crate::log_debug;

    // Unit test is disabled for now due to API incompatibilities
    // Will be re-enabled once the MCP API stabilizes
    #[allow(dead_code)]
    async fn test_release_notes_tool() {
        // Initialize dependencies
        let git_repo = match GitRepo::new_from_url(None) {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                log_debug!("Error creating git repo for test: {}", e);
                return;
            }
        };
        
        let config = Config::default();
        
        // Create the toolbox
        let toolbox = GitIrisToolbox::new(git_repo, config);
        
        // Create a request for release notes
        let request = ReleaseNotesRequest {
            from: "HEAD~5".to_string(), // Last 5 commits
            to: Some("HEAD".to_string()),
            detail_level: Some("minimal".to_string()),
            custom_instructions: Some("Keep it brief".to_string()),
        };
        
        // Convert to JSON and create an arguments object
        let args = json!({
            "from": "HEAD~5",
            "to": "HEAD",
            "detail_level": "minimal",
            "custom_instructions": "Keep it brief"
        });
        
        log_debug!("Test parameters: {:?}", args);
        log_debug!("Test request: {:?}", request);
        
        // The rest of the test is commented out until we can properly
        // integrate with the latest RMCP API
        /*
        let args_map = serde_json::from_value(args)
            .expect("Failed to convert to JsonObject");
        
        // Create tool params
        let params = CallToolRequestParam {
            name: Cow::Borrowed("git_iris_release_notes"),
            arguments: Some(args_map),
        };
        
        // Create a dummy context - no need for a real context in tests
        let context = RequestContext::<RoleServer> {
            id: None,
            peer: rmcp::Peer::default(),
        };
        
        // Call the tool
        let result = toolbox.call_tool(params, context).await;
        
        // Check the result
        match result {
            Ok(res) => {
                log_debug!("Tool call result: {:?}", res);
                assert!(!res.content.is_empty(), "Expected non-empty content in response");
                assert!(res.is_error.unwrap_or(false) == false, "Expected successful result");
            }
            Err(e) => {
                panic!("Tool call failed: {}", e);
            }
        }
        */
    }

    #[tokio::test]
    #[ignore = "API incompatibilities"]
    async fn test_generate_release_notes() {
        // This test is temporarily disabled due to API compatibility issues
        // with RequestContext construction
        /*
        use crate::git::GitRepo;
        use crate::config::Config as GitIrisConfig;
        use crate::mcp::tools::releasenotes::ReleaseNotesRequest;
        use crate::mcp::tools::GitIrisToolbox;
        use rmcp::{ServerHandler, RoleServer};
        use rmcp::model::{CallToolRequestParam, JsonObject};
        use rmcp::service::RequestContext;
        use std::borrow::Cow;
        use serde_json::json;
        
        // Initialize dependencies
        let git_repo = match GitRepo::new_from_url(None) {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                log_debug!("Error creating git repo for test: {}", e);
                return;
            }
        };
        
        let config = GitIrisConfig::load().unwrap_or_default();
        let toolbox = GitIrisToolbox::new(git_repo, config);
        
        // Create request parameters
        let args_value = json!({
            "from": "HEAD~5",
            "to": "HEAD",
            "detail_level": "minimal",
            "custom_instructions": "Keep it brief"
        });
        
        log_debug!("Test parameters: {:?}", args_value);
        
        // Convert to JsonObject for the CallToolRequestParam
        let args: JsonObject = serde_json::from_value(args_value)
            .expect("Failed to convert to JsonObject");
        
        // Create the call tool request
        let request = CallToolRequestParam {
            name: Cow::<'static, str>::Borrowed("git_iris_release_notes"),
            arguments: Some(args),
        };
        
        // TODO: Fix context creation and re-enable this test
        */
    }
} 