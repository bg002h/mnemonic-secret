//! `ms hashlock --kind`: the operator surface for the three new hash kinds
//! (SPEC_hashlock_kinds §6, §13.4).
//!
//! WHY THIS FILE EXISTS. The flag shipped once with ZERO coverage and two
//! Criticals rode in behind a 568-green suite: the card told the operator to
//! compose a `sha256=` operand out of a `ripemd160` digest, and the no-`--kind`
//! path silently assumed sha256 while its own `--help` claimed otherwise. A
//! green suite is only evidence about what it tests.

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";

fn run(args: &[&str]) -> (String, String) {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// §6's producer rule: BARE for sha256, `hash:<kind>:<hex>` for the rest, hex
/// at `digest_len()*2`. A bare record MEANS sha256, so emitting one under
/// another kind hands `me sysw pack` a digest it reads as sha256 — the scheme
/// §6 names as rejected because it composes an unspendable wallet.
#[test]
fn the_record_follows_the_producer_rule_for_every_kind() {
    let (so, _) = run(&["hashlock", "--hashlock-phrase-stdin"]);
    assert!(
        so.trim().starts_with("hash:") && so.trim().matches(':').count() == 1,
        "no --kind must still emit the BARE sha256 record: {so:?}"
    );
    assert_eq!(so.trim().len(), "hash:".len() + 64, "bare record is 64 hex");

    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let (so, _) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        let rec = so.trim();
        if kind == "sha256" {
            assert!(
                !rec.starts_with("hash:sha256:"),
                "--kind sha256 must emit the BARE form, for byte-identical compatibility: {rec:?}"
            );
            assert_eq!(rec.len(), "hash:".len() + hexlen);
        } else {
            let want = format!("hash:{kind}:");
            assert!(rec.starts_with(&want), "{kind}: want {want:?}, got {rec:?}");
            assert_eq!(rec.len(), want.len() + hexlen, "{kind}: hex length");
        }
    }
}

/// §11: this line is the ONLY thing keeping the digest function and `md
/// compose`'s option name in agreement. Under `--kind ripemd160` it must say
/// `ripemd160=`, or it instructs the operator to build a wallet whose hashlock
/// nobody can satisfy.
#[test]
fn the_md_compose_line_names_the_chosen_kind() {
    for kind in ["hash256", "ripemd160", "hash160"] {
        let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        let line = se
            .lines()
            .find(|l| l.contains("for md compose:"))
            .unwrap_or_else(|| panic!("{kind}: no `for md compose:` line on the card"));
        assert!(
            line.contains(&format!("{kind}=")),
            "{kind}: the card says {line:?} — an operator following it composes the wrong wallet"
        );
        assert!(
            !line.contains("sha256="),
            "{kind}: the card still offers a sha256= operand: {line:?}"
        );
    }
}

/// §13.4: never silently assume sha256. A plate cut before this existed carries
/// no kind, so all four are listed and the operator matches.
///
/// BOTH argv shapes, because the first version of this test checked only the
/// default one and the fallback had been nested inside the `--no-engraving-card`
/// guard — so the flag that suppresses the card also suppressed the §13.4
/// notice, and this test passed anyway (R0 round 4, C-1). §13.4's clause is
/// unconditional; the operator who hides the card is the one who most needs the
/// lookup, because the card is what carries the preimage.
#[test]
fn without_a_kind_every_digest_is_listed_on_stderr() {
    for argv in [
        &["hashlock", "--hashlock-phrase-stdin"][..],
        &["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"][..],
    ] {
        let (so, se) = run(argv);
        for (kind, hexlen) in [
            ("sha256", 64),
            ("hash256", 64),
            ("ripemd160", 40),
            ("hash160", 40),
        ] {
            // The fallback's own line shape: `  <token padded to 10> <hex>`.
            // Matching the DIGEST and its WIDTH, not just the word: stderr
            // already says "sha256" twice without the loop running at all --
            // on the `for md compose:` line and in this block's own header
            // ("stdout carries the sha256 record") -- so a bare `contains`
            // would report coverage of a sha256 row that was never printed.
            // Width is what separates the 20-byte kinds from the 32-byte ones.
            let found = se.lines().any(|l| {
                let Some(rest) = l.strip_prefix("  ") else {
                    return false;
                };
                let Some(rest) = rest.strip_prefix(kind) else {
                    return false;
                };
                let hexpart = rest.trim_start();
                hexpart.len() == hexlen && hexpart.bytes().all(|b| b.is_ascii_hexdigit())
            });
            assert!(
                found,
                "{argv:?}: stderr carries no {kind} digest line; §13.4 forbids \
                 assuming sha256 in silence.\n{se}"
            );
        }
        assert!(
            !so.contains("hash256"),
            "the fallback must not reach stdout — purity contract: {so:?}"
        );
    }
}

