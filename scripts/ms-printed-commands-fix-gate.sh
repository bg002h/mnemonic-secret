#!/usr/bin/env bash
# Row checker for design/FIX_bughunt1119_ms_printed_commands.md (bh-ms-I6, bh-ms-I9).
#
# What it does: runs the RED rows and control rows of the FIX document's section 4
# against the `ms` binary given, each in a fresh directory: it makes `ms` print
# its command (the `--in` hex note of encode/split, the user-manpath example of
# `ms gen-man --help`), runs that line in `sh -c` exactly as printed, and checks
# the result. Mode `red` requires every RED row to show its defect and every
# control row to hold (the default branch); mode `green` requires every row to
# hold (the fixed build). Exit 0 iff the mode's expectation holds for all rows;
# one line per row: `PASS|FAIL <mode> <row> :: <observed>`, then `rows failed: N`.
#
# What it does NOT cover: the Rust test functions the builder writes (section 5),
# the full workspace suite, fmt, clippy, docs-only, code-docs-check, the macOS,
# musl, release, miri, FreeBSD and fuzz CI legs, shells other than `sh`, the
# toolkit manual, and any review. Test vector only: 16 zero bytes of entropy.
#
# Usage: scripts/ms-printed-commands-fix-gate.sh red|green <ms binary> <scratch dir, not under /tmp>
set -u
MODE=$1 M=$(realpath "$2") BASE=$3
case "$BASE" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
case "$MODE" in red|green) ;; *) echo "mode is red or green" >&2; exit 2;; esac
mkdir -p "$BASE"; BASE=$(realpath "$BASE")
BIN=$(mktemp -d "$BASE/bin.XXXXXX"); ln -s "$M" "$BIN/ms"
SPATH="$BIN:/usr/bin:/bin"
ZERO=00000000000000000000000000000000
C12=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
FAILS=0
fresh() { local d="$BASE/$1"; rm -rf "$d"; mkdir -p "$d"; echo "$d"; }
verdict() { # verdict <row> <kind red|ctl> <holds 0|1> <observed>
  local row=$1 kind=$2 holds=$3 obs=$4 ok
  if [ "$MODE" = green ] || [ "$kind" = ctl ]; then ok=$holds; else ok=$((1 - holds)); fi
  if [ "$ok" = 1 ]; then echo "PASS $MODE $row :: $obs"; else echo "FAIL $MODE $row :: $obs"; FAILS=$((FAILS + 1)); fi
}
# remedy_row <row> <file name> <expected printed line> <ms args...>
# Runs `ms <args>` (rc 1 expected), takes the stderr line that starts with `ms `
# after trimming, runs it in `sh -c` in the same directory, and checks: the line
# is the expected text, it exits 0, and its stdout ends with the canonical card
# (encode) or holds the expected number of shares (split).
remedy_row() {
  local row=$1 name=$2 want=$3; shift 3
  local d; d=$(fresh "$row"); echo "$ZERO" >"$d/$name"
  (cd "$d" && PATH="$SPATH" "$M" "$@" </dev/null >out.txt 2>err.txt); local rc0=$?
  local line; line=$(sed -e 's/^[[:space:]]*//' "$d/err.txt" | grep -m1 '^ms ')
  (cd "$d" && env -i PATH="$SPATH" HOME="$d" sh -c "$line" >run.out 2>run.err); local rc=$?
  local ok=0 body src="$d/run.out"
  [ -n "${OUTFILE:-}" ] && src="$d/$OUTFILE"
  case "$1" in
    encode) body=$(tail -1 "$src" 2>/dev/null); [ "$body" = "$C12" ] && ok=1 ;;
    split) body=$(grep -c '^ms1' "$src" 2>/dev/null)
      if [ "$body" = "$EXPECT_SHARES" ] && [ -n "${OUTFILE:-}" ]; then
        head -2 "$src" >"$d/two.txt"
        (cd "$d" && "$M" combine --in two.txt --json </dev/null >comb.out 2>comb.err)
        grep -q "\"entropy_hex\":\"$ZERO\"" "$d/comb.out" || body="combine-mismatch"
      fi
      [ "$body" = "$EXPECT_SHARES" ] && ok=1 ;;
  esac
  [ "$rc0" = 1 ] && [ "$line" = "$want" ] && [ "$rc" = 0 ] && [ "$ok" = 1 ]
  verdict "$row" red $((1 - $?)) "rc=$rc0 printed=[$line] run-rc=$rc run-err=[$(head -1 "$d/run.err")]"
}

# R1: split's note carries the threshold and share count it was given.
EXPECT_SHARES=3 remedy_row printed_cmd_split_remedy_runs entropy.hex \
  "ms split -k 2 -n 3 --hex - < 'entropy.hex'" split -k 2 -n 3 --in entropy.hex
EXPECT_SHARES=5 remedy_row printed_cmd_split_remedy_long_flags_runs entropy.hex \
  "ms split -k 3 -n 5 --hex - < 'entropy.hex'" split --in entropy.hex --shares 5 --threshold 3
