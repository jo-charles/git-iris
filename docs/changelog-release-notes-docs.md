# Git-Iris Changelog and Release Notes System

## 1. Overview

The Changelog and Release Notes System is a powerful feature of Git-Iris that generates comprehensive documentation of changes between Git references. This tool helps improve project documentation, communication with users, and release management.

## 2. Purpose

The main purposes of the Changelog and Release Notes System are:

1. To automatically document changes between versions or commits
2. To categorize changes in a meaningful way
3. To highlight important information such as breaking changes
4. To provide metrics about the scope of changes
5. To improve communication with users and stakeholders

## 3. Components

### 3.1 Changelog Generator

The Changelog Generator creates a structured list of changes following the [Keep a Changelog](https://keepachangelog.com/) format. It categorizes changes into:

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed
- **Removed**: Features that were removed
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### 3.2 Release Notes Generator

The Release Notes Generator creates more comprehensive documentation that includes:

- A summary of the release
- Highlighted features
- Detailed sections with changes
- Breaking changes
- Upgrade notes
- Metrics

### 3.3 Integration with Git-Iris

Both components seamlessly integrate with the rest of Git-Iris:

1. They use the Git Repository analysis system to extract changes
2. They leverage the change analyzer to categorize and prioritize changes
3. They benefit from AI-powered content generation
4. They respect custom instructions and presets for personalized output

## 4. Usage

### 4.1 Generating a Changelog

To generate a changelog between two Git references:

```bash
git-iris changelog --from v1.0.0 --to v1.1.0
```

This will:

1. Analyze all changes between v1.0.0 and v1.1.0
2. Categorize the changes
3. Generate a well-formatted changelog
4. Display the changelog in the terminal

### 4.2 Generating Release Notes

To generate comprehensive release notes:

```bash
git-iris release-notes --from v1.0.0 --to v1.1.0
```

This will:

1. Analyze all changes between v1.0.0 and v1.1.0
2. Generate detailed release notes with sections and highlights
3. Display the release notes in the terminal

### 4.3 Command-line Options

Both commands support several options:

- `--from`: Starting Git reference (required)

  ```bash
  git-iris changelog --from v1.0.0
  ```

- `--to`: Ending Git reference (defaults to HEAD)

  ```bash
  git-iris changelog --from v1.0.0 --to v1.1.0
  ```

- `-i`, `--instructions`: Provide custom instructions

  ```bash
  git-iris changelog --from v1.0.0 --instructions "Focus on user-facing changes"
  ```

- `--provider`: Specify an LLM provider

  ```bash
  git-iris changelog --from v1.0.0 --provider anthropic
  ```

- `--preset`: Use a specific instruction preset

  ```bash
  git-iris changelog --from v1.0.0 --preset detailed
  ```

- `--detail-level`: Set the detail level (minimal, standard, detailed)

  ```bash
  git-iris changelog --from v1.0.0 --detail-level detailed
  ```

- `--gitmoji`: Enable or disable Gitmoji
  ```bash
  git-iris changelog --from v1.0.0 --gitmoji true
  ```

### 4.4 Custom Instructions

Custom instructions allow you to focus on specific aspects:

```bash
git-iris changelog --from v1.0.0 --instructions "Emphasize API changes and highlight potential breaking changes"
```

### 4.5 Using Presets

You can use instruction presets to guide the generation:

```bash
git-iris changelog --from v1.0.0 --preset conventional
```

This will apply conventional-commits-focused instructions to the generation process.

## 5. Output Format

### 5.1 Changelog Format

The changelog follows the Keep a Changelog format:

```
# Changelog

## [1.1.0] - 2023-06-15

### Added
- New feature A that allows users to X
- Support for Y integration

### Changed
- Improved performance of Z component
- Updated dependencies to latest versions

### Fixed
- Bug in error handling that caused crashes
- Issue with file permissions on Windows

### Security
- Fixed vulnerability in authentication system

## Metrics
- Total commits: 24
- Files changed: 42
- Lines inserted: 1,230
- Lines deleted: 530
```

### 5.2 Release Notes Format

The release notes provide a more comprehensive output:

```
# Release Notes - v1.1.0 (2023-06-15)

## Summary
Version 1.1.0 brings significant improvements to performance and reliability,
along with several new features and important bug fixes.

## Highlights
### Improved Performance
The core processing engine has been optimized, resulting in a 30% speed improvement
for most operations.

### Enhanced Security
Multiple security improvements, including better authentication and
permission handling.

## Features and Improvements
1. Added support for custom themes
2. Implemented new API for external integrations
3. Improved error handling and recovery
4. Added detailed logging for troubleshooting

## Bug Fixes
1. Fixed crash when processing invalid input
2. Resolved permission issues on Windows systems
3. Fixed memory leak in long-running processes

## Breaking Changes
- The `legacy_api` module has been removed in favor of the new API
- Configuration format has changed; see upgrade notes

## Upgrade Notes
- Run the migration script to update your configuration
- Update API clients to use the new endpoints

## Metrics
- Total commits: 24
- Files changed: 42
- Lines inserted: 1,230
- Lines deleted: 530
```

## 6. Best Practices

### 6.1 When to Generate Documentation

- Before releasing a new version
- When preparing release candidates
- For major feature additions
- When communicating changes to stakeholders

### 6.2 How to Get the Most from Generated Documentation

1. **Choose the Right Detail Level**: Select the appropriate detail level based on your audience
2. **Provide Context**: Use custom instructions to focus on what matters most to your users
3. **Review and Edit**: Always review the generated content before publishing
4. **Add to Version Control**: Commit generated changelogs to your repository for historical reference
5. **Use in Release Process**: Incorporate changelog generation into your release process

## 7. Advanced Usage

### 7.1 Generating Documentation for Specific Changes

You can generate documentation for specific types of changes by using custom instructions:

```bash
git-iris changelog --from v1.0.0 --instructions "Focus only on API changes and breaking changes"
```

### 7.2 Comparing Branch Changes

You can compare branches to see what changes will be introduced:

```bash
git-iris changelog --from main --to feature-branch
```

### 7.3 Saving Generated Documentation

You can save the generated content to files:

```bash
git-iris changelog --from v1.0.0 > CHANGELOG.md
git-iris release-notes --from v1.0.0 > RELEASE_NOTES.md
```

## 8. Limitations

1. The quality of the output depends on the AI model used
2. Large repositories with many changes may be truncated due to token limits
3. The system relies on meaningful commit messages for best results
4. Complex architectural changes might need manual editing

## 9. Troubleshooting

If you encounter issues with the changelog or release notes generation:

1. **Invalid References**: Ensure your Git references (tags, branches, or commits) are valid
2. **Token Limit Errors**: Try using a more specific range of commits or a higher token limit
3. **Low Quality Output**: Try using more specific instructions or a different LLM provider
4. **Missing Context**: Ensure your repository has good commit messages and summaries

For further assistance, please refer to the [Git-Iris documentation](https://github.com/hyperb1iss/git-iris/wiki) or [open an issue](https://github.com/hyperb1iss/git-iris/issues) on the GitHub repository.
