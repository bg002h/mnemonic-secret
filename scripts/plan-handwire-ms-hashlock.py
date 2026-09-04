#!/usr/bin/env python3
"""plan-handwire-ms-hashlock.py <scratch-copy>

Applies IMPLEMENTATION_PLAN_ms_hashlock_H1.md's FRAGMENTS (edits to existing
files) to a scratch copy of mnemonic-secret, so plan-build-gate-ms.sh can
build the plan's new files against them. Every replacement is exact-anchor:
a missing anchor is an error naming the file, never a silent skip. Refuses to
run twice on one copy (sentinel .handwired)."""
import os, sys, re

root = sys.argv[1]
sentinel = os.path.join(root, ".handwired")
if os.path.exists(sentinel):
    sys.exit("already hand-wired: " + root)

def edit(path, pairs):
    full = os.path.join(root, path)
    s = open(full, encoding="utf-8").read()
    for old, new in pairs:
        if old not in s:
            sys.exit("anchor not found in %s:\n%s" % (path, old[:120]))
        s = s.replace(old, new, 1)
    open(full, "w", encoding="utf-8").write(s)
    print("  wired", path)

# ---- ms-codec ---------------------------------------------------------------
edit("crates/ms-codec/Cargo.toml", [
    ('getrandom = "0.3"',
     'getrandom = "0.3"\n# Hashlock derivation (spec §2), spelled as `me` spells them.\npbkdf2 = { version = "0.12", default-features = false, features = ["hmac"] }\nsha2 = "0.10"'),
    ('version = "0.7.0"', 'version = "0.8.0"'),
])
edit("crates/ms-codec/src/lib.rs", [
    ("pub mod error;", "pub mod error;\npub mod hashlock;"),
])
edit("crates/ms-codec/src/consts.rs", [
    ('pub const MNEM_PREFIX: u8 = 0x02;',
     'pub const MNEM_PREFIX: u8 = 0x02;\n\n/// v0.8 preimage-prefix byte: `[0x03][X:32]`, a hashlock preimage (SPEC_ms_hashlock §1).\npub const PREIMAGE_PREFIX: u8 = 0x03;\n\n/// The only string length a preimage single can have: 9 fixed + ceil(33*8/5)=53 payload + 13 cksum.\npub const VALID_PREIMAGE_STR_LENGTHS: &[usize] = &[75];\n\n/// 4-byte type tag carried by preimage SINGLES (SPEC_ms_hashlock §1, L14).\npub const TAG_HASH: [u8; 4] = *b"hash";'),
    ('pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk"];',
     'pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk", *b"hash"];'),
])
edit("crates/ms-codec/src/tag.rs", [
    ("use crate::consts::TAG_ENTR;", "use crate::consts::{TAG_ENTR, TAG_HASH};"),
    ("    pub const ENTR: Tag = Tag(TAG_ENTR);",
     "    pub const ENTR: Tag = Tag(TAG_ENTR);\n\n    /// The v0.8 emit-tag for a hashlock preimage single (id `hash`).\n    pub const HASH: Tag = Tag(TAG_HASH);"),
])
edit("crates/ms-codec/src/error.rs", [
    ("    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).\n    ReservedPrefixViolation {",
     "    /// A `0x03` payload whose length after the prefix byte is not 32 (SPEC_ms_hashlock §1).\n    PreimageLengthMismatch {\n        /// Bytes after the prefix byte -- the would-be X. Expected 32.\n        got: usize,\n    },\n    /// A single's tag names one kind and its prefix byte another (SPEC_ms_hashlock §1 rule 2).\n    TagKindMismatch {\n        /// The 4-byte tag observed.\n        tag: [u8; 4],\n        /// The prefix byte observed.\n        prefix: u8,\n    },\n    /// The OS CSPRNG could not fill the buffer (`getrandom` failed closed).\n    RandomnessUnavailable,\n    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).\n    ReservedPrefixViolation {"),
    ("            Error::ReservedPrefixViolation { got } => {",
     "            Error::PreimageLengthMismatch { got } => write!(\n                f,\n                \"preimage payload is {got} bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex characters)\"\n            ),\n            Error::TagKindMismatch { tag, prefix } => write!(\n                f,\n                \"tag {:?} does not name the kind the prefix byte 0x{prefix:02x} carries; refusing rather than reading one kind as another\",\n                String::from_utf8_lossy(tag)\n            ),\n            Error::RandomnessUnavailable => write!(f, \"the OS random source is unavailable; no preimage was produced\"),\n            Error::ReservedPrefixViolation { got } => {"),
])
edit("crates/ms-codec/src/payload.rs", [
    ("    /// BIP-39 mnemonic entropy with wordlist language tag (16/20/24/28/32 B entropy).\n    Mnem,\n}",
     "    /// BIP-39 mnemonic entropy with wordlist language tag (16/20/24/28/32 B entropy).\n    Mnem,\n    /// A hashlock preimage: exactly 32 B (SPEC_ms_hashlock §1).\n    Preimage,\n}\n\nimpl PayloadKind {\n    /// The tag a SINGLE of this kind carries: `entr` for the two seed kinds,\n    /// `hash` for a preimage. Decode CHECKS a single's tag against this; encode\n    /// refuses to emit a mismatch (SPEC_ms_hashlock §1 rule 2).\n    pub fn single_tag(self) -> crate::tag::Tag {\n        match self {\n            PayloadKind::Entr | PayloadKind::Mnem => crate::tag::Tag::ENTR,\n            PayloadKind::Preimage => crate::tag::Tag::HASH,\n        }\n    }\n}"),
    ("pub enum Payload {", "pub enum Payload {\n    /// A hashlock preimage, exactly 32 bytes; scrubbed on drop (SPEC_ms_hashlock §3).\n    Preimage(zeroize::Zeroizing<[u8; 32]>),"),
    ("            Payload::Mnem { .. } => PayloadKind::Mnem,\n        }",
     "            Payload::Mnem { .. } => PayloadKind::Mnem,\n            Payload::Preimage(_) => PayloadKind::Preimage,\n        }"),
    ("            Payload::Mnem { entropy, .. } => entropy,\n        }",
     "            Payload::Mnem { entropy, .. } => entropy,\n            Payload::Preimage(x) => &x[..],\n        }"),
    ("    pub fn validate(&self) -> Result<()> {\n        match self {\n            Payload::Entr(data) => {",
     "    pub fn validate(&self) -> Result<()> {\n        match self {\n            // A preimage's length is structural in the variant (SPEC_ms_hashlock §3).\n            Payload::Preimage(_) => Ok(()),\n            Payload::Entr(data) => {"),
])
edit("crates/ms-codec/src/envelope.rs", [
    ("use crate::consts::{", "use crate::consts::{PREIMAGE_PREFIX, "),
    ("        other => {\n            return Err(Error::ReservedPrefixViolation { got: other });\n        }",
     "        PREIMAGE_PREFIX => {\n            // 0x03 -> Preimage: LENGTH CHECK BEFORE CONSTRUCTION, so the entr\n            // length error never names a legal entr length as illegal and no\n            // slice index can panic (SPEC_ms_hashlock §1).\n            let rest = &data[1..];\n            let x: [u8; 32] = rest\n                .try_into()\n                .map_err(|_| Error::PreimageLengthMismatch { got: rest.len() })?;\n            Payload::Preimage(Zeroizing::new(x))\n        }\n        other => {\n            return Err(Error::ReservedPrefixViolation { got: other });\n        }"),
    ("        Payload::Mnem { language, entropy } => {\n            // [0x02 mnem-prefix] || [language] || entropy",
     "        Payload::Preimage(x) => {\n            // [0x03 preimage-prefix] || X\n            let mut v = Zeroizing::new(Vec::with_capacity(33));\n            v.push(PREIMAGE_PREFIX);\n            v.extend_from_slice(&x[..]);\n            v\n        }\n        Payload::Mnem { language, entropy } => {\n            // [0x02 mnem-prefix] || [language] || entropy"),
])
# Both doc-comment copies of the prefix table gain the 0x03 line; they differ
# only in column alignment, so the anchor is the line's head and BOTH must hit.
_p = os.path.join(root, "crates/ms-codec/src/envelope.rs")
_s = open(_p, encoding="utf-8").read()
if _s.count("/// - any other prefix") != 2:
    sys.exit("envelope.rs: expected exactly two `/// - any other prefix` doc lines")
