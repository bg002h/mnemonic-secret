# SPEC_ms_hashlock — R0 round 1, FOLD VERIFICATION (sonnet, mechanical)

**Fold commit:** `1a14a4dc5f234d09840e33c68cdebc47078f4aee`, over draft `5ba61ca`.
**Scope:** `git diff 5ba61ca..1a14a4d -- design/SPEC_ms_hashlock.md` (852-line diff, 510→877 lines).
**Question:** did the fold address every Critical and Important from the three R0 r0 reports
(`d02185e` tests 1C/11I, `e6ef0a0` correctness 1C/7I, `4c59d8e` adversarial 4C/4I = 6C/22I), and
did it introduce a contradiction or a false claim of its own?
**Method:** read-only. No code was modified; no sub-agents. Three claims were re-executed
(brief item 4); all other verification is fold-text-against-report-text.

---

## 1. Finding table — 6 Critical / 22 Important

### Tests lens (`d02185e`)

| # | fix location (quoted) | verdict |
| --- | --- | --- |
| C-1 | §4.2: `pub(crate) fn looks_like_ms1(raw: &str)` … "trims, lowercases, strips display separators, and only then tests the HRP and charset — so the two cannot drift. **The normalisation is part of the predicate, not of its callers**" | **FIXED** — spot-checked (§3 below): today's raw `is_ms1_shaped` fails on uppercase; the proposed `looks_like_ms1` passes lowercase/UPPERCASE/grouped/space-padded all four |
| I-1 | §1 rule 2: "`TagKindMismatch { tag, prefix }`"; §8: "id/prefix disagreement, both directions, both refused with `TagKindMismatch`" | **FIXED** |
| I-2 | §11: "entr and mnem strings refused with the seed-backup wording and **entr-32 specifically** (the colliding length; tests I-2)"; §8 pairs `ms10entrsq…`/`ms10hashsq…` | **FIXED** |
| I-3 | §8: "the ms1-shaped refusal in four spellings … on both phrase channels (tests C-1, I-3)" | **FIXED** |
| I-4 | §8: "a phrase with a hyphen and a comma — `correct-horse,battery staple` … (tests I-4)" | **FIXED** |
| I-5 | §4.3: "Printable ASCII means bytes `0x20..=0x7E`, inclusive, and nothing else. TAB, DEL and every C0 control byte are refused…"; §8: "TAB and DEL refused, `0x20` and `0x7E` accepted" | **FIXED** |
| I-6 | §4.3: "The 64-hex guard uses the same hex predicate as `--hex`'s parser (`hex::FromHex`… which accepts both cases)"; §8: "the 64-hex refusal in lowercase AND uppercase" | **FIXED** |
| I-7 | §11: "stdout is exactly the record line … **under `--out` and under `--method sha256`** — the two configurations where a stdout-purity mutation can hide … (tests I-7)" | **FIXED** |
| I-8 | §11: "the warnings at their boundaries: hardened at 19 (warns) and 20 (does not), sha256 at 100 characters (still warns) (tests I-8)" | **FIXED** |
| I-9 | §8: "The salt string, the iteration count, the dkLen and every expected hex appear in the test as LITERALS, independent of the crate's constants, with one separate assertion that the constants equal the literals" | **FIXED** |
| I-10 | §8 lockstep rows: "the spaces row … the empty-phrase refusal; the printable-ASCII boundary; and the id/prefix-mismatch pair … The fork's pin test drives the vendored rows in **both directions**" | **FIXED** |
| I-11 | §3: "the `_ =>` catch-all is KEPT and stays `unreachable!` … A committed test asserts the count of `_ => unreachable` arms in `crates/ms-cli/src` equals the number this cycle leaves" | **FIXED** |

### Correctness lens (`e6ef0a0`)

