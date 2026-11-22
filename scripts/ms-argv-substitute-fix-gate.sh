#!/usr/bin/env bash
# Row checker for design/FIX_bughunt1119_ms_argv_substitute.md (bh-ms-C1, bh-ms-C2).
#
# What it does: runs every RED row and every control row of the FIX document's
# section 4 against the `ms` binary given, each in a fresh directory, and checks
# the printed expectation. Mode `red` requires every RED row to show its defect
# and every control row to hold (the default branch); mode `green` requires every
# row to hold (the fixed build). Exit 0 iff the mode's expectation holds for all
# rows; one line per row: `PASS|FAIL <mode> <row> :: <observed>`.
#
# What it does NOT cover: the Rust test functions the builder writes (section 5),
# the full workspace suite, fmt, clippy, docs-only, code-docs-check, the macOS,
# musl, release, miri, FreeBSD and fuzz CI legs, the toolkit manual, and any
# review. Signet/test vectors only: the all-zero 12- and 15-word BIP-39 phrases.
#
# Usage: scripts/ms-argv-substitute-fix-gate.sh red|green <ms binary> <scratch dir, not under /tmp>
set -u
MODE=$1 M=$(realpath "$2") BASE=$3
case "$BASE" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
case "$MODE" in red|green) ;; *) echo "mode is red or green" >&2; exit 2;; esac
mkdir -p "$BASE"; BASE=$(realpath "$BASE")
P12="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
P15="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address"
C12=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
C15=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqke34pjrgn5p6k
PLATE=ms10hashsqvxdut60pd6jqph6ld525scg2cgjd7wkfqwtv5d9ym0whh6462afc7jgxua504u3ms
SH1=ms124gwqqj5pjwlwrxuxsc8mhrxrhpjh7ny3q8mw8zv43meqgf
SH2=ms124gwqpklhk4n4sfrf3d7a0sfs0hk0ypwxqdj0vx0ucnm0ks
N=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
N64=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
SIX=59413e908f3b72e4ce859bf153fffc8054615bc3e445f7ac5e7b2adb9b8bbe70
FIVE=897abbb178d59a98b5bbc2f51aa5ef612730a437bd717f8a5b344d8a747d987f
FAILS=0
RC=0
run() { # run <dir> <args...> ; leaves rc in RC, out.txt and err.txt in the dir
  local d=$1; shift
  (cd "$d" && "$M" "$@" </dev/null >out.txt 2>err.txt); RC=$?
}
fresh() { local d="$BASE/$1"; rm -rf "$d"; mkdir -p "$d"; echo "$d"; }
verdict() { # verdict <row> <kind red|ctl> <holds 0|1> <observed>
  local row=$1 kind=$2 holds=$3 obs=$4 ok
  if [ "$MODE" = green ] || [ "$kind" = ctl ]; then ok=$holds; else ok=$((1 - holds)); fi
  if [ "$ok" = 1 ]; then echo "PASS $MODE $row :: $obs"; else echo "FAIL $MODE $row :: $obs"; FAILS=$((FAILS + 1)); fi
}
wc_of() { grep -o '"word_count":[0-9]*' "$1/out.txt" | cut -d: -f2; }
miss_line() { echo "error: failed to read --in $1: No such file or directory (os error 2)"; }

# R1 read side, space spelling: the named 15-word file, not the 12-word ./- decoy.
d=$(fresh r1); echo "$P15" >"$d/$N"; echo "$P12" >"$d/-"
run "$d" encode --allow-argv-secret --in "$N" --json
w=$(wc_of "$d"); [ "$RC" = 0 ] && [ "$w" = 15 ]; verdict argv_sub_in_space_reads_named_file red $((1 - $?)) "rc=$RC word_count=$w"

# R2 read side, equals spelling.
d=$(fresh r2); echo "$P15" >"$d/$N"; echo "$P12" >"$d/-"
run "$d" encode --allow-argv-secret "--in=$N" --json
w=$(wc_of "$d"); [ "$RC" = 0 ] && [ "$w" = 15 ]; verdict argv_sub_in_equals_reads_named_file red $((1 - $?)) "rc=$RC word_count=$w stderr=$(head -1 "$d/err.txt")"