_s = _s.replace("/// - any other prefix", "/// - `0x03` (`PREIMAGE_PREFIX`) → `Payload::Preimage(rest)` iff rest is 32 bytes\n/// - any other prefix", 2)
open(_p, "w", encoding="utf-8").write(_s)
print("  wired crates/ms-codec/src/envelope.rs (both prefix-table doc comments)")
edit("crates/ms-codec/src/decode.rs", [
    ("use crate::consts::{\n    RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_MNEM_STR_LENGTHS, VALID_STR_LENGTHS,\n};",
     "use crate::consts::{\n    RESERVED_NOT_EMITTED_V01, TAG_ENTR, TAG_HASH, VALID_MNEM_STR_LENGTHS,\n    VALID_PREIMAGE_STR_LENGTHS, VALID_STR_LENGTHS,\n};"),
    ("        PayloadKind::Mnem => VALID_MNEM_STR_LENGTHS,\n    }",
     "        PayloadKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        PayloadKind::Preimage => VALID_PREIMAGE_STR_LENGTHS,\n    }"),
    ("        x if x == TAG_ENTR => {\n            match payload {\n                Payload::Entr(data) => {",
     "        // Rule 6b (SPEC_ms_hashlock §1 rule 2): a single's tag must name the\n        // kind its prefix byte carries. Checked BEFORE the per-tag arms so a\n        // `hash` tag over a seed payload, or `entr` over a preimage, is refused\n        // rather than read as the other kind.\n        x if (x == TAG_ENTR || x == TAG_HASH) && tag != payload.kind().single_tag() => {\n            return Err(Error::TagKindMismatch {\n                tag: x,\n                prefix: crate::envelope::prefix_of(&payload),\n            });\n        }\n        x if x == TAG_HASH => {\n            // A preimage single: length is structural in the variant.\n            payload\n        }\n        x if x == TAG_ENTR => {\n            match payload {\n                Payload::Entr(data) => {"),
    ("                Payload::Mnem { language, entropy } => {\n                    let p = Payload::Mnem { language, entropy };\n                    // §4 rule 10: validate (language range + entropy length).\n                    p.validate()?;\n                    p\n                }\n            }",
     "                Payload::Mnem { language, entropy } => {\n                    let p = Payload::Mnem { language, entropy };\n                    // §4 rule 10: validate (language range + entropy length).\n                    p.validate()?;\n                    p\n                }\n                // Unreachable: rule 6b above refused the mismatch. Kept as a\n                // typed error, never a panic.\n                other => {\n                    return Err(Error::TagKindMismatch {\n                        tag: x,\n                        prefix: crate::envelope::prefix_of(&other),\n                    })\n                }\n            }"),
])
edit("crates/ms-codec/src/envelope.rs", [
    ("/// `Payload` is a closed 2-variant enum within this crate (`#[non_exhaustive]`\n/// only affects downstream crates), so the match is exhaustive.",
     "/// `Payload` is a closed 3-variant enum within this crate (`#[non_exhaustive]`\n/// only affects downstream crates), so the match is exhaustive."),
    ("pub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>> {",
     "/// The prefix byte a payload writes on the wire, for error reporting.\npub(crate) fn prefix_of(p: &Payload) -> u8 {\n    match p {\n        Payload::Entr(_) => RESERVED_PREFIX,\n        Payload::Mnem { .. } => MNEM_PREFIX,\n        Payload::Preimage(_) => PREIMAGE_PREFIX,\n    }\n}\n\npub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>> {"),
])
# The check sits AFTER the reserved-not-emitted check, not at the top of the fn:
# `seed`/`xprv` are not kind-naming ids, and the v0.1 SPEC §4 rule 7 error they
# have always returned is a shipped guarantee (encode.rs's own
# `encode_rejects_seed_tag` / `encode_rejects_xprv_tag` pin it).
edit("crates/ms-codec/src/encode.rs", [
    ("    // §3.5: payload length validation.",
     "    // SPEC_ms_hashlock §1 rule 2, emit side: never mint a single whose tag\n    // names a different kind than its prefix byte -- decode would refuse it.\n    // Placed AFTER the reserved-not-emitted check so `seed`/`xprv` keep the\n    // v0.1 SPEC §4 rule 7 error they have always returned.\n    if tag != payload.kind().single_tag() {\n        return Err(Error::TagKindMismatch {\n            tag: *tag.as_bytes(),\n            prefix: crate::envelope::prefix_of(payload),\n        });\n    }\n    // §3.5: payload length validation."),
])
edit("crates/ms-codec/src/inspect.rs", [
    ("    /// Any other prefix byte — future or invalid.\n    Unknown,",
     "    /// `hash` — a hashlock preimage (0x03 prefix byte, v0.8).\n    Preimage,\n    /// Any other prefix byte — future or invalid.\n    Unknown,"),
])

