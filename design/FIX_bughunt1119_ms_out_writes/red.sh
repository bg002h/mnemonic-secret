#!/usr/bin/env bash
# design/FIX_bughunt1119_ms_out_writes/red.sh -- the FIX doc's rows o1-o21 and
# the out-of-scope evidence x1-x4, as printed commands against one `ms` binary.
#
# Usage: MS=<path to ms> bash design/FIX_bughunt1119_ms_out_writes/red.sh > OUT
# Prints one block per row: exit code, stdout, stderr, each named file's size,
# mode and first bytes. It asserts nothing: the FIX doc's section 4 table holds
# the expected column for the base and for the fix; the test binary
# crates/ms-cli/tests/bh1119_ms_out_writes.rs is the assertion.
# Needs Linux (prlimit, /dev/stdout, mkfifo, GNU stat). Not covered: macOS,
# Windows, FreeBSD; a crash between the temp write and the rename.
# Test data only: entropy 00..00 / 11..11 and preimages 01.. / 02.. (no wallet).
set -u
MS="${MS:?set MS to the ms binary}"
D="$(mktemp -d "${TMPDIR_BASE:-$PWD}/ms-out-writes-run.XXXXXX")"
cd "$D" || exit 1
A=00000000000000000000000000000000
B=11111111111111111111111111111111
H=0101010101010101010101010101010101010101010101010101010101010101
H2=0202020202020202020202020202020202020202020202020202020202020202
# A file-size limit of 8 bytes with SIGXFSZ ignored: the write returns EFBIG,
# standing in for a full disk or a quota. stderr goes through a pipe, because
# the limit would also cut a stderr FILE.
FS8() { bash -c "trap '' XFSZ; exec prlimit --fsize=8 \"\$@\"" _ "$@"; }
show() { # $1 row, $2 rc, then files
  local row=$1 rc=$2; shift 2
  echo "== $row rc=$rc"
  echo "-- stdout:"; cat out 2>/dev/null
  echo "-- stderr:"; cat err 2>/dev/null
  for f in "$@"; do
    if [ -L "$f" ]; then echo "-- $f symlink -> $(readlink "$f")"; fi
    if [ -e "$f" ]; then echo "-- $f ($(stat -c '%s bytes %a' "$f")):"; head -c 72 "$f" | od -An -c | head -5; else echo "-- $f absent"; fi
  done
  echo "-- dir: $(ls -A | tr '\n' ' ')"
}
reset() { rm -f out err; }
enc() { printf '%s\n' "$1" | "$MS" encode --hex - --out "$2" --no-engraving-card >/dev/null 2>&1; }
# One substitution in the 00..00 card: repair corrects it (exit 4, report on stdout).
printf 'ms10entrsqqqqqqqqqqqpqqqqqqqqqqqqqqqqcj9sxraq34v7f\n' > BAD

# ---- I13: a failed write keeps the old file (R1, R2) ----
enc "$A" F; cp F F.orig
reset; printf '%s\n' "$B" | FS8 "$MS" encode --hex - --out F --no-engraving-card >out 2> >(cat > err); rc=$?; sleep 0.1
show o1_encode_failed_overwrite_keeps_previous_card $rc F; cmp -s F F.orig && echo "F == previous" || echo "F CHANGED"
"$MS" verify --in F >/dev/null 2>&1; echo "verify --in F rc=$?"
rm -f F F.orig

printf '%s\n' "$H" | "$MS" hashlock --hex - --kind sha256 --no-engraving-card --out P >/dev/null 2>&1; cp P P.orig
reset; printf '%s\n' "$H2" | FS8 "$MS" hashlock --hex - --kind sha256 --no-engraving-card --out P >out 2> >(cat > err); rc=$?; sleep 0.1
show o2_hashlock_failed_overwrite_keeps_previous_plate $rc P; cmp -s P P.orig && echo "P == previous" || echo "P CHANGED"
rm -f P P.orig

enc "$A" C; cp C C.orig
reset; FS8 "$MS" repair --in BAD --out C >out 2> >(cat > err); rc=$?; sleep 0.1
show o3_repair_failed_overwrite_keeps_previous_card $rc C; cmp -s C C.orig && echo "C == previous" || echo "C CHANGED"
rm -f C C.orig

reset; printf '%s\n' "$A" | FS8 "$MS" encode --hex - --out N --no-engraving-card >out 2> >(cat > err); rc=$?; sleep 0.1
show o4_absent_target_failed_write_leaves_nothing $rc N
rm -f N

reset; printf '%s\n' "$A" | "$MS" encode --hex - --out nodir/x.ms1 --no-engraving-card >out 2>err; rc=$?
show o5_missing_directory_text_and_exit $rc nodir/x.ms1

if [ "$(id -u)" != 0 ]; then
  mkdir ro; enc "$A" ro/F; cp ro/F F.orig; chmod 0500 ro
  reset; printf '%s\n' "$B" | "$MS" encode --hex - --out ro/F --no-engraving-card >out 2>err; rc=$?
  show o6_read_only_directory_refuses_and_keeps_file $rc ro/F; cmp -s ro/F F.orig && echo "ro/F == previous" || echo "ro/F CHANGED"
  chmod 0700 ro; rm -f ro/F F.orig; rmdir ro