| # | fix location (quoted) | verdict |
| --- | --- | --- |
| C-1 | §1 rule 1: "`hash` joins the single-string accept set. … Without this rule every preimage single is refused before any payload dispatch runs" | **FIXED** |
| I-1 | §1: "the codeword bracket is **16..44 payload bytes** … the wrong-length set that can reach `PreimageLengthMismatch` through `decode` is exactly `{17, 18, 21, 22, 25, 26, 29, 30, 34}`" | **FIXED** — independently re-derived, matches exactly (§3 below) |
| I-2 | §2: "`preimage_random` lives HERE, not in the CLI: ms-cli has no `getrandom` and no `rand` … while ms-codec already depends on `getrandom 0.3`" | **FIXED** |
| I-3 | §4.4: "one JSON object on stdout **in place of** the record line — the shipped shape (`encode.rs:218-230`, `decode.rs:123`…)" | **FIXED** |
| I-4 | §4.4: "for the two phrase sources only, omitted otherwise, with the same rule as `method` — `method` (…) and `phrase_chars`" | **FIXED** |
| I-5 | §4.1: "**L8, binding here and everywhere below:** every refusal and every help line that names the preimage's size says **'32 bytes (64 hex characters)'** — both spellings, always" | **FIXED** |
| I-6 | §8: byte-exact rows through BOTH channels (I-6.1); ms1-shaped refusal in 4 spellings on both channels (I-6.2, shared with tests I-3); "**The downgrade row**: a `0x03` single fed to ms-codec 0.7 is refused with `ReservedPrefixViolation` and never panics" (I-6.3) | **FIXED**, all three sub-items |
| I-7 | §9 item 3: "sweep **every catch-all** over `Payload`, `PayloadKind` and `InspectKind` — `_ => <value>` arms as much as `_ => unreachable!`"; §3: "ms-cli has **18** of those. Three sit inside `ms inspect`'s would-decode verdict" | **FIXED** |

### Adversarial lens (`4c59d8e`)

| # | fix location (quoted) | verdict |
| --- | --- | --- |
| C-1 | §4.1: "`--random` refuses (exit 64, naming `--out`) unless `--out FILE` is given. … `--json` no longer satisfies the gate" | **FIXED** (controller default pending operator veto, labeled as such) |
| C-2 | §4.1: "Under `--random`, `--out` refuses to overwrite (`create_new`; exit 64 naming the existing file)" | **FIXED** |
| C-3 | §9 rewritten: "The brainstorm's 'older readers refuse' premise is measured false, and §9 no longer claims it," + reader table + "**H0 — the prerequisite.** Before ms-cli 0.18.0 is released…" | **FIXED** (controller default reordering, labeled as such) |
| C-4 | §4.1: "`--hex` gets an unconditional warning of its own … 'The first spend of this hash path publishes these 32 bytes in the clear, forever…'" | **FIXED** |
| I-1 | §6: "5. The same binding for `--hex`… 6. And for the positional `<ms1>`… Three material channels, three bindings, three gates" | **FIXED** |
| I-2 | §4.4: "the record `me sysw pack` reads from stdin when given neither `--in` nor argv records … `--in -` is NOT a stdin sentinel there and exits 2"; §12.6 respelled | **FIXED** — spot-checked (§3 below), matches exactly |
| I-3 | §4.1: "**Zero sources is exit 64**, listing the five sources … It must not default to stdin" | **FIXED** |
| I-4 | §1: same rewrite as correctness I-1 (shared measurement) | **FIXED** |

**All 6 Critical and all 22 Important findings are FIXED in the text**, none PARTIAL, none DECLINED, none NOT FIXED.

---

## 2. Minors and Nits — one line each