# R3 read side, a 64-hex name and an ms1-shaped name.
d=$(fresh r3); echo "$C15" >"$d/$N64"; echo "$C12" >"$d/-"
run "$d" inspect --allow-argv-secret --in "$N64" --json
grep -q '"payload_bytes_hex":"0000000000000000000000000000000000000000"' "$d/out.txt"; h=$?
[ "$RC" = 0 ] && [ "$h" = 0 ]; verdict argv_sub_in_64_hex_name_reads_named_file red $((1 - $?)) "rc=$RC 20-byte-payload=$((1 - h))"
d=$(fresh r3b); echo "$C15" >"$d/$C12"; echo "$C12" >"$d/-"
run "$d" verify --allow-argv-secret --in "$C12"
o=$(head -1 "$d/out.txt"); [ "$RC" = 0 ] && [ "$o" = "OK: valid v0.1 entr (15 words, 56 chars)" ]
verdict argv_sub_in_ms1_shaped_name_reads_named_file red $((1 - $?)) "rc=$RC stdout=$o"

# R4 every verb with --in: the named file is absent and ./- holds valid input -> exit 1, the read error.
for v in encode decode inspect verify derive hashlock repair split combine; do
  d=$(fresh "r4-$v")
  case $v in
    encode|split) echo "$P12" >"$d/-" ;;
    hashlock) echo "$PLATE" >"$d/-" ;;
    combine) printf '%s\n%s\n' "$SH1" "$SH2" >"$d/-" ;;
    *) echo "$C12" >"$d/-" ;;
  esac
  case $v in
    derive) run "$d" derive --allow-argv-secret --in "$N" --network testnet ;;
    hashlock) run "$d" hashlock --allow-argv-secret --in "$N" --kind sha256 ;;
    split) run "$d" split --allow-argv-secret -k 2 -n 3 --in "$N" ;;
    *) run "$d" "$v" --allow-argv-secret --in "$N" ;;
  esac
  e=$(cat "$d/err.txt"); [ "$RC" = 1 ] && [ "$e" = "$(miss_line "$N")" ] && [ ! -s "$d/out.txt" ]
  verdict "argv_sub_in_missing_named_file_fails[$v]" red $((1 - $?)) "rc=$RC stderr=$(head -1 "$d/err.txt" | cut -c1-90)"
done

# R5 write side, space spelling, every --out verb: the named file exists (0600), ./- does not.
for v in encode hashlock repair split; do
  d=$(fresh "r5-$v"); echo "$P12" >"$d/seed.txt"; echo "$C12" >"$d/card.ms1"
  case $v in
    encode) run "$d" encode --allow-argv-secret --in seed.txt --out "$N" ;;
    hashlock) run "$d" hashlock --allow-argv-secret --random --kind sha256 --no-engraving-card --out "$N" ;;
    repair) run "$d" repair --allow-argv-secret --in card.ms1 --out "$N" ;;
    split) run "$d" split --allow-argv-secret -k 2 -n 3 --in seed.txt --out "$N" ;;
  esac
  mode=$(stat -c %a "$d/$N" 2>/dev/null)
  [ "$RC" = 0 ] && [ "$mode" = 600 ] && [ ! -e "$d/-" ]
  verdict "argv_sub_out_space_creates_named_file[$v]" red $((1 - $?)) "rc=$RC named=${mode:-absent} stray=$([ -e "$d/-" ] && echo yes || echo no)"
done
c=$(cat "$BASE/r5-encode/$N" 2>/dev/null)
[ "$c" = "$C12" ]; verdict argv_sub_out_space_card_bytes red $((1 - $?)) "named-file=${c:-absent}"

