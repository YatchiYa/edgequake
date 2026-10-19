//! One-time access links and password management (SPEC-169).
//!
//! ```text
//!   issue (admin)  ─►  token shown once  ─►  preview / redeem (public, token-gated)
//!                                              ├─ magic_login    → signed in
//!                                              └─ password_reset → new password → signed in
//!   password (self-service change)
//! ```
//!
//! No SMTP dependency: like invitations, the token is returned once and the admin shares it.
//! `guards` holds the pure authorization rules; `session` bridges to the login machinery.

pub mod guards;
pub mod issue;
pub mod password;
pub mod redeem;
mod session;

pub use issue::issue_access_link;
pub use password::change_password;
pub use redeem::{preview_access_link, redeem_access_link};
