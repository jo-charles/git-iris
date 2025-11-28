//! Utility functions for Iris Studio
//!
//! Common utilities used across the TUI, including string truncation.

use unicode_width::UnicodeWidthStr;

// ═══════════════════════════════════════════════════════════════════════════════
// String Truncation Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Truncate a string to a maximum character count, adding "..." if truncated.
///
/// This is useful for simple text truncation where unicode display width
/// isn't critical (e.g., log previews, notifications).
///
/// # Example
/// ```ignore
/// let result = truncate_chars("Hello, World!", 8);
/// assert_eq!(result, "Hello...");
/// ```
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else if max_chars <= 3 {
        s.chars().take(max_chars).collect()
    } else {
        format!("{}...", s.chars().take(max_chars - 3).collect::<String>())
    }
}

/// Truncate a string to a maximum display width, adding "…" if truncated.
///
/// This accounts for unicode character display widths (e.g., CJK characters
/// take 2 columns, emoji may take 2, etc.). Essential for TUI rendering.
///
/// # Example
/// ```ignore
/// let result = truncate_width("Hello, World!", 8);
/// assert_eq!(result, "Hello,…");
/// ```
pub fn truncate_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let s_width = s.width();
    if s_width <= max_width {
        return s.to_string();
    }

    if max_width <= 1 {
        return ".".to_string();
    }

    // Reserve space for ellipsis (width = 1 for "…")
    let target_width = max_width - 1;

    let mut result = String::new();
    let mut current_width = 0;

    for ch in s.chars() {
        let ch_width = ch.to_string().width();
        if current_width + ch_width > target_width {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }

    result.push('…');
    result
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_chars_no_truncation() {
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_chars_with_truncation() {
        assert_eq!(truncate_chars("hello world", 8), "hello...");
        assert_eq!(truncate_chars("hello world", 6), "hel...");
    }

    #[test]
    fn test_truncate_chars_edge_cases() {
        assert_eq!(truncate_chars("hello", 0), "");
        assert_eq!(truncate_chars("hello", 3), "hel");
        assert_eq!(truncate_chars("hello", 2), "he");
    }

    #[test]
    fn test_truncate_width_no_truncation() {
        assert_eq!(truncate_width("hello", 10), "hello");
        assert_eq!(truncate_width("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_width_with_truncation() {
        assert_eq!(truncate_width("hello world", 8), "hello w…");
        assert_eq!(truncate_width("hello world", 6), "hello…");
    }

    #[test]
    fn test_truncate_width_edge_cases() {
        assert_eq!(truncate_width("hello", 0), "");
        assert_eq!(truncate_width("hello", 1), ".");
        assert_eq!(truncate_width("hello", 2), "h…");
    }

    #[test]
    fn test_truncate_width_unicode() {
        // CJK characters are typically 2 columns wide
        let cjk = "你好世界"; // 8 columns wide (4 chars x 2)
        assert_eq!(cjk.width(), 8);

        let result = truncate_width(cjk, 6);
        assert!(result.width() <= 6);
    }
}
