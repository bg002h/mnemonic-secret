You are an INDEPENDENT R0 reviewer of a SPEC, round 0, in the mnemonic-secret repository (`/scratch/code/shibboleth/mnemonic-secret`, master at `5ba61ca763804f89d27e4a551ba2117d5f2979db`). The artifact is `design/SPEC_ms_hashlock.md`. Its input is the brainstorm record `/scratch/code/shibboleth/mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md`, whose rulings L1-L23 are the OPERATOR'S and are not up for review -- a finding that a ruling is wrong is filed as a note for the operator, not as a spec defect. The two brainstorm review reports (`design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md` and `...-r2-security-software-expert.md`, in mnemonic-engrave) are already folded; do not re-raise their findings unless the spec fails to carry one.

Read-only. Copy the repo with `cp -r` if you need to run anything against a modified tree. Commit nothing. Do NOT spawn sub-agents. Read no `.jsonl` file.

## Already machine-checked by the controller -- spend your budget past these
- All fourteen §14 citations re-grepped at `7fc1e58` (= the tree this spec was written against; master has moved only by the spec commit itself).
- The four derivation values in §2 recomputed in `python3 hashlib` and cross-checked in `openssl kdf`.
- The entr-32 single's shape (`ms10entrsq...`, 75 characters) measured with the shipped `ms`.
- `grep -rn '_ => unreachable' crates/ms-cli/src` = 4, at the four cited lines.

## Severity
Critical = a wrong result, funds or preimage loss, an unmet guarantee, or a contradiction between two normative statements; Important = a real defect, a missing case, an unsound assumption, a ruling not carried; Minor/Nit = wording. Secret-handling defects (material reaching a log, argv, a 0644 file) are NEVER Critical or Important on this project (operator ruling 2026-08-27) -- record them as Minor with a reproduction. Number findings C-n / I-n / M-n / N-n. A finding you cannot make concrete is not a finding -- say what you tried. Do not pad.

## YOUR LENS: correctness and internal consistency

ONE QUESTION: is every normative statement in the spec true of the code it cites and consistent with every other statement in the spec -- and does the spec carry every ruling L1-L23 and every agreed section (4.1-4.6) of the brainstorm without loss or drift?

Do this, in order:
1. **Traceability table, both directions.** For each of L1-L23: the spec section that carries it, or NOT CARRIED. For each normative sentence in brainstorm 4.2, 4.3, 4.5 and 4.6: where it lands in the spec, or DROPPED. A dropped ruling is Important; a ruling the spec states DIFFERENTLY from the brainstorm is Important and you quote both.
2. **Internal contradictions.** Read the spec as a hostile implementer: two sentences that cannot both be implemented. Candidates to test: the phrase rule (§4.3) against the stdin reader; `--out` never suppressing stdout (§4.4) against `--random`'s refusal (§4.1); the method-line copy (§7) against the `--json` method key (§4.4); the "digest is not zeroized" rule (§2) against the `--json` advisory (§4.4); §5's verb table against §3's unreachable-site table.
3. **Code truth.** Every claim about the current tree that is NOT in the machine-checked list above: verify it. In particular the claim that `payload_lang.rs:61` is reached only from `verify` and `derive`; the claim that `ms encode --hex` stays entr; the claim that a K-of-N set recovers to a `0x03` payload is SUPPORTABLE by the codec's share path without a wire change; the claim that `read_stdin_passphrase` strips exactly one LF/CRLF and nothing else.
4. **Arithmetic and constants.** The length table (50/56/62/69/75 vs 51/58/64/70/77); the 75-character claim for 33 payload bytes; the "top five bits" claim for `q`; the 16..46 BIP-93 bracket.

Do NOT: review the cryptographic design (the adversarial lens owns it); assess test sufficiency (the tests lens owns it); re-derive the brainstorm's KDF-rate figures.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-hashlock-spec-R0-r0-correctness.md` (create; must not exist): the two traceability tables in full; findings numbered by severity, each with the spec line(s) and the contradicting source quoted; closing counts C/I/M/N. Return a two-line summary plus the path.
