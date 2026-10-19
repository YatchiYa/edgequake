//! Organization invitations (SPEC-169) — "invite by email" without an SMTP dependency.
//!
//! The admin receives a single-use accept link (token shown once; only its SHA-256 is stored),
//! shares it, and the invitee either creates a password account or signs in with SSO.
//!
//! ```text
//!   handlers/members/{invitations,accept}   HTTP surface
//!        │
//!   model · token                            pure rules
//!        │
//!   store (trait) ◄── memory_store | pg_store
//! ```

pub mod memory_store;
pub mod model;
pub mod store;
pub mod token;

#[cfg(feature = "postgres")]
pub mod pg_store;

pub use memory_store::MemoryInvitationStore;
pub use model::{Invitation, INVITATION_TTL_DAYS};
pub use store::{InvitationStore, SharedInvitationStore};
