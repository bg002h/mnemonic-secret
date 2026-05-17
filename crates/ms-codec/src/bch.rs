//! BIP 93 codex32 BCH primitives for HRP `"ms"` (regular code only).
//!
//! Vendored from md-codec's structure at the v0.34.0 promotion (descriptor-mnemonic
//! commit `94069ea`) per plan §2.B.2 / D22. ms1 strings are all regular-code length
//! per `consts::VALID_STR_LENGTHS`, so the long-code primitives are intentionally
//! absent (mk-codec carries the long-code variants).
//!
//! All public per plan D22 (no `pub(crate)` half-private items in ms-codec): the
//! downstream `bch_decode` module (B.4) re-declares the 3 internal consts locally
//! per the Q3 lock — they stay bare-private here.
//!
//! The `MS_REGULAR_CONST` value is byte-exact with the toolkit's vendored copy at
//! `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:42` per Phase B.0 (e)
//! cross-check. The toolkit's v0.23.0 migration (Phase B.7) deletes its local
//! copy and delegates to this crate.

/// BCH(93,80,8) generator polynomial coefficients (5 × 65-bit).
///
/// Identical across mk/ms/md (the polynomial is BIP-93's; only the per-HRP
/// target residue differs).
pub const GEN_REGULAR: [u128; 5] = [
    0x19dc500ce73fde210,
    0x1bfae00def77fe529,
    0x1fbd920fffe7bee52,
    0x1739640bdeee3fdad,
    0x07729a039cfc75f5a,
];

/// MS-domain target residue: codex32's "SECRETSHARE32" Fe-vec packed in
/// big-endian 5-bit chunks (the natural u128 representation that
/// [`polymod_run`] produces for a valid ms1 input).
///
/// Empirical-stable across distinct valid ms1 strings — see the toolkit's
/// `ms_nums_target_is_stable_across_distinct_valid_strings` cell at
/// `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs` for the
/// stability derivation. Byte-exact with the toolkit per Phase B.0 (e).
pub const MS_REGULAR_CONST: u128 = 0x962958058f2c192a;

const POLYMOD_INIT: u128 = 0x23181b3;
const REGULAR_SHIFT: u32 = 60;
const REGULAR_MASK: u128 = 0x0fffffffffffffff;

fn polymod_step(residue: u128, value: u128) -> u128 {
    let b = residue >> REGULAR_SHIFT;
    let mut new_residue = ((residue & REGULAR_MASK) << 5) ^ value;
    for (i, &g) in GEN_REGULAR.iter().enumerate() {
        if (b >> i) & 1 != 0 {
            new_residue ^= g;
        }
    }
    new_residue
}

/// Run the BCH polymod over `values` starting from the BIP-93 initial residue.
///
/// Returns the final residue; callers XOR against the per-HRP target
/// constant ([`MS_REGULAR_CONST`]) to produce a checksum or to verify
/// one. Inputs are 5-bit symbols (`u8` in `0..32`); larger values are
/// reduced modulo 32 by the underlying step.
pub fn polymod_run(values: &[u8]) -> u128 {
    let mut residue = POLYMOD_INIT;
    for &v in values {
        residue = polymod_step(residue, v as u128);
    }
    residue
}

/// BIP 173-style HRP expansion: `[c >> 5 for c in hrp] ++ [0] ++ [c & 31 for c in hrp]`.
pub fn hrp_expand(hrp: &str) -> Vec<u8> {
    let bytes = hrp.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 2 + 1);
    for &c in bytes {
        out.push(c >> 5);
    }
    out.push(0);
    for &c in bytes {
        out.push(c & 31);
    }
    out
}

/// 13-symbol regular-code BCH checksum over `hrp_expand(hrp) || data || [0; 13]`.
pub fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13] {
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data);
    input.extend(std::iter::repeat_n(0, 13));
    let polymod = polymod_run(&input) ^ MS_REGULAR_CONST;
    let mut out = [0u8; 13];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = ((polymod >> (5 * (12 - i))) & 0x1F) as u8;
    }
    out
}

/// Verify a regular-code BCH checksum over the data-part-with-checksum.
pub fn bch_verify_regular(hrp: &str, data_with_checksum: &[u8]) -> bool {
    if data_with_checksum.len() < 13 {
        return false;
    }
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    polymod_run(&input) == MS_REGULAR_CONST
}
