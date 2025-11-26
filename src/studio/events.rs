//! Event handling for Iris Studio
//!
//! This module re-exports the handlers from the split handler modules.
//! For implementation details, see the `handlers/` directory.

pub use super::handlers::{Action, IrisQueryRequest, handle_key_event};