# ---- ms-cli -----------------------------------------------------------------
edit("crates/ms-cli/Cargo.toml", [
    ('version = "0.17.1"', 'version = "0.18.0"'),
    ('ms-codec = { path = "../ms-codec", version = "=0.7.0" }', 'ms-codec = { path = "../ms-codec", version = "=0.8.0" }'),
])
edit("crates/ms-cli/src/cmd/mod.rs", [
    ("pub mod gui_schema;", "pub mod gui_schema;\npub mod hashlock;"),
])
edit("crates/ms-cli/src/main.rs", [
    ("mod format;", "mod format;\nmod hashlock_phrase;"),
    ("    Decode(cmd::decode::DecodeArgs),",
     "    Decode(cmd::decode::DecodeArgs),\n\n    /// Derive a hashlock preimage from a phrase (or take one), print the `hash:` record, and back the preimage up as an ms1 plate string.\n    #[command(\n        after_long_help = \"EXAMPLES:\\n  ms hashlock --hashlock-phrase-stdin < phrase.txt\\n  ms hashlock --hashlock-phrase-stdin --method sha256 < phrase.txt\\n  ms hashlock --random --out preimage.txt\\n  ms hashlock --in preimage.txt\\n  ms hashlock --hashlock-phrase-stdin < phrase.txt | me sysw pack --out payload.bin\"\n    )]\n    Hashlock(cmd::hashlock::HashlockArgs),"),
    ("        Command::Decode(args) => cmd::decode::run(args),",
     "        Command::Decode(args) => cmd::decode::run(args),\n        Command::Hashlock(args) => cmd::hashlock::run(args),"),
    ("        Command::Decode(a) => a.json,", "        Command::Decode(a) => a.json,\n        Command::Hashlock(a) => a.json,"),
])
edit("crates/ms-cli/src/argv_guard.rs", [
    ('const SUBCOMMANDS: [&str; 12] = [\n    "derive",', 'const SUBCOMMANDS: [&str; 13] = [\n    "hashlock",\n    "derive",'),
    ('const SECRET_FLAGS: [&str; 4] = ["--phrase", "--hex", "--ms1", "--passphrase"];',
     'const SECRET_FLAGS: [&str; 5] = ["--phrase", "--hex", "--ms1", "--passphrase", "--hashlock-phrase"];'),
    ("            Some(\"encode\")\n                | Some(\"decode\")", "            Some(\"hashlock\")\n                | Some(\"encode\")\n                | Some(\"decode\")"),
    ('        "--ms1" => "an ms1 string",', '        "--ms1" => "an ms1 string",\n        "--hashlock-phrase" => "a hashlock phrase",'),
    ("    if is_ms1_shaped(candidate) {\n        return Some(\"an ms1 string (or one share of an ms1 share-set)\");",
     "    // ONE predicate for the ms1 shape, shared with the phrase channels: the\n    // normalisation is inside it (SPEC_ms_hashlock §4.3; R0 r0 tests C-1).\n    if looks_like_ms1(candidate) {\n        return Some(\"an ms1 string (or one share of an ms1 share-set)\");"),
    ("/// The nine flag-keyed secret channels, as strings. No parse, no clap.",
     "/// The five flag-keyed secret channels, as strings. No parse, no clap."),
    ("fn is_ms1_shaped(s: &str) -> bool {",
     "/// `is_ms1_shaped` over the NORMALISED token: trimmed, lowercased, display\n/// separators stripped. The one predicate both the argv guard and the phrase\n/// channels call, so the two cannot drift (SPEC_ms_hashlock §4.3). An\n/// uppercase plate string -- the BIP-173/QR spelling `ms decode` accepts --\n/// is caught here and only here.\npub(crate) fn looks_like_ms1(raw: &str) -> bool {\n    is_ms1_shaped(&raw.trim().to_ascii_lowercase())\n}\n\nfn is_ms1_shaped(s: &str) -> bool {"),
])
edit("crates/ms-cli/src/error.rs", [
    ("    BadInput(String),",
     "    BadInput(String),\n    /// A usage error the verb itself detects (source arithmetic, a gate a flag\n    /// must satisfy): exit 64, the same code clap uses for its own.\n    Usage(String),"),
    ("            | CliError::PayloadLengthMismatch { .. } => 1,",
     "            | CliError::PayloadLengthMismatch { .. } => 1,\n            CliError::Usage(_) => 64,"),
    ("            CliError::BadInput(_) => \"BadInput\",", "            CliError::BadInput(_) => \"BadInput\",\n            CliError::Usage(_) => \"Usage\","),
    ("            CliError::BadInput(m) => m.clone(),", "            CliError::BadInput(m) => m.clone(),\n            CliError::Usage(m) => m.clone(),"),
    # C-1 (R0 r0 fidelity): the three new codec errors get their own arms, or
    # they fall into the catch-all as `unhandled ms_codec::Error variant` at exit 1.
    ("            // ms_codec::Error is #[non_exhaustive]; v0.2+ may add variants.",
     "            ms_codec::Error::PreimageLengthMismatch { got } => CliError::FormatViolation {\n                underlying_kind: \"PreimageLengthMismatch\",\n                message: format!(\"preimage payload is {got} bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex characters)\"),\n                details: Some(json!({ \"got\": got })),\n            },\n            ms_codec::Error::TagKindMismatch { tag, prefix } => CliError::FormatViolation {\n                underlying_kind: \"TagKindMismatch\",\n                message: format!(\n                    \"the id {:?} names a different kind than the prefix byte 0x{prefix:02x} carries; refusing rather than reading one kind as another\",\n                    std::str::from_utf8(&tag).unwrap_or(\"<non-utf8>\")\n                ),\n                details: Some(json!({ \"tag\": std::str::from_utf8(&tag).unwrap_or(\"<non-utf8>\"), \"prefix\": prefix })),\n            },\n            ms_codec::Error::RandomnessUnavailable => CliError::BadInput(\n                \"the OS random source is unavailable; no preimage was produced\".to_string(),\n            ),\n            // ms_codec::Error is #[non_exhaustive]; v0.2+ may add variants."),
])
edit("crates/ms-cli/src/cmd/decode.rs", [
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };",
     "        // A preimage is rendered by `emit_preimage`, never as words, and the\n        // verb RETURNS here: the second match below (entropy extraction) is\n        // never reached for this kind and keeps its catch-all (SPEC_ms_hashlock §5).\n        Payload::Preimage(x) => return emit_preimage(x, args.json),\n        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };"),
])
edit("crates/ms-cli/src/out.rs", [
    ("pub(crate) fn write_artifact(", '/// Like `write_artifact`, but REFUSES an existing path (exit 64, naming it)\n/// instead of truncating. For `--random` only: that artifact is a function of\n/// nothing and cannot be re-made (SPEC_ms_hashlock §4.1).\npub(crate) fn write_artifact_create_new(path: &std::path::Path, body: &str) -> Result<()> {\n    use std::io::Write;\n    let mut opts = std::fs::OpenOptions::new();\n    opts.write(true).create_new(true);\n    #[cfg(unix)]\n    {\n        use std::os::unix::fs::OpenOptionsExt;\n        opts.mode(0o600);\n    }\n    // O_CREAT|O_EXCL: the check and the create are ONE syscall, so nothing can\n    // slip a file in between them and be truncated (R0 r0 fidelity I-4).\n    let mut f = match opts.open(path) {\n        Ok(f) => f,\n        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {\n            return Err(CliError::Usage(format!(\n                "--out {} already exists; a --random preimage will not overwrite it (choose another file, or move the old one first)",\n                path.display()\n            )));\n        }\n        Err(e) => {\n            return Err(CliError::BadInput(format!("failed to write --out {}: {}", path.display(), e)));\n        }\n    };\n    f.write_all(body.as_bytes())\n        .map_err(|e| CliError::BadInput(format!("failed to write --out {}: {}", path.display(), e)))?;\n    Ok(())\n}\n\npub(crate) fn write_artifact('),
])
# emit_preimage is appended to decode.rs as a new fn (see the Rust block below).
edit("crates/ms-cli/src/cmd/combine.rs", [
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"combine_shares returned an unknown Payload variant\"),\n    };",
     "        Payload::Preimage(x) => {\n            // A recovered preimage prints as `decode` does and never as words.\n            return crate::cmd::decode::emit_preimage(x, args.json);\n        }\n        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"combine_shares returned an unknown Payload variant\"),\n    };"),
])
# I-3 (R0 r0 fidelity): the fifth value-returning catch-all over the three
# kind types, unreachable through today's CLI (F-468) and swept anyway.
edit("crates/ms-cli/src/cmd/split.rs", [
    ('        PayloadKind::Entr => ("entr", None),', '        PayloadKind::Preimage => ("hash", None),\n        PayloadKind::Entr => ("entr", None),'),
])
edit("crates/ms-cli/src/cmd/payload_lang.rs", [
    (") -> (Zeroizing<Vec<u8>>, CliLanguage, bool) {\n    match payload {\n        Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted),",
     ") -> crate::error::Result<(Zeroizing<Vec<u8>>, CliLanguage, bool)> {\n    Ok(match payload {\n        // A preimage is not a seed: refuse HERE, before the catch-all, with\n        // the executable remedy (SPEC_ms_hashlock §5; review I-3).\n        Payload::Preimage(_) => {\n            return Err(crate::error::CliError::BadInput(\n                \"this is a hashlock preimage plate, not a seed backup; use `ms hashlock <ms1>` (or `ms hashlock --in FILE`) to re-derive its digest\".to_string(),\n            ))\n        }\n        Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted),"),
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    }\n}",
     "        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    })\n}"),
])
edit("crates/ms-cli/src/cmd/inspect.rs", [
    ("    if tag_bytes != TAG_ENTR {", "    if tag_bytes != TAG_ENTR && tag_bytes != TAG_HASH {"),
    ("        InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        _ => VALID_STR_LENGTHS,",
     "        InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        InspectKind::Preimage => VALID_PREIMAGE_STR_LENGTHS,\n        _ => VALID_STR_LENGTHS,"),
    ("        InspectKind::Mnem => {\n            // payload_bytes = [lang_byte, entropy...]; valid if len - 1 ∈ VALID_ENTR_LENGTHS.",
     "        InspectKind::Preimage => {\n            if report.payload_bytes.len() != 32 {\n                reasons.push(\"payload-length-mismatch\");\n            }\n        }\n        InspectKind::Mnem => {\n            // payload_bytes = [lang_byte, entropy...]; valid if len - 1 ∈ VALID_ENTR_LENGTHS."),
    # C-2 (R0 r0 fidelity): rule 6b, the tag/kind check, sits OUTSIDE the
    # per-kind arms so a `hash` id over a seed payload (or `entr` over a
    # preimage) is a reason on every kind -- `ms inspect` must never say
    # "would decode" for a string `ms decode` refuses.
    ("    // Rule 8: prefix byte must be a recognised kind (0x00 = entr, 0x02 = mnem).",
     "    // Rule 6b (SPEC_ms_hashlock §1 rule 2): a single's id must name the kind\n    // its prefix byte carries. Checked for EVERY recognised kind, not inside\n    // one arm, because the failure this guards is exactly the mismatch.\n    let expected_tag = match report.kind {\n        InspectKind::Entr | InspectKind::Mnem => Some(TAG_ENTR),\n        InspectKind::Preimage => Some(TAG_HASH),\n        _ => None,\n    };\n    if let Some(expected) = expected_tag {\n        if (tag_bytes == TAG_ENTR || tag_bytes == TAG_HASH) && tag_bytes != expected {\n            reasons.push(\"tag-kind-mismatch\");\n        }\n    }\n    // Rule 8: prefix byte must be a recognised kind (0x00 = entr, 0x02 = mnem)."),
    ("        \"unknown-tag\" => \"tag not in v0.1 RESERVED_TAG_TABLE\",\n        \"non-zero-prefix\" => \"prefix byte is not a recognised kind (0x00=entr, 0x02=mnem)\",",
     "        \"unknown-tag\" => \"tag not in the accept set (entr, hash)\",\n        \"tag-kind-mismatch\" => \"the id names a different kind than the prefix byte carries\",\n        \"non-zero-prefix\" => \"prefix byte is not a recognised kind (0x00=entr, 0x02=mnem, 0x03=preimage)\","),
    ("            InspectKind::Mnem => \"v0.2\",\n            _ => \"v0.1\",",
     "            InspectKind::Mnem => \"v0.2\",\n            InspectKind::Preimage => \"v0.8\",\n            _ => \"v0.1\","),
    ("            \"string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem)\"",
     "            \"string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem / [75] preimage)\""),
])
# emit_preimage, appended to decode.rs (Task 8).
_p = os.path.join(root, "crates/ms-cli/src/cmd/decode.rs")
_s = open(_p, encoding="utf-8").read()
if _s.count("#[cfg(test)]") != 1:
    sys.exit("decode.rs: expected exactly one #[cfg(test)] to insert emit_preimage before")
