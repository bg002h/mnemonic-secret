//! Subcommand handlers. Each module is independent and consumes Phase 1
//! foundation modules + the `ms-codec` library.

pub mod combine;
pub mod decode;
pub mod derive;
pub mod encode;
pub mod gen_man;
pub mod gui_schema;
pub mod inspect;
pub mod payload_lang;
pub mod repair;
pub mod split;
pub mod vectors;
pub mod verify;
