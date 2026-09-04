You are an INDEPENDENT R0 reviewer of a SPEC, round 0, in the mnemonic-secret repository (`/scratch/code/shibboleth/mnemonic-secret`, master at `5ba61ca763804f89d27e4a551ba2117d5f2979db`). The artifact is `design/SPEC_ms_hashlock.md`. Its input is the brainstorm record `/scratch/code/shibboleth/mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md`, whose rulings L1-L23 are the OPERATOR'S and are not up for review -- a finding that a ruling is wrong is filed as a note for the operator, not as a spec defect. The two brainstorm review reports (`design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md` and `...-r2-security-software-expert.md`, in mnemonic-engrave) are already folded; do not re-raise their findings unless the spec fails to carry one.

Read-only. Copy the repo with `cp -r` if you need to run anything against a modified tree. Commit nothing. Do NOT spawn sub-agents. Read no `.jsonl` file.

## Already machine-checked by the controller -- spend your budget past these
- All fourteen §14 citations re-grepped at `7fc1e58` (= the tree this spec was written against; master has moved only by the spec commit itself).
- The four derivation values in §2 recomputed in `python3 hashlib` and cross-checked in `openssl kdf`.
- The entr-32 single's shape (`ms10entrsq...`, 75 characters) measured with the shipped `ms`.
- `grep -rn '_ => unreachable' crates/ms-cli/src` = 4, at the four cited lines.

## Severity
Critical = a wrong result, funds or preimage loss, an unmet guarantee, or a contradiction between two normative statements; Important = a real defect, a missing case, an unsound assumption, a ruling not carried; Minor/Nit = wording. Secret-handling defects (material reaching a log, argv, a 0644 file) are NEVER Critical or Important on this project (operator ruling 2026-08-27) -- record them as Minor with a reproduction. Number findings C-n / I-n / M-n / N-n. A finding you cannot make concrete is not a finding -- say what you tried. Do not pad.

## YOUR LENS: adversarial -- construct the loss

ONE QUESTION: under this spec exactly as written, construct a concrete sequence of operator actions that ends in (a) funds that cannot be spent, (b) a preimage that exists nowhere, (c) a preimage or phrase exposed to a party who should not have it, or (d) a plate that a reader -- human or machine -- takes for something it is not. A counterexample, not an assessment: name the commands, the inputs, and what the operator sees at each step.

Walk these journeys as the operator, and at every step ask what ELSE they might reasonably do:
1. **Phrase to plate to spend.** `ms hashlock --hashlock-phrase-stdin` with a phrase they typed; the card; `--out`; the plate; the `hash:` record into `me sysw pack`; the composer's `Which hash?`; a year later, the spend -- from the plate alone, from the phrase alone, from the phrase plus a lost method line. Where does the chain break?
2. **The wrong slot.** A preimage pasted as a phrase (64 hex); a plate string pasted as a phrase; a phrase pasted where a plate goes; the W-5 by-hand digest reproduced with the DEFAULT method. Does the spec's refusal actually catch each, and is the remedy it names executable?
3. **Polarity.** stdout public, stderr secret, `--out` secret, `--json` secret. Construct the shell line a careful operator writes that lands the preimage somewhere they did not intend -- then decide whether the spec's copy would have stopped them, and whether the loss is worse than saying nothing.
4. **The two methods.** Is there any state of the world in which the operator cannot tell which method made their digest? `--random`'s "nothing can be remembered" -- what does the operator hold if the plate is lost, and does the spec say so before the fact?
5. **The reader.** A 0.7 `ms`, a pre-H2 SH2, `me` 0.8.0, and the toolkit: what does each do with a `0x03` string, and does the spec's claim "the failure is a refusal and never a seed" hold for EACH? Verify against the code, not the spec's assertion.
6. **The KDF choice as a design.** L4/L5/L13 are rulings and stand. Within them: is anything in §2 exploitable that the brainstorm's two reviews did not already record (they recorded the shared-table cost and the brainwallet rate)? A new attack is Important; a restatement of a recorded one is not a finding.

Do NOT: audit citations (the correctness lens owns them); judge test coverage (the tests lens); propose replacing the KDF or adding a salt flag (ruled L4, L13).

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-hashlock-spec-R0-r0-adversarial.md` (create; must not exist): each journey with its steps and the divergence found (or "no divergence, and this is what I tried"); findings numbered by severity with the concrete sequence that produces the loss; the classification of each divergence as refusal / warning / default / not our concern / documentation only; closing counts. Return a two-line summary plus the path.
