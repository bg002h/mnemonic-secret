//! `ms-codec` — reference implementation of the **ms1** backup format (HRP `ms`).
//!
//! Status: pre-v0.1.0. Wire format and public API are specified in
//! [`design/SPEC_ms_v0_1.md`](../../design/SPEC_ms_v0_1.md). See also
//! [`MIGRATION.md`](../../MIGRATION.md) for the v0.1 → v0.2 contract.
//!
//! v0.1 emits BIP-39 entropy only (16/20/24/28/32 B). Direct BIP-32 master seed
//! and xpriv payloads are reserved-not-emitted in v0.1 and deferred to v0.2+
//! with separate framing (they overflow BIP-93 codex32's length brackets when
//! prepended with the v0.2-migration prefix byte).

#![cfg_attr(not(test), deny(missing_docs))]

pub mod consts;
pub mod error;
pub mod payload;
pub mod tag;

pub use error::{Error, Result};
pub use payload::{Payload, PayloadKind};
pub use tag::Tag;