# R2: the path is single-quoted, an embedded ' written as '\''.
remedy_row printed_cmd_encode_remedy_text entropy.hex \
  "ms encode --hex - < 'entropy.hex'" encode --in entropy.hex
remedy_row printed_cmd_encode_remedy_path_with_space_runs "two words.hex" \
  "ms encode --hex - < 'two words.hex'" encode --in "two words.hex"
remedy_row printed_cmd_encode_remedy_path_with_quote_runs "it's.hex" \
  "ms encode --hex - < 'it'\\''s.hex'" encode --in "it's.hex"
EXPECT_SHARES=3 remedy_row printed_cmd_split_remedy_path_with_space_runs "two words.hex" \
  "ms split -k 2 -n 3 --hex - < 'two words.hex'" split -k 2 -n 3 --in "two words.hex"

# Q2 (b): K/N out of range are refused before the secret is read: exit 1, stderr is only
# the range error (no `note:` line), nothing on stdout.
# range_row <row> <expected stderr> <ms args...>
range_row() {
  local row=$1 want=$2; shift 2
  local d; d=$(fresh "$row"); echo "$ZERO" >"$d/entropy.hex"
  (cd "$d" && PATH="$SPATH" "$M" "$@" </dev/null >out.txt 2>err.txt); local rc=$?
  local err; err=$(cat "$d/err.txt")
  [ "$rc" = 1 ] && [ "$err" = "$want" ] && [ ! -s "$d/out.txt" ]
  verdict "$row" red $((1 - $?)) "rc=$rc stderr=[$(head -1 "$d/err.txt")] lines=$(wc -l <"$d/err.txt")"
}
range_row printed_cmd_split_bad_threshold_refused_before_read \
  "error: invalid threshold 1; K-of-N shares require k in 2..=9" split -k 1 -n 3 --in entropy.hex
range_row printed_cmd_split_bad_share_count_refused_before_read \
  "error: invalid share count n=3 for threshold k=5; require k <= n <= 31" split -k 5 -n 3 --in entropy.hex

# R3: `--out` as given is carried, quoted, before the hex channel.
OUTFILE="my card.ms1" remedy_row printed_cmd_encode_remedy_keeps_out entropy.hex \
  "ms encode --out='my card.ms1' --hex - < 'entropy.hex'" encode --in entropy.hex --out "my card.ms1"
OUTFILE=shares.txt EXPECT_SHARES=3 remedy_row printed_cmd_split_remedy_keeps_out_and_recombines "two words.hex" \
  "ms split -k 2 -n 3 --out='shares.txt' --hex - < 'two words.hex'" split -k 2 -n 3 --in "two words.hex" --out shares.txt

# R4: the user-manpath example of `ms gen-man --help` runs with XDG_DATA_HOME unset or empty.
EX=$("$M" gen-man --help | sed -e 's/^[[:space:]]*//' | grep -m1 '^ms gen-man --out "\$' | sed -e 's/[[:space:]]*#.*$//')
d=$(fresh printed_cmd_gen_man_example_runs_without_xdg)
(cd "$d" && env -i PATH="$SPATH" HOME="$d" sh -c "$EX" >run.out 2>run.err); rc=$?
[ "$rc" = 0 ] && [ -f "$d/.local/share/man/man1/ms.1" ]
verdict printed_cmd_gen_man_example_runs_without_xdg red $((1 - $?)) "example=[$EX] rc=$rc err=[$(head -1 "$d/run.err")]"
d=$(fresh printed_cmd_gen_man_example_runs_with_empty_xdg)
(cd "$d" && env -i PATH="$SPATH" HOME="$d" XDG_DATA_HOME= sh -c "$EX" >run.out 2>run.err); rc=$?
[ "$rc" = 0 ] && [ -f "$d/.local/share/man/man1/ms.1" ]
verdict printed_cmd_gen_man_example_runs_with_empty_xdg red $((1 - $?)) "rc=$rc err=[$(head -1 "$d/run.err")]"
# Control: with XDG_DATA_HOME set, the example writes there and nowhere under HOME.
d=$(fresh printed_cmd_gen_man_example_honours_xdg)
(cd "$d" && env -i PATH="$SPATH" HOME="$d/home" XDG_DATA_HOME="$d/xdg" sh -c "$EX" >run.out 2>run.err); rc=$?
[ "$rc" = 0 ] && [ -f "$d/xdg/man/man1/ms.1" ] && [ ! -e "$d/home" ]
verdict printed_cmd_gen_man_example_honours_xdg ctl $((1 - $?)) "rc=$rc err=[$(head -1 "$d/run.err")]"
# Control: the hex channel itself reads the file (both builds).
d=$(fresh printed_cmd_hex_channel_control); echo "$ZERO" >"$d/entropy.hex"
(cd "$d" && "$M" encode --hex - <entropy.hex >run.out 2>run.err); rc=$?
[ "$rc" = 0 ] && [ "$(tail -1 "$d/run.out")" = "$C12" ]
verdict printed_cmd_hex_channel_control ctl $((1 - $?)) "rc=$rc card=$(tail -1 "$d/run.out")"

echo "rows failed: $FAILS"
[ "$FAILS" = 0 ]