else
  echo "== o6_read_only_directory_refuses_and_keeps_file SKIPPED (euid 0)"
fi

printf 'OLD\n' > T; chmod 0644 T; ln -s T L
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out L --no-engraving-card >out 2>err; rc=$?
show o7_symlink_out_replaces_target_keeps_link $rc L T
rm -f T L

printf 'OLD BACKUP LINE\n' > S2
reset; printf '%s\n' "$A" | FS8 "$MS" encode --hex - --out /dev/stdout --no-engraving-card >>S2 2> >(cat > err); rc=$?; sleep 0.1
show o8_stdio_alias_onto_a_file_names_no_state $rc S2
rm -f S2

reset; printf '%s\n' "$A" | FS8 "$MS" split -k 2 -n 3 --hex - --out SH >out 2> >(cat > err); rc=$?; sleep 0.1
show o9_create_new_failed_write_text $rc SH
reset; printf '%s\n' "$A" | "$MS" split -k 2 -n 3 --hex - --out SH >out 2>err; rc=$?
show o9b_create_new_retry_after_failed_write $rc SH
rm -f SH

reset; FS8 "$MS" hashlock --random --no-engraving-card --out R >out 2> >(cat > err); rc=$?; sleep 0.1
show o9c_random_preimage_failed_write_leaves_nothing $rc R
rm -f R

# ---- I1: --out and standard output are one file (R3) ----
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F --json --no-engraving-card >F 2>err; rc=$?
show o10_encode_json_stdout_is_out_refused $rc F
"$MS" verify --in F >/dev/null 2>&1; echo "verify --in F rc=$?"
rm -f F

: > F; ln F F2
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F --json --no-engraving-card >F2 2>err; rc=$?
show o11_encode_json_stdout_hard_link_refused $rc F
rm -f F F2

: > F; ln -s F LF
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F --json --no-engraving-card >LF 2>err; rc=$?
show o12_encode_json_stdout_symlink_refused $rc F
rm -f F LF

reset; printf '%s\n' "$H" | "$MS" hashlock --hex - --kind sha256 --no-engraving-card --out P >P 2>err; rc=$?
show o13_hashlock_stdout_is_out_refused $rc P
"$MS" hashlock --kind sha256 --in P --no-engraving-card >/dev/null 2>&1; echo "hashlock --in P rc=$?"
rm -f P

reset; printf '%s\n' "$H" | "$MS" hashlock --hex - --kind sha256 --json --no-engraving-card --out P >P 2>err; rc=$?
show o14_hashlock_json_stdout_is_out_refused $rc P
rm -f P

reset; "$MS" repair --in BAD --out C >C 2>err; rc=$?
show o15_repair_stdout_is_out_refused $rc C
"$MS" verify --in C >/dev/null 2>&1; echo "verify --in C rc=$?"
rm -f C

# ---- controls ----
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F --no-engraving-card >F 2>err; rc=$?
show o16_text_encode_out_with_stdout_on_it_still_writes $rc F
"$MS" verify --in F >/dev/null 2>&1; echo "verify --in F rc=$?"
rm -f F

reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F 2>F >out; rc=$?
show o17_stderr_on_the_out_file_keeps_the_card $rc F
"$MS" verify --in F >/dev/null 2>&1; echo "verify --in F rc=$?"
rm -f F

reset; printf '%s\n' "$A" | "$MS" split -k 2 -n 3 --hex - --out SH >SH 2>err; rc=$?
show o18_split_out_with_stdout_on_it_refused_as_existing $rc SH
rm -f SH

reset; "$MS" hashlock --random --no-engraving-card --out R >R 2>err; rc=$?
show o19_hashlock_random_out_with_stdout_on_it_refused_as_existing $rc R
rm -f R

mkfifo FF; ( cat FF > fifo.got & ); sleep 0.2
reset; printf '%s\n' "$A" | "$MS" encode --hex - --out FF --no-engraving-card >out 2>err; rc=$?; sleep 0.2
show o20_fifo_out_written_directly $rc fifo.got
rm -f FF fifo.got

reset; printf '%s\n' "$A" | "$MS" encode --hex - --out F --json --no-engraving-card 2>err | cat > out; rc=${PIPESTATUS[1]}
show o21_json_out_with_piped_stdout_writes_both $rc F
rm -f F

# ---- out of scope (Brian 2026-11-23 Q2, a separate row): evidence only ----
printf 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n' > S
reset; "$MS" encode --in S --out S --no-engraving-card >out 2>err; rc=$?
show x1_encode_in_is_out $rc S
rm -f S
printf 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n' > S
reset; "$MS" encode --phrase - --out S --no-engraving-card <S >out 2>err; rc=$?
show x2_encode_stdin_is_out $rc S
rm -f S
cp BAD C
reset; "$MS" repair --ms1 - --out C <C >out 2>err; rc=$?
show x3_repair_stdin_is_out $rc C
rm -f C
printf '%s\n' "$H" | "$MS" hashlock --hex - --kind sha256 --no-engraving-card --out P >/dev/null 2>&1
reset; "$MS" hashlock --kind sha256 --in P --no-engraving-card --out P >out 2>err; rc=$?
show x4_hashlock_in_is_out $rc P
rm -f P BAD out err
cd .. && rmdir "$D" && echo "== done"