# R6 write side, equals spelling.
d=$(fresh r6); echo "$P12" >"$d/seed.txt"
run "$d" encode --allow-argv-secret --in seed.txt "--out=$N"
c=$(cat "$d/$N" 2>/dev/null); [ "$RC" = 0 ] && [ "$c" = "$C12" ] && [ ! -e "$d/-" ]
verdict argv_sub_out_equals_creates_named_file red $((1 - $?)) "rc=$RC named-file=${c:-absent}"

# R7 controls: unchanged on both builds.
d=$(fresh k1); run "$d" decode --allow-argv-secret -- --in "$N"
[ "$RC" = 64 ] && grep -q "unexpected argument '-' found" "$d/err.txt" && ! grep -q "$N" "$d/err.txt"
verdict argv_sub_after_dashdash_still_substituted ctl $((1 - $?)) "rc=$RC"
d=$(fresh k2); run "$d" decode --allow-argv-secret --IN "$N"
[ "$RC" = 64 ] && grep -q "unexpected argument '--IN' found" "$d/err.txt" && ! grep -q "$N" "$d/err.txt"
verdict argv_sub_flag_match_is_exact ctl $((1 - $?)) "rc=$RC"
d=$(fresh k3); echo "$C12" >"$d/card.ms1"; run "$d" derive --allow-argv-secret --in card.ms1 --account 12345678901234567890123456789012
[ "$RC" = 64 ] && grep -q "invalid value '-' for '--account <ACCOUNT>'" "$d/err.txt" && ! grep -q 1234567890 "$d/err.txt"
verdict argv_sub_non_path_value_still_substituted ctl $((1 - $?)) "rc=$RC"
d=$(fresh k4); echo "$P15" >"$d/$N"; run "$d" encode --in "$N" --json
[ "$RC" = 1 ] && grep -q "is raw hex entropy, 32 characters long" "$d/err.txt" && [ ! -s "$d/out.txt" ]
verdict argv_sub_without_override_still_refuses ctl $((1 - $?)) "rc=$RC"

# R8 C2: the `=` spelling admits the same bytes as the space spelling.
hl() { local d; d=$(fresh "$1"); shift; run "$d" hashlock --allow-argv-secret "$@" --method sha256 --json
  grep -o '"preimage_hex":"[0-9a-f]*"' "$d/out.txt" | cut -d'"' -f4; }
eq=$(hl c2eq "--hashlock-phrase=trail "); sp=$(hl c2sp --hashlock-phrase "trail ")
[ "$sp" = "$SIX" ] || { echo "FAIL $MODE control: space spelling drifted ($sp)"; FAILS=$((FAILS + 1)); }
[ "$eq" = "$SIX" ]; verdict argv_sub_hashlock_phrase_equals_keeps_trailing_space red $((1 - $?)) "preimage=$eq (five-byte=$([ "$eq" = "$FIVE" ] && echo yes || echo no))"
eq=$(hl c2leq "--hashlock-phrase= trail"); sp=$(hl c2lsp --hashlock-phrase " trail")
[ -n "$eq" ] && [ "$eq" = "$sp" ]; verdict argv_sub_hashlock_phrase_equals_keeps_leading_space ctl $((1 - $?)) "eq=$eq"
for f in phrase hex ms1; do
  case $f in
    phrase) val=" $P12 " verb=encode ;;
    hex) val=" 00000000000000000000000000000000 " verb=encode ;;
    ms1) val=" $C12 " verb=repair ;;
  esac
  d1=$(fresh "e-$f-eq"); d2=$(fresh "e-$f-sp")
  run "$d1" "$verb" --allow-argv-secret "--$f=$val" --json; r1=$RC
  run "$d2" "$verb" --allow-argv-secret "--$f" "$val" --json; r2=$RC
  [ "$r1" = 0 ] && [ "$r2" = 0 ] && cmp -s "$d1/out.txt" "$d2/out.txt"
  verdict "argv_sub_equals_matches_space[$f]" ctl $((1 - $?)) "rc=$r1/$r2"
done

echo "rows failed: $FAILS"
[ "$FAILS" = 0 ]
