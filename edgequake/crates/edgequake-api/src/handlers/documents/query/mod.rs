//! Document query handlers — split by SRP.

pub mod list;
pub mod detail;
pub mod track_status;
pub mod scan;

pub use list::*;
pub use detail::*;
pub use track_status::*;
pub use scan::*;
