s1: 0C/0I/1M/2N

# Whole-diff review s1 (executed advice, design, threat): ms bug-hunt fixes + toolkit manual companion

Reviewer: Opus, slice s1 of 2. Brief: `coordinator/briefs/bughunt-ms-wholediff-s1.md`.
Under review: ms `origin/master...7b6169b` (merge-base `85efbd4`); toolkit `c2b09861..451ca5f8`; against `design/FIX_bughunt_ms.md` (frozen `e67eb6a`).
Read-only. Builds (cargo 1.85.0, `~/.cargo/bin` first, `--locked`, dev profile): ms at `7b6169b` in scratch worktree `/scratch/code/shibboleth/wd-s1-ms` (target `/scratch/cache/bughunt/wd-s1-ms`); toolkit `mnemonic` at `c2b09861` in `/scratch/code/shibboleth/wd-s1-tk` (target `/scratch/cache/bughunt/wd-s1-tk`). Both worktrees removed after the run; target dirs kept. Probe scripts and raw results: `/scratch/cache/bughunt/wd-s1-work/` (`scripts/`, `r1-results.txt`, `r2-results.txt`, `pos-results.txt`).

## Answer to the one question

Yes, the build does what the FIX decides, and no changed text or output gives a wrong secret, a wrong card, a wrong verdict or a printed command that fails as printed. Across about 2,900 executed runs of the changed manual text's commands, on every card kind it reaches, no run printed a wrong string. Every refusal was the one the text predicts. One Minor and two Nits are below, none of them in a funds path.

## 1. Build vs FIX (sections 2-5, 9)

- Production code matches FIX section 3 point by point. `shares.rs` ZERO returns `encode::encode(tag, secret)?`. `parse_ms1_symbols` reads `&s[3..]`; membership is `is_ascii` plus `to_ascii_lowercase` in the alphabet; the error is `InvalidChar(c)` as typed; the HRP check is unchanged; the comment is FIX 2.4's text. In `combine.rs`, `--to phrase` returns early to `emit_preimage`; any other `--to` falls through with `"preimage"`; `emit_ms1` uses `payload.kind().single_tag()`; the warning is text-mode only, after stdout and before the advisory. In `inspect.rs`, `is_secret_share` drives both share emitters, and T2 pushes `mnem-unknown-language` before `payload-length-mismatch`; both `reason_text` arms are verbatim.
- `&s[3..]` cannot panic: `lower` is `to_ascii_lowercase` (byte-length preserving), so `lower.starts_with("ms1")` means bytes 0-2 of `s` are ASCII.
- T2's predicate (`MNEM_LANGUAGE_NAMES.get(code)` is None, so code >= 10) equals `Payload::validate`'s `*language >= 10` (`payload.rs:95`). Measured on codes 9, 10, 15, 0x10 and 0xff, lower and upper case: `ms inspect` FAILs exactly when `ms decode` refuses.
- Spec replacements (FIX 4) are verbatim in the diff (script). Toolkit: all 10 FIX 5.x New texts occur once in `451ca5f8` (script). FOLLOWUPS: the 9.1, 9.2, 9.3 and 9.4 blocks are verbatim. 9.5 differs from the FIX by one added bullet, "Journey input (Brian 2026-11-01 …)", plus the extra 9.2 entry `primer-lowercase-rule-after-subst-hint`. Commit `7b6169b` names both as coordinator additions from the build brief. Not a finding.

## 2. Executed advice lens and run what you print

Tools: built `ms` (this branch) and toolkit `mnemonic` at `c2b09861`. Signet/test material:
- E16 `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s`;
- E32 (32-byte entr, 75 characters);
- M17 (Japanese mnem, 51 characters);
- M33 (Japanese mnem, 24 words, 77 characters);
- H, the hashlock plate `ms10hash…` (preimage 0x11 × 32);
- SH, a 2-of-3 share from `ms split`;
- MK1, the 2-string mk1 of the `abandon … about` bip84 signet bundle;
- MD1, its 3-string chunked md1;
- MD1s, `md1yqpqqxqq8xtwhw4xwn4qh`.

Each case ran in lowercase and in capitals, unbroken and grouped in fives (as the engraving card prints). Damage was random, in the data part, and in a random string of a multi-string card. "OK" means the original string is in stdout.

Primer 5.6.6, way 1 (leave out; `mnemonic repair --<ms1|mk1|md1> - --max-indel N [--max-subst 1]`), 756 runs:

| card | lowercase, N = 1..4 left out | lowercase, N + 1 wrong, `--max-subst 1` | capitals |
|---|---|---|---|
| E16, E32, M17, M33 | exit 5, OK | exit 4, OK | exit 2 (the text says to type it in lowercase) |
| MK1, MD1 (every string in the file) | exit 5, OK | exit 4, OK | exit 2 |
| H, MD1s, SH | exit 2 (the text says it does not work) | exit 2 | exit 2 |