**Tests M/N:** M-1 FIXED (`got` defined as post-prefix length) · M-2 FIXED (moot — `--json`-alone success path removed by C-1's fix) · M-3 FIXED (`--method` named for all three sources) · M-4 FIXED (all ten pairs, stdin-contention pair named) · M-5 FIXED (preflight exercises real pbkdf2 call) · M-6 FIXED ("every derivation row … reproduced externally") · M-7 FIXED (compile-time `Zeroizing` assertion) · M-8 FIXED (hex-case row) · M-9 FIXED (`beef` accepted row) · M-10 FIXED (both `--json` schema variants) · M-11 FIXED (`flag_class` test + three `/dev/null` gates named) · N-1 FIXED (§12.3 now "the corpus row pins") · N-2 FIXED ("never" carried into §11) · N-3 FIXED (instruction text pinned) · N-4 FIXED (doc-comment fix test line added).

**Correctness M/N:** M-1 FIXED (§14 now cites `consts.rs:17`/`:39` as definitions) · M-2 FIXED ("IS source-breaking … stated rather than called 'additive'") · M-3 FIXED (11-row negative-content matrix, superset of the original 9) · M-4 FIXED (both dropped refusal texts restored) · M-5 FIXED (build gate + publish dry run + both tags all named) · M-6 FIXED (`Vec<u8>`/`read_to_end` stated explicitly) · N-1 FIXED (Hamming-distance-of-full-codewords wording corrected) · **N-2 NOT FIXED** — §7 still reads "write this next to your phrase … if the method line is lost, try each method" with no statement that this instruction is suppressed under `--hex`/`--random`/`<ms1>`; the new `--hex` warning sentence is additive, not a suppression note (Nit, non-blocking).

**Adversarial M/N:** M-1 FIXED ("The file you just wrote is the only copy until you cut the plate") · M-2 FIXED (documented: argv channel never reaches the `--hex` remedy) · M-3 FIXED (`repair` row added to §5) · M-4 RECORDED, non-gating per the 2026-08-27 ruling — the cited text (§4.4 "this verb inverts `ms`'s usual polarity … lands a preimage in a `0644` file") is **pre-existing, untouched by this diff** (confirmed: zero hits for "0644"/"polarity" in the diff), so it was already on record before this fold · N-1 FIXED (shape-test-before-length-cap stated in §4.3 and a group-size-2 row added to §8).

---

## 3. Contradiction hunt (brief item 3) and the two named self-checks

Read §1, §4.1, §8, §9, §11, §12, §14 as a hostile implementer.

- **§8's length-row paragraph vs §1's reachable set:** identical — both state the door-by-door
  breakdown `{17,18,21,22,25,26,29,30,34}` via `decode`, `{16,32,44}` via `combine_shares` only,
  46 unconstructible. Independently re-derived below; matches exactly. **No contradiction.**
- **§11's "`--random` without `--out` exits 64 including with `--json`" vs §4.1 and §12.5:** all
  three state the same rule (`--out FILE` required regardless of `--json`; `--out FILE` succeeds
  and refuses to overwrite). **No contradiction.**
- **§6's six-part edit vs §11's three `/dev/null` gates:** §6 parts 4/5/6 name exactly
  `--hashlock-phrase`, `--hex`, the positional; §11 names the same three. **No contradiction.**
- **§9's H0 vs front matter and §12.7:** front matter, §9, §10 and §12 item 1/7 all agree H0
  (fork classifier + `me`'s `validate_record`) ships and is flashed **before** the 0.18.0 release,
  and acceptance is gated on it. **No contradiction.**
- **§14's citations vs the sections that use them:** spot-checked the sites that changed
  (accept-set `decode.rs:85-103`, length gate `decode.rs:46`, codeword bracket
  `codex32/mod.rs:198-201`, prefix-constant definitions `consts.rs:17`/`:39`) against the prose
  using them — consistent in every case checked.
- **A genuine, but self-corrected, false-looking grep:** the fold commit message claims "no
  `--in -`; no '16..46'" as machine-checks. Literal grep finds 3 hits for `--in -` and 1 for
  `16..46` in the current text — but all four are the fold correctly *describing the old, wrong
  claim as wrong* ("`--in -` is NOT a stdin sentinel", "the brainstorm's '16..46' was wrong"),
  never asserting either as true. Checked every `sysw pack` invocation in the spec (6 sites,
  line-numbered) — none uses the broken `--in -` spelling as a working example. Not a defect;
  worth flagging only because the commit message's own self-check phrasing is imprecise about
  what a bare grep would show.
- **The three controller-default labels** (§1 rule 2 TagKindMismatch, §4.1's L21 narrowing, §9's
  H0 reordering) are each explicitly marked in-text as a controller default pending the
  operator's word, per the brief's already-settled note. Not argued here.
- Line count 877 (claimed) = 877 (measured). Fourteen `## §` headings, §1 through §14, in order
  (measured). Both true.

**No new contradiction found.**

---

## 4. Spot-checks (brief item 4) — all three re-executed

**(a) Wrong-length set, `22+ceil(8N/5)` against the union set.** Independent Python
re-derivation over N = 2..46:

```
Reaches prefix dispatch (decode): [(17,50),(18,51),(21,56),(22,58),(25,62),(26,64),
                                    (29,69),(30,70),(33,75),(34,77)]
Refused earlier by UnexpectedStringLength (decode), includes: (16,48),(32,74),(44,93), ...
N=45 len: 94   N=46 len: 96   (both outside the 48..93 short-checksum bracket)
```

Removing the valid length (33), the wrong-length set reaching `PreimageLengthMismatch` through
`decode` is exactly `{17,18,21,22,25,26,29,30,34}` — **matches the spec's claim exactly**, and
16/32/44 refused earlier, 46 unconstructible — also exact matches.

**(b) `me sysw pack` reading stdin with no `--in`**, binary `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me` (confirmed `me 0.8.0`):

```
no --in, no positional -> stdin  : exit=0, container written
--in -                            : "me: -: No such file or directory (os error 2)", exit=2
positional -                     : exit=0, container written
```

**Matches the spec's claim exactly** (§4.4, §12.6, §14).

**(c) `is_ms1_shaped`'s case behaviour.** Transcribed `argv_guard.rs:134-145` (`BECH32_CHARSET`,
`MIN_MS1_LEN`, `is_ms1_shaped`) plus `format.rs:12-14,35-37` (`is_display_separator`,
`strip_display_separators`) verbatim into a standalone `rustc` binary, then added the fold's
*proposed* `looks_like_ms1` (trim + lowercase, then call `is_ms1_shaped`):

```
today's is_ms1_shaped alone:  lowercase -> true   UPPERCASE -> false   grouped(lc) -> true
proposed looks_like_ms1:      lowercase -> true   UPPERCASE -> true    grouped UPPER -> true
                               space-padded -> true
```

Confirms both halves: **the defect the fold cites as "today's" behaviour is real** (raw
`is_ms1_shaped` still fails on uppercase in the current tree), and **the fold's proposed fix
actually closes it** — folding normalisation into the predicate itself correctly classifies all
four spellings.

---

## 5. Closing counts

- Critical: 6/6 FIXED.
- Important: 22/22 FIXED.
- Minor: 21/22 FIXED, 1/22 RECORDED (adversarial M-4, non-gating, pre-existing text).
- Nit: 6/7 FIXED, 1/7 NOT FIXED (correctness N-2, wording gap, non-blocking).
- New contradictions introduced by the fold: **0**.
- False claims in the fold's own machine-checks: **0** (one self-check description is loosely
  worded but the underlying substance is correct, per §3 above).
- Spot-checks re-executed: 3/3, all confirm the spec's claims.

## GREEN

The fold addresses every one of the 28 Critical/Important findings in the text (not merely in
the commit message), the three re-executed spot-checks all confirm the spec's specific numeric
and behavioural claims, and no new contradiction was found across the seven rewritten sections
read as a hostile implementer. The one open item (correctness N-2, a Nit) does not gate.
