//! Document query handlers — split by SRP.

pub mod detail;
pub mod download;
pub mod list;
pub mod mm_assets;
pub mod pages;
pub mod scan;
pub mod search;
pub mod track_status;

pub use detail::*;
pub use download::*;
pub use list::*;
pub use mm_assets::*;
pub use pages::*;
pub use scan::*;
pub use search::*;
pub use track_status::*;
