use crate::context::ProjectMetadata;
use crate::file_analyzers;
use crate::log_debug;
use anyhow::Result;
use futures::future::join_all;
use std::path::Path;
use tokio::task;

/// Analyzes a single file and extracts its metadata
pub async fn analyze_file(file_path: &str) -> Option<ProjectMetadata> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    let analyzer: Box<dyn file_analyzers::FileAnalyzer + Send + Sync> =
        file_analyzers::get_analyzer(file_name);

    log_debug!("Analyzing file: {}", file_path);

    if file_analyzers::should_exclude_file(file_path) {
        log_debug!("File excluded: {}", file_path);
        None
    } else if let Ok(content) = tokio::fs::read_to_string(file_path).await {
        let metadata = analyzer.extract_metadata(file_name, &content);
        log_debug!("Extracted metadata for {}: {:?}", file_name, metadata);
        Some(metadata)
    } else {
        log_debug!("Failed to read file: {}", file_path);
        None
    }
}

/// Extracts project metadata from a collection of files
///
/// Uses a batch processing approach to limit concurrent tasks
pub async fn extract_project_metadata(
    changed_files: &[String],
    batch_size: usize,
) -> Result<ProjectMetadata> {
    log_debug!(
        "Getting project metadata for {} changed files",
        changed_files.len()
    );

    let mut combined_metadata = ProjectMetadata::default();
    let mut any_file_analyzed = false;

    // Process files in batches to limit concurrent tasks
    for chunk in changed_files.chunks(batch_size) {
        let metadata_futures = chunk.iter().map(|file_path| {
            let file_path = file_path.clone();
            task::spawn(async move { analyze_file(&file_path).await })
        });

        let batch_results = join_all(metadata_futures).await;

        for metadata in batch_results.into_iter().flatten().flatten() {
            log_debug!("Merging metadata: {:?}", metadata);
            combined_metadata.merge(metadata);
            any_file_analyzed = true;
        }
    }

    log_debug!("Final combined metadata: {:?}", combined_metadata);

    if !any_file_analyzed {
        log_debug!("No files were analyzed!");
        combined_metadata.language = Some("Unknown".to_string());
    } else if combined_metadata.language.is_none() {
        combined_metadata.language = Some("Unknown".to_string());
    }

    Ok(combined_metadata)
}
