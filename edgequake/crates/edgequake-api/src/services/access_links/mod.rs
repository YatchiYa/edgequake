//! One-time access links (SPEC-169): **magic sign-in** and **password reset** without SMTP.
//!
//! An organization admin issues a link for a member, shares it out of band, and the member opens
//! it once. Only the SHA-256 of the token is stored (same 256-bit scheme as invitations).
//!
//! ```text
//!   handlers/members/access_links   HTTP surface (issue · preview · redeem)
//!        │
//!   model (purpose, TTL, usability)  ·  guards (who may issue for whom)
//!        │
//!   store (trait) ◄── memory_store | pg_store
//! ```

pub mod memory_store;
pub mod model;
pub mod store;

#[cfg(feature = "postgres")]
pub mod pg_store;

pub use memory_store::MemoryAccessLinkStore;
pub use model::{AccessLink, AccessPurpose};
pub use store::{AccessLinkStore, SharedAccessLinkStore};
