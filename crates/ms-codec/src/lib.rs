//! `ms-codec` — reference implementation of the **ms1** backup format (HRP `ms`).
//!
//! Status: pre-v0.1 scaffold. Wire format and public API are specified
//! in [`design/SPEC_ms_v0_1.md`](../../design/SPEC_ms_v0_1.md). The
//! brainstorm rationale chain is in
//! [`design/BRAINSTORM_ms_v0_1.md`](../../design/BRAINSTORM_ms_v0_1.md).
//! Phase-by-phase implementation is tracked in
//! [`design/IMPLEMENTATION_PLAN_ms_v0_1.md`](../../design/IMPLEMENTATION_PLAN_ms_v0_1.md).
//!
//! # v0.1 → v0.2 migration contract
//!
//! v0.1 always emits BIP-93 threshold = 0 (single-string secret).
//! v0.2 will add K-of-N share encoding. v0.1 reserves a `0x00`
//! payload-prefix byte so v0.2 strings (prefix ≥ `0x01`) never collide
//! with v0.1 strings on disambiguation. See `MIGRATION.md`.

#![cfg_attr(not(test), deny(missing_docs))]