/// Case is rejected, never folded (§6).
#[test]
fn an_uppercase_kind_is_refused() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["hashlock", "--hashlock-phrase-stdin", "--kind", "RIPEMD160"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "RIPEMD160 must be refused, not folded"
    );
}

/// §13.4 on the MACHINE channel. The stderr listing cannot be used under
/// `--json`: `--json --no-engraving-card` pins stderr to exactly the
/// PrivateKeyMaterial advisory (§4.4, §11), a contract `hashlock_outputs.rs`
/// enforces. So the notice travels in the object instead — and this test is
/// what stops that from being a silent drop rather than a change of channel.
#[test]
fn under_json_the_object_carries_every_kind_when_none_was_named() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--no-engraving-card",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();

    assert_eq!(
        v["kind_specified"], false,
        "no --kind was given; the object must say so rather than let \
         `hash_operand`'s sha256 default read as a stated choice"
    );
    let by = v["digests_by_kind"]
        .as_object()
        .expect("digests_by_kind must be an object");
    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let d = by[kind]
            .as_str()
            .unwrap_or_else(|| panic!("no digests_by_kind.{kind}"));
        assert_eq!(d.len(), hexlen, "{kind}: digest width");
        assert!(d.bytes().all(|b| b.is_ascii_hexdigit()), "{kind}: not hex");
    }
    assert_eq!(by["sha256"], v["digest"], "sha256 row must match `digest`");

    // The purity contract the stderr listing was moved OUT of the way for.
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("no --kind given"),
        "the human listing must not reach stderr under --json"
    );

    // With an explicit kind there is nothing to disambiguate.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--kind",
            "ripemd160",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["kind"], "ripemd160");
    assert!(v.get("digests_by_kind").is_none(), "{v}");
    assert!(v.get("kind_specified").is_none(), "{v}");
}

/// The cell between the other two: `--json` WITHOUT `--no-engraving-card`.
///
/// This is the normal way to use `--json` — `ms hashlock … --json > out.json` —
/// and it is the one the stderr purity contract does NOT cover, because that
/// contract is on the two flags TOGETHER. The card still prints to the
/// terminal here, so the human is reading stderr, and the `for md compose:`
/// line they are reading says `sha256=`. Without the §13.4 listing beside it
/// they are shown one unlabelled sha256 operand and nothing saying a kind was
/// never chosen — while the machine-readable notice sits in the redirected
/// file they are not looking at.
///
/// The guard was `!args.json` for one round and this cell was untested, which
/// is how it got through (R0 round 5, I-1).
#[test]
fn under_json_with_the_card_the_human_still_gets_the_listing() {
    let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--json"]);
    assert!(
        se.contains("for md compose:"),
        "precondition: the card must still be on stderr here, or this test is \
         asserting nothing:\n{se}"
    );
    for (kind, hexlen) in [
        ("sha256", 64),
        ("hash256", 64),
        ("ripemd160", 40),
        ("hash160", 40),
    ] {
        let found = se.lines().any(|l| {
            let Some(rest) = l.strip_prefix("  ") else {
                return false;
            };
            let Some(rest) = rest.strip_prefix(kind) else {
                return false;
            };
            let hexpart = rest.trim_start();
            hexpart.len() == hexlen && hexpart.bytes().all(|b| b.is_ascii_hexdigit())
        });
        assert!(
            found,
            "--json with the card: stderr carries no {kind} digest line, so the \
             human reading `for md compose: sha256=` has nothing telling them a \
             kind was never chosen.\n{se}"
        );
    }
}