open(_p, "w", encoding="utf-8").write(_s.replace("#[cfg(test)]", '/// Render a preimage: kind, hex, digest. NEVER words -- a preimage is not\n/// entropy, and a 24-word rendering would be a seed nobody holds\n/// (SPEC_ms_hashlock §5).\npub(crate) fn emit_preimage(x: &[u8; 32], json: bool) -> crate::error::Result<u8> {\n    use std::io::Write;\n    let h = ms_codec::hashlock::digest(x);\n    let hx = hex::encode(x);\n    let hh = hex::encode(h);\n    let mut out = std::io::stdout().lock();\n    if json {\n        writeln!(out, "{}", serde_json::json!({"kind": "preimage", "preimage_hex": hx, "digest": hh})).ok();\n    } else {\n        writeln!(out, "kind:      preimage (hashlock, 32 bytes / 64 hex characters)").ok();\n        writeln!(out, "preimage:  {hx}").ok();\n        writeln!(out, "digest:    {hh}").ok();\n    }\n    drop(out);\n    let mut err = std::io::stderr().lock();\n    crate::advisory::emit_output_class_advisory(crate::advisory::OutputClass::PrivateKeyMaterial, &mut err);\n    Ok(0)\n}\n' + "\n#[cfg(test)]", 1))
print("  wired crates/ms-cli/src/cmd/decode.rs (emit_preimage inserted before the test module)")
edit("crates/ms-cli/src/cmd/inspect.rs", [
    ("use ms_codec::consts::{", "use ms_codec::consts::{TAG_HASH, VALID_PREIMAGE_STR_LENGTHS, "),
])
# I-2 (R0 r0 fidelity): the whole-range refusal loop must skip 0x03, which is
# no longer undefined, and pin what 0x03 does instead.
edit("crates/ms-codec/tests/forward_compat.rs", [
    ("        if prefix == 0x02 {", "        // 0x03 is the preimage kind now (SPEC_ms_hashlock §1): a 17-byte 0x03\n        // payload is refused by LENGTH, not by prefix -- hashlock_kind.rs's\n        // `preimage_prefix_is_refused_by_length_not_prefix` pins what it does.\n        if prefix == 0x02 || prefix == 0x03 {"),
])
edit("crates/ms-codec/src/inspect.rs", [
    ("use crate::consts::MNEM_PREFIX;", "use crate::consts::{MNEM_PREFIX, PREIMAGE_PREFIX};"),
    ("            InspectKind::Unknown => \"unknown\",", "            InspectKind::Preimage => \"preimage\",\n            InspectKind::Unknown => \"unknown\","),
    ("        _ => (InspectKind::Unknown, None),", "        PREIMAGE_PREFIX => (InspectKind::Preimage, None),\n        _ => (InspectKind::Unknown, None),"),
])
# verify.rs and derive.rs: the helper now returns Result, so each call gains `?`
# -- exact anchors, because one call is a match-arm expression (`),`) and the
# other a statement (`);`).
edit("crates/ms-cli/src/cmd/verify.rs", [
    ("            &mut stderr,\n        ),\n        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {",
     "            &mut stderr,\n        )?,\n        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {"),
])
edit("crates/ms-cli/src/cmd/derive.rs", [
    ("                    &mut stderr,\n                );\n            let m = Mnemonic::from_entropy_in(",
     "                    &mut stderr,\n                )?;\n            let m = Mnemonic::from_entropy_in("),
])
# payload_lang.rs's own unit tests destructure the helper's tuple; it returns
# Result now, so each of the five calls unwraps (test code only).
_p = os.path.join(root, "crates/ms-cli/src/cmd/payload_lang.rs")
_s = open(_p, encoding="utf-8").read()
if _s.count("            &mut buf,\n        );") != 5:
    sys.exit("payload_lang.rs: expected five unit-test calls to the helper")
_s = _s.replace("            &mut buf,\n        );", "            &mut buf,\n        )\n        .unwrap();", 5)
open(_p, "w", encoding="utf-8").write(_s)
print("  wired crates/ms-cli/src/cmd/payload_lang.rs (five test unwraps)")
# FORMAT THE WIRED COPY: the fragments above lengthen a few existing lines
# (the blocklist, two import lines, SECRET_FLAGS) past rustfmt's width, and a
# fragment kept fmt-clean by hand would drift the first time an anchor moved.
# The implementer runs `cargo fmt` after applying fragments for the same reason.
import subprocess
subprocess.run(["cargo", "fmt"], cwd=root, check=True)
print("  cargo fmt on the wired copy")
open(sentinel, "w").write("wired\n")
print("hand-wire complete")
