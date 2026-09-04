You are an INDEPENDENT R0 reviewer of a SPEC, round 0, in the mnemonic-secret repository (`/scratch/code/shibboleth/mnemonic-secret`, master at `5ba61ca763804f89d27e4a551ba2117d5f2979db`). The artifact is `design/SPEC_ms_hashlock.md`. Its input is the brainstorm record `/scratch/code/shibboleth/mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md`, whose rulings L1-L23 are the OPERATOR'S and are not up for review -- a finding that a ruling is wrong is filed as a note for the operator, not as a spec defect. The two brainstorm review reports (`design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md` and `...-r2-security-software-expert.md`, in mnemonic-engrave) are already folded; do not re-raise their findings unless the spec fails to carry one.

Read-only. Copy the repo with `cp -r` if you need to run anything against a modified tree. Commit nothing. Do NOT spawn sub-agents. Read no `.jsonl` file.

## Already machine-checked by the controller -- spend your budget past these
- All fourteen §14 citations re-grepped at `7fc1e58` (= the tree this spec was written against; master has moved only by the spec commit itself).
- The four derivation values in §2 recomputed in `python3 hashlib` and cross-checked in `openssl kdf`.
- The entr-32 single's shape (`ms10entrsq...`, 75 characters) measured with the shipped `ms`.
- `grep -rn '_ => unreachable' crates/ms-cli/src` = 4, at the four cited lines.

## Severity
Critical = a wrong result, funds or preimage loss, an unmet guarantee, or a contradiction between two normative statements; Important = a real defect, a missing case, an unsound assumption, a ruling not carried; Minor/Nit = wording. Secret-handling defects (material reaching a log, argv, a 0644 file) are NEVER Critical or Important on this project (operator ruling 2026-08-27) -- record them as Minor with a reproduction. Number findings C-n / I-n / M-n / N-n. A finding you cannot make concrete is not a finding -- say what you tried. Do not pad.

## YOUR LENS: tests and vectors -- can they fail?

ONE QUESTION: for each guarantee the spec makes, name the mutation that would break it, and then say whether §8's vector rows and §11's tests as written would CATCH that mutation. A guarantee whose named mutation passes every listed test is a finding.

Do this:
1. **Build the mutation table first**, from the spec alone, before judging the tests: wrong salt (one byte); iterations 10,000 or 1,000,000; methods swapped; dkLen 16; `digest` computing sha256 of the phrase instead of the preimage; the byte-verbatim reader stripping TWO trailing newlines, or a leading space; `HASHLOCK_PHRASE_MAX_CHARS` at 99 or 101; the 64-hex refusal testing on 63; `is_ms1_shaped` not called on the stdin channel; `PREIMAGE_PREFIX` = 0x01; length check after the `try_from`; the id `hash` written to the blocklist but not used on singles (or the reverse); `--random` gating on `--out` only; `--out` suppressing stdout; `--method` silently ignored with `--hex`; each of the four `unreachable!` arms left in place; the `derive`/`verify` refusal placed AFTER `payload_entropy_and_language`. Add your own.
2. **For each mutation: which spec'd row or test fails, quoted.** If none, that is a finding, Important if the mutation changes X or H or loses a preimage, Minor otherwise.
3. **The reproduction test.** §8 says a test runs `python3` and `openssl kdf` and FAILS if either is absent, with a CI preflight step. Design the false-PASS: how could that test print ok while proving nothing? A skip, a cached value, a stub binary on PATH, a test that compares the tool against itself.
4. **Vector sufficiency for the Go port (H2).** The rows are meant to be vendored into the fork and to catch a behaviour-faithful port that drifts. Which drift would the listed rows NOT see? (The device shares no code with the host, so only rows can catch it.)
5. **The census question.** The spec pins several structural counts (four unreachable arms, five blocklist ids, twelve subcommands, four secret flags). Which of these should become a committed assertion so a later change cannot silently move them, and which are fine as prose?

Do NOT: audit citations (correctness lens); construct funds-loss journeys (adversarial lens); propose changing the rulings.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-hashlock-spec-R0-r0-tests.md` (create; must not exist): the mutation table with a CAUGHT BY / NOT CAUGHT column; findings numbered by severity; the false-PASS designs for item 3 and whether §8's wording already forbids each; closing counts. Return a two-line summary plus the path.