/// Spec §3 F1: `OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL`. `<hashop>`
/// is the VARIABLE; the 32 is not. The card said `OP_SHA256` under every kind
/// for one release — a false statement about the object in the operator's hand,
/// on the one axis this cycle exists to disambiguate, in the line a confused
/// operator would self-check against (journey walk, I-1).
#[test]
fn the_card_names_the_kinds_own_opcode() {
    for (kind, op) in [
        ("sha256", "OP_SHA256"),
        ("hash256", "OP_HASH256"),
        ("ripemd160", "OP_RIPEMD160"),
        ("hash160", "OP_HASH160"),
    ] {
        let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        let line = se
            .lines()
            .find(|l| l.contains("OP_SIZE 32"))
            .unwrap_or_else(|| panic!("{kind}: no script line on the card"));
        assert!(
            line.contains(&format!("before {op}")),
            "{kind}: the card says {line:?} — an operator self-checking against \
             it confirms the wrong hash function"
        );
        // The width half is correct for all four and must survive.
        assert!(
            line.contains("32 bytes (64 hex characters)"),
            "{kind}: {line:?}"
        );
    }
}

/// The `for md compose:` line proposes a command fragment with full confidence,
/// and `md compose` refuses a non-sha256 operand with "unknown option
/// ripemd160" until phase 1 ships — which reads as a typo, not as "not wired up
/// yet" (journey walk, I-2). Absent for sha256, which works today.
#[test]
fn a_non_sha256_kind_warns_that_md_may_not_accept_the_operand() {
    for kind in ["hash256", "ripemd160", "hash160"] {
        let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        // BOTH doors the journey walk found, and they live on DIFFERENT SIDES
        // of the card/notice boundary: `md compose` is about the card's own
        // operand line, `me sysw pack` is about the record on stdout, which
        // ships whether or not the card does (R0 round 7 M-1, round 8 M-3).
        assert!(
            se.contains(&format!("requires `{kind}=` support in `md compose`")),
            "{kind}: the card proposes an operand with no hint that `md` may \
             refuse it:\n{se}"
        );
        // Quoted as `md` actually prints it -- measured, backticks included.
        assert!(
            se.contains(&format!("unknown option `{kind}`")),
            "{kind}: the caveat must quote md's real refusal:\n{se}"
        );
        // The stdout half survives suppression, because that is the path it
        // describes.
        let (_, se2) = run(&[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--kind",
            kind,
            "--no-engraving-card",
        ]);
        assert!(
            se2.contains("`me sysw pack`"),
            "{kind}: the record still ships under --no-engraving-card and the \
             notice about its second door does not:\n{se2}"
        );
    }
    let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", "sha256"]);
    assert!(
        !se.contains("requires `md compose` support"),
        "sha256 works today; warning about it is noise:\n{se}"
    );
}

/// Spec §13.2's operator-facing Critical at the moment of emission: `hash256`
/// and `sha256` digests are BOTH 64 hex, so the tag is all that separates them
/// — and `me sysw pack`'s refusal ("must be exactly 64 hex characters") is
/// satisfied by deleting the tag, which yields a record accepted as sha256
/// (journey walk, I-3). Only `hash256` is exposed; the 20-byte kinds are not.
#[test]
fn hash256_warns_against_stripping_its_own_tag() {
    // BOTH argv shapes. It is a NOTICE, not a card line, so
    // `--no-engraving-card` must not suppress it -- and that flag is part of
    // the very pipe the hazard travels (`ms hashlock … | me sysw pack`). It
    // shipped inside the card guard once, leaving stderr at zero bytes while
    // `hash:hash256:…` still went to stdout (R0 round 7, I-1).
    for argv in [
        &["hashlock", "--hashlock-phrase-stdin", "--kind", "hash256"][..],
        &[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--kind",
            "hash256",
            "--no-engraving-card",
        ][..],
    ] {
        let (_, se) = run(argv);
        assert!(
            se.contains("DO NOT DELETE THE TAG"),
            "{argv:?}: hash256's record is one prefix-strip from a valid sha256 \
             record and nothing says so:\n{se}"
        );
    }
    for kind in ["sha256", "ripemd160", "hash160"] {
        let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", kind]);
        assert!(
            !se.contains("DO NOT DELETE THE TAG"),
            "{kind}: not the 64-hex collision; the warning is noise here:\n{se}"
        );
    }
}

