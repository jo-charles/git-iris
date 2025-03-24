# Git-Iris Code Review System

## 1. Overview

The Code Review System is a powerful feature of Git-Iris that provides AI-powered feedback on your code changes before they are committed. This tool helps improve code quality, catch potential issues early, and promote best practices.

## 2. Purpose

The main purposes of the Code Review System are:

1. To analyze staged code changes for potential issues
2. To provide constructive feedback on code quality
3. To suggest improvements that align with best practices
4. To highlight positive aspects of the changes

## 3. Components

### 3.1 Review Structure

The code review is structured into several sections:

1. **Summary**: Brief overview of the changes and main findings
2. **Code Quality Assessment**: Detailed evaluation of the overall code quality
3. **Positive Aspects**: Recognition of good practices and well-implemented features
4. **Issues Identified**: Problems or concerns found in the code
5. **Suggestions for Improvement**: Actionable recommendations to enhance the code

### 3.2 Integration with Git-Iris

The Code Review System seamlessly integrates with the rest of Git-Iris:

1. It uses the same context extraction system as commit message generation
2. It leverages the file analyzers to understand language-specific patterns
3. It benefits from the relevance scoring to focus on important changes
4. It respects custom instructions and presets for personalized feedback

## 4. Usage

### 4.1 Basic Usage

To generate a code review for your staged changes:

```bash
git-iris review
```

This will:

1. Analyze all staged files
2. Generate a comprehensive review
3. Display the review in the terminal

### 4.2 Command-line Options

The review command supports several options:

- `-i`, `--instructions`: Provide custom instructions for this review

  ```bash
  git-iris review -i "Focus on security best practices and error handling"
  ```

- `--provider`: Specify an LLM provider (openai, claude, ollama)

  ```bash
  git-iris review --provider claude
  ```

- `--preset`: Use a specific instruction preset

  ```bash
  git-iris review --preset security
  ```

- `-p`, `--print`: Print the generated review to stdout and exit
  ```bash
  git-iris review --print > my-review.txt
  ```

### 4.3 Custom Instructions

Custom instructions allow you to focus the review on specific aspects:

```bash
git-iris review -i "Pay special attention to concurrency issues and resource leaks"
```

You can also set default instructions in your configuration:

```bash
git-iris config --instructions "Focus on code maintainability and test coverage"
```

### 4.4 Using Presets

You can use instruction presets to guide the review:

```bash
git-iris review --preset security
```

This will apply security-focused instructions to the review process.

## 5. Review Output Format

The review output is formatted as follows:

```
✨ Code Review Summary ✨
A concise overview of the changes and major findings...

🔍 Code Quality Assessment
Detailed assessment of the overall code quality...

✅ Positive Aspects
1. Well-structured error handling throughout the changes
2. Comprehensive comments explain complex logic
...

❌ Issues Identified
1. Potential resource leak in file_handler.rs
2. Race condition in multi-threaded context
...

💡 Suggestions for Improvement
1. Consider using the `?` operator instead of manual match statements
2. Extract duplicated logic into a separate function
...
```

## 6. Best Practices

### 6.1 When to Use Code Reviews

- Before committing significant changes
- When implementing complex features
- When fixing critical bugs
- When working in unfamiliar parts of the codebase
- Before submitting pull requests

### 6.2 How to Get the Most from Code Reviews

1. **Stage Relevant Changes Only**: Stage only the files you want to be reviewed
2. **Provide Context**: Use custom instructions to focus the review on areas of concern
3. **Review Incrementally**: For large changes, stage and review in logical chunks
4. **Combine with Manual Review**: Use AI review as a supplement to, not replacement for, human review
5. **Iterate**: Address issues and run the review again to verify improvements

## 7. Advanced Usage

### 7.1 Combining with Commit Generation

After reviewing your code, you can generate a commit message that incorporates the review findings:

```bash
git-iris review
# Address issues identified
git-iris gen -i "Incorporate feedback from code review"
```

### 7.2 Language-Specific Reviews

For more targeted reviews, you can stage only files of a specific language:

```bash
git add *.rs
git-iris review -i "Focus on Rust-specific best practices"
```

### 7.3 Saving Reviews

You can save reviews for future reference:

```bash
git-iris review --print > reviews/feature-x-review.txt
```

## 8. Limitations

1. The quality of the review depends on the AI model used
2. Large diffs may be truncated due to token limits
3. Some language-specific nuances may not be captured
4. Complex architectural issues might need human review

## 9. Troubleshooting

If you encounter issues with the code review feature:

1. **No Review Generated**: Ensure you have staged changes using `git add`
2. **Low Quality Review**: Try using a different provider or more specific instructions
3. **Token Limit Errors**: Stage fewer files or increase the token limit in your configuration
4. **Missing Context**: Make sure your custom instructions provide necessary context

For further assistance, please refer to the [Git-Iris documentation](https://github.com/hyperb1iss/git-iris/wiki) or [open an issue](https://github.com/hyperb1iss/git-iris/issues) on the GitHub repository.
