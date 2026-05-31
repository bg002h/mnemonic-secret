# End-of-cycle R1 — ms-cli 0.5.0 ms derive

Reviewer: opus. After R0 1C/2I/3M fold.

- C1 CONFIRMED: parse.rs:54 read_stdin_passphrase strips only trailing \r?\n (preserves internal/leading/trailing spaces+tabs); derive.rs:194 uses it for --passphrase-stdin; read_input no longer on any passphrase path; regression test stdin "a b c" == inline "a b c" != no-passphrase.
- I1 CONFIRMED: 43-ms.md ## ms derive section (all flags) + cli-subcommands.list + "Seven subcommands".
- I2 CONFIRMED: gui ms.rs derive (--hex/--phrase/--passphrase secret:true, dropdowns) + repair backfill + pin 0.5.0 (master bf13f00).
- Minors M1/M2/M3 non-issues. Drift sweep clean: no borrow issue; single-stdin guard fires before passphrase read; no-secret-on-stdout intact; oracle fp/xpub unchanged; release hygiene intact.

VERDICT: GREEN (0C/0I) — clear to tag/ship.