/// §5's two axes are METHOD (phrase → preimage) and KIND (preimage → digest).
/// The write-down instruction named only the first, and §13.2's own rule is
/// that "a write-down list that omits a field is worse than no list, because
/// the operator stops writing where the list stops" (journey walk, I-4).
#[test]
fn the_write_down_line_names_both_axes() {
    let (_, se) = run(&["hashlock", "--hashlock-phrase-stdin", "--kind", "ripemd160"]);
    let line = se
        .lines()
        .find(|l| l.starts_with("phrase:"))
        .expect("no write-down line on the card");
    // ASSERT THE INSTRUCTION, NOT THE WORDS. This asserted
    // `contains("method line")` for one round, which the RECOVERY clause later
    // in the same line ("if the method line is lost") also satisfies -- so
    // deleting `write the method line AND` from the instruction left the suite
    // at 11 passed, 0 failed. A test that cannot fail for the thing it names
    // (R0 round 7, I-2).
    assert!(
        line.contains("write the method line AND the hash line"),
        "the write-down INSTRUCTION must name both axes; a later clause \
         mentioning either word is not the instruction: {line:?}"
    );
    assert!(
        line.contains("(ripemd160)"),
        "the instruction must name the kind in hand, not the word \"hash\": {line:?}"
    );
    // BOTH recovery clauses, not just the method one. An operator who loses the
    // hash line needs the §13.4 lookup named, and that clause was unpinned
    // while its method twin was (R0 round 8, M-2).
    assert!(
        line.contains("if the method line is lost"),
        "the method-axis recovery is missing: {line:?}"
    );
    assert!(
        line.contains("if the hash line is lost"),
        "the kind-axis recovery is missing, so losing it has no stated remedy: \
         {line:?}"
    );
}

/// The fallback header must not call the object a phrase: `--random`, `--hex`
/// and the ms1-plate route have none, and on `--random` the line lands two
/// lines after "No phrase exists" (journey walk, M-1). The digest is of the
/// PREIMAGE on every route, so one word is true everywhere.
#[test]
fn the_fallback_header_names_the_preimage_not_a_phrase() {
    // `--hex -` reads stdin: the argv guard refuses 64 raw hex characters on the
    // command line BEFORE clap parses, which is the behaviour, not an obstacle.
    let hex64 = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
    for (argv, stdin) in [
        (&["hashlock", "--hashlock-phrase-stdin"][..], PHRASE),
        (&["hashlock", "--hex", "-"][..], hex64),
    ] {
        let out = Command::cargo_bin("ms")
            .unwrap()
            .args(argv)
            .write_stdin(stdin)
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).into_owned();
        let line = se
            .lines()
            .find(|l| l.contains("digest under each kind"))
            .unwrap_or_else(|| panic!("{argv:?}: no fallback header:\n{se}"));
        assert!(
            !line.contains("phrase"),
            "{argv:?}: the header calls it a phrase, and this route has none: {line:?}"
        );
    }
}

/// The stderr purity contract, pinned ACROSS KINDS.
///
/// `hashlock_outputs.rs::json_both_variants` pins stderr under `--json
/// --no-engraving-card` to exactly the PrivateKeyMaterial advisory — but it
/// never passes `--kind`, so it is blind to any notice that only a particular
/// kind fires. The `hash256` tag-strip warning is exactly that, and it shipped
/// unguarded for one round, putting 485 bytes on a stream pinned to one line
/// (R0 round 8, I-1).
///
/// Every kind, and the kind-omitted case, in one place.
#[test]
fn the_json_purity_contract_holds_for_every_kind() {
    for kind in [
        None,
        Some("sha256"),
        Some("hash256"),
        Some("ripemd160"),
        Some("hash160"),
    ] {
        let mut argv = vec![
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--no-engraving-card",
        ];
        if let Some(k) = kind {
            argv.push("--kind");
            argv.push(k);
        }
        let (_, se) = run(&argv);
        let lines: Vec<&str> = se.trim_end().lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "{kind:?}: `--json --no-engraving-card` pins stderr to exactly the \
             advisory; a notice does not get to break that contract, it moves \
             into the object.\n{se}"
        );
        assert!(
            lines[0].contains("stdout carries private key material"),
            "{kind:?}: the one line must be the advisory: {:?}",
            lines[0]
        );
    }
}
