//! Document recovery handlers.
//!
//! | Sub-module   | Responsibility                                     |
//! |--------------|----------------------------------------------------|
//! | `reprocess`  | Reprocess with lifecycle admission SSOT (GAP-039) |
//! | `reprocess_one` | Per-document admit/commit/compensate (issue #385) |
//! | `stuck`      | Recover documents stuck in "processing" status     |
//! | `chunks`     | Retry/list failed chunks (FEAT0408, FEAT0409)      |

mod chunks;
mod reanalyze;
mod reprocess;
mod reprocess_one;
mod stuck;

pub use chunks::*;
pub use reanalyze::*;
pub use reprocess::*;
pub use stuck::*;