Larger budgets: `--max-subst 2` and `3` (1 + 2, 1 + 3 and 2 + 2 damage) on E16, M33, MK1 and MD1 gave 48/48 exit 4, OK, one candidate each. Every position: one character left out at each position of E16, M17, E32, MK1 (both strings) and MD1 (all three) gave exit 5, OK. That covers all positions, including the HRP and the first and last characters. H and MD1s gave exit 2 at every position.

Way 2 (best guess; `ms repair --in card.txt` for ms1, `mnemonic repair --mk1|--md1 - < card.txt`), 756 runs, plus a guess at every data position:

| card | 0 wrong | 1..4 wrong | 5 wrong |
|---|---|---|---|
| E16, E32, M17, M33, H (lower, CAPS, grouped) | exit 0 | exit 4, OK | exit 2 |
| MK1, MD1 | exit 0 | exit 5, OK | exit 2 |
| MD1s | exit 0 | exit 4, OK | exit 2 |
| SH (lower, CAPS) | exit 2 | exit 2 | exit 2 |

This includes a wrong guess at the threshold digit (position 3) of an ms1 single, which was corrected (exit 4) rather than refused as a share.

Other claims, executed:
- `?` is refused by `ms repair` (exit 1, `invalid character '?'`) and by `mnemonic repair --ms1|--mk1|--md1` (exit 2) on E16, H, MK1, MD1 and MD1s.
- `ms repair` and `mnemonic repair --ms1 -` refuse a share, lower and CAPS, exit 2.
- `ms combine --in` with K clean grouped shares recovers the right secret: E16 2-of-3 lower and CAPS, and the M33 3-of-5 (Japanese), exit 0.

43-ms.md 5.1 and the combine warning:
- `ms combine --in preimage-shares.txt` with default `--to` and with `--to phrase` gives text and `--json` byte-identical (`cmp`) to `ms decode` of the hash plate.
- `--to entropy` prints 64 hex characters and the warning line (byte-equal to FIX 2.2).
- `--to ms1` is `cmp`-identical to `ms hashlock --kind sha256 --hex - --out`. `ms decode --in` and `ms hashlock --in` read it, with or without `--kind` (exit 0).
- `--json` gives the FIX objects with no warning.
- The entr and mnem `--to ms1` and `--to entropy` outputs equal the unfixed installed `ms`.
- The warning's premise holds: `ms encode --hex -` and `ms derive --hex -` accept the 64 hex as a seed.

43-ms.md 5.2 and 5.5: the s-index share at k=2 and k=9, lower and CAPS, gives `FAIL: would NOT combine` with `share-index-secret`, `--json` false/false, exit 0, and a k=2 index-`q` control is unchanged. The `ms combine` refusal named in the reason text was confirmed. Unknown-language mnem prints `language: unknown` and FAIL, exit 0. Code 9 is unchanged (OK, `portuguese`, and `ms decode` reads it).

#7 (FIX 2.4 after-fix block): every command reproduces byte for byte (b, B, ?, ĳ, text and `--json`, exit 1). Extra probes: a Kelvin sign U+212A and a long s U+017F are refused as InvalidChar, exit 1. A full-width HRP is refused as WrongHrp, exit 2. Mixed case `Ms1…` is refused as mixed case, exit 1, unchanged.

## 3. Create-input table (commands the changed text prints or names)

