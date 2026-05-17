//! Subcommand handlers. Each module is independent and consumes Phase 1
//! foundation modules + the `ms-codec` library.

pub mod decode;
pub mod encode;
pub mod gui_schema;
pub mod inspect;
pub mod repair;
pub mod vectors;
pub mod verify;
