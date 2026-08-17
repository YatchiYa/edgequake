//! Document deletion handlers.
//!
//! | Sub-module | Responsibility                                    |
//! |------------|---------------------------------------------------|
//! | `single`   | Delete a single document by ID (cascade cleanup)  |
<<<<<<< HEAD
//! | `bulk`     | Delete all documents (bulk clear with skip logic)  |
//! | `impact`   | Read-only deletion impact preview                 |

=======
//! | `batch`    | Selected multi-document delete (SPEC-084 / GH-317)|
//! | `bulk`     | Delete all documents (bulk clear with skip logic)  |
//! | `impact`   | Read-only deletion impact preview                 |

mod batch;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
mod bulk;
mod impact;
mod single;

<<<<<<< HEAD
=======
pub use batch::*;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
pub use bulk::*;
pub use impact::*;
pub use single::*;
