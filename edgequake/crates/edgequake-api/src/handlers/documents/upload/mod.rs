//! Document upload handlers.

pub mod text_upload;
pub mod file_upload;
pub mod batch_upload;

pub use text_upload::*;
pub use file_upload::*;
pub use batch_upload::*;
