# cycle-prep recon — 2026-05-30 — mnem-wordlist-language-hint-on-wire

**Origin/master SHA at recon time:** `e3d5665` (mnemonic-secret)
**Local branch:** `master` (up-to-date, 0/0)
**Untracked:** this recon doc

Slug verified: `mnem-wordlist-language-hint-on-wire` (`design/FOLLOWUPS.md:340`). **All citations ACCURATE; the decisive finding is SCOPE — not a standalone fix.**

---

## Per-slug verification — `mnem-wordlist-language-hint-on-wire`
- **WHAT:** ms1 v0.1 doesn't carry the BIP-39 wordlist language on the wire → a non-English seed recovered via English-defaulted *third-party* software silently derives a different master seed → empty wallet. A `mnem` payload kind (entropy + language discriminant) makes the card self-describing.
- **Citations (all ACCURATE vs `e3d5665`):**
  - `crates/ms-codec/src/consts.rs:39` reserved tag — **ACCURATE**: `RESERVED_NOT_EMITTED_V01 = &[*b"seed", *b"xprv", *b"mnem", *b"prvk"]`.
  - `crates/ms-cli/src/cmd/decode.rs:43` "our decoder is NOT silent" — **ACCURATE**: `let (cli_lang, defaulted) = match args.language { Some(l) => (l, false), None => (CliLanguage::English, true) };` — `defaulted` is tracked + loud-annotated. The footgun is third-party software, NOT `ms decode`.
  - `design/SPEC_ms_v0_1.md:59` §6.3 hazard — **ACCURATE**: documents the exact "non-English user silently recovers the wrong mnemonic from English-defaulted wallet software" risk + allocates `mnem` for a "future v0.2+ entropy+wordlist-hint payload."
  - `design/SPEC_ms_v0_1.md:24-29` §1.3 v0.2 framing — **ACCURATE**: `seed`/`xprv`/`mnem`/`prvk` + K-of-N shares ALL deferred to v0.2 "with own framing" requiring the `0x00` prefix byte; the 64-B seed + prefix = 128 chars > BIP-93 long-code max 127 (a real wire constraint).
- **Action for brainstorm spec:** cite source SHA `e3d5665`. **The slug's own scope note is the headline:** `mnem` rides the v0.2 prefix-byte migration, NOT standalone.

---

## Cross-cutting observations
1. **SCOPE FINDING (decisive):** `mnem` is **not a small footgun fix.** It requires the v0.2 prefix-byte wire-format migration (`0x00`/`0x01` discriminator), which is the SAME framing that K-of-N share encoding (Theme D — the biggest unshipped ms capability) and the `seed`/`xprv`/`prvk` kinds all need. Doing `mnem` means doing (at least the entry point of) the ms v0.2 wire migration.
2. **Immediate risk already mitigated in-house:** `ms decode` loud-annotates "DEFAULT" when `--language` is omitted; the toolkit's `bundle`/`convert` paths inherit ms-codec. The unaddressed gap is purely cross-decoder (third-party wallets) — exactly what a self-describing wire fixes, but only via v0.2.
3. **Two consecutive cycle-prep saves:** the prior pick (`bip85` no-op flags) was already-resolved (v0.8); this pick is a big-cycle-in-disguise. Both were mis-sized by the feature survey.

---

## Recommended brainstorm-session scope
**`mnem` standalone is not feasible.** Three honest paths:

- **(A) Small footgun-flavored win NOW (advisory-only, no wire change):** a toolkit/ms-cli **stderr advisory** when a non-English seed is `encode`d / `bundle`d without the language being durably recorded — "record your wordlist language alongside the engraved card; ms1 v0.1 does not carry it." Pure behavior, no wire/format change. S. Mitigates the third-party hazard via operator guidance. (Could also bold the §6.3 manual note.) Closest to the "footgun fix" the pick implied.
- **(B) The real fix — ms v0.2 prefix-byte migration (BIG, format-defining):** introduce the `0x00`/`0x01` discriminator framing, then `mnem` (entropy + language) as the first new kind. Overlaps Theme D (K-of-N shares ride the same framing). L; cross-repo (ms-codec MINOR wire + ms-cli + toolkit consumers + manual). This is a major multi-cycle arc, not a quick footgun patch.
- **(C) Pivot to a genuinely-small open item:** `slip39-cli-extendable-flag` (S, toolkit-only-ish, GUI+manual lockstep) — the cleanest small cycle on the board.

**Recommendation:** if the goal is a sharp footgun mitigation now, do **(A)** the advisory (small, honest, ships value without the v0.2 commitment) and file/keep `mnem` under the v0.2 arc. If the goal is the format-defining capability, scope **(B)** deliberately as the ms-v0.2 cycle (with K-of-N shares). Mandatory opus R0 on whichever.