| command (as printed) | required inputs per the real CLI | given by the text? | ran |
|---|---|---|---|
| `mnemonic repair --ms1 - --max-indel 1 < card.txt` | one of `--ms1/--mk1/--md1/STRING`; stdin for `-` | yes | yes |
| same with `--mk1 -` / `--md1 -`; with `--max-subst 1` added | same | yes | yes |
| `ms repair --in card.txt` | one of `--ms1`/`--in` | yes | yes |
| `mnemonic repair --mk1 - < card.txt` (`--md1 -`) | as above | yes | yes |
| `ms combine` ("recover … with `ms combine`", prose) | `--in FILE` or positional/`-` | prose only, no full command | yes, `--in` |
| `ms combine --to phrase/entropy/ms1 [--json]` (43-ms.md row) | `--in` or shares | the synopsis gives `--in <FILE>` | yes |
| `ms decode`, `ms hashlock --in` (read the `--to ms1` string) | `--in FILE` | yes (`--kind` optional) | yes |
| `ms encode` / `ms split` / `ms derive --hex` (named as don'ts) | `--hex` / `--hex` with `-k -n` | n/a (prohibitions) | encode and derive accept the hex, as warned |

## 4. Design and threat (the four production changes)

Each path now accepts the same as before or less, or emits only what FIX decides:
- **ZERO threshold (#1):** strictly more refusals. No constellation caller passes ZERO (the toolkit calls with `Threshold::new(k)`, FIX 8.9).
- **Non-alphabet (#7/#3):** the accepted set strictly shrinks. ASCII handling is identical (per-char `to_ascii_lowercase` equals the old `lower[3..]`). Non-ASCII, which `c as u8` used to truncate (U+0133 read as '3' and silently "repaired"), is now refused. No input that decoded or repaired before now decodes to anything different. The only caller is `decode_with_correction`. The toolkit is unaffected until its ms-codec pin moves (filed, 9.4).
- **Preimage via `combine --to` (#4):** the new emissions are the preimage hex (already printed before, inside the record) and the `hash` plate string. The plate string is byte-identical to `ms hashlock --out`, so a wrong card cannot be minted. Seed-kind `--to ms1` still gets `Tag::ENTR` (`single_tag`), byte-identical to before. The preimage-as-seed path is the FIX's decided text-mode warning. No constellation consumer reads `ms combine`'s JSON `entropy_hex` (grep of `mnemonic-gui/src`: none).
- **Inspect of the secret share (#5) and T2:** output-only verdict changes. No mint and no refusal changed. `would_decode: false` has no consumer in `mnemonic-gui`.

No mutation-gate survivors (30/30 caught), so there is nothing to grade on design or threat grounds.

## 5. External facts (BIP-93, `/scratch/cache/bughunt/ms-grade/bip-0093.upstream.mediawiki`)

- Primer 5.6.1–5.6.3 ("4 substitutions, 8 erasures, 13 consecutive erasures" for the regular code) match BIP-93 l.152 and l.413.
- 5.6.5's "BIP-93 suggests marking one with `?`" matches l.158 ("Implementations interpret "?" as an erasure", a recommendation).
- The long-code 15 is already filed (`toolkit-primer-long-code-erasures-15`); not re-reported.

## Minor

**M-1: way 1 drops every line after the first of a one-string `ms1` card typed over two lines; way 2 joins them.** Measured on a clean E16 typed as two rows of five groups:
- `ms repair --in card.txt` exits 0 and prints the right string.
- `mnemonic repair --ms1 - < card.txt` exits 2 ("too many errors … singleton bound").
- With one character left out, `--max-indel 1` and `--max-indel 1 --max-subst 1` both exit 2, while `ms repair --in` says "more than 8 errors", exit 2.

`--ms1 -` reads one chunk, so the second line is silently ignored. This fails closed: no wrong string, and the printed command works on a card typed as the text says ("its one `ms1` string, … one per line"). Likelihood-test conditions: (a) the holder types an engraved `ms1` string over two lines because the plate wraps, against "one per line"; (b) the holder takes way 1. The same holds for a wrapped `mk1` or `md1` string, which becomes an extra chunk. Fix (for later, FOLLOWUPS `deferred-minor`): in 5.6.6, after "one per line (not only the damaged one)", add "; type each string on a single line even where the plate wraps it". Optionally, `mnemonic repair --ms1 -` could refuse a second non-empty stdin line rather than ignore it.

## Nits

**N-1:** the primer newly sends a damaged hashlock plate to `ms repair` ("use the second way for those"). `ms repair`'s unchanged exit-4 stderr then says "confirm the derived address/xpub against a known-good copy". A preimage has neither, and `ms derive` refuses it. The primer's own "compare the result with the plate" is the usable check, and a wrong preimage cannot spend, so no funds are at risk. Fix (later): a preimage-kind variant of the UNVERIFIED line, for example "confirm with `ms hashlock --in` that its digest matches the hash in your wallet policy".

**N-2:** this extends the filed `ms-invalid-char-message-invisible-char`; it is not a re-report. A printable non-ASCII lookalike is named as typed, so U+212A KELVIN SIGN prints `invalid character 'K'`, which reads as an ASCII letter whose lowercase is in the alphabet. It fails closed (exit 1). Fix (later): the filed item's escaped form should cover any non-ASCII character, not only non-printing ones.

## Design lines (ungraded)

- `ms decode` and `ms repair` on an s-index share (threshold 2-9) still advise "use `ms combine` to recombine K shares", which `ms combine` refuses. That is pre-existing text, and it now contradicts the new `inspect` reason for the same string.
- The companion manual documents ms 0.20.2 behaviour, but the toolkit's install pin stays `ms-cli-v0.20.1` until the release commit (FIX 8, "Release"). Merging the companion with this PR, before that release and pin move, publishes `--to entropy`/`--to ms1` and inspect FAIL text that the installed `ms` does not yet produce. Every divergence fails safe (the record is printed, or OK is shown where FAIL is documented). Merging the companion after the pin move avoids it.
- `mnemonic repair --mk1 - --max-indel 4` on the 111-character mk1 string took 75 s here (dev, opt-level 2), against its own "this may take a few seconds" note. This is pre-existing tool text.
