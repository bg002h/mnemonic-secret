//! fixdoc-ms-codex32-ctors-m1-unchecksummed-unscrubbed: `from_unchecksummed_string`
//! must not free the caller's secret body unscrubbed (growth, early `?` return).
//!
//! A tracking allocator checks every freed block (a realloc is modelled as
//! alloc + copy + free, so the old block is always seen at its free).

use ms_codec::codex32::Codex32String as C;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 26 distinct bech32 data characters; no other test allocation holds them.
const SENTINEL: &[u8] = b"pzry9x8gf2tvdw0s3jn54khce6";
static LEAKS: AtomicUsize = AtomicUsize::new(0);

fn holds_sentinel(p: *const u8, n: usize) -> bool {
    n >= SENTINEL.len()
        && unsafe { std::slice::from_raw_parts(p, n) }
            .windows(SENTINEL.len())
            .any(|w| w == SENTINEL)
}

struct Tracking;
unsafe impl GlobalAlloc for Tracking {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        if holds_sentinel(p, l.size()) {
            LEAKS.fetch_add(1, Ordering::SeqCst);
        }
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let n = System.alloc(Layout::from_size_align_unchecked(new, l.align()));
        if !n.is_null() {
            std::ptr::copy_nonoverlapping(p, n, l.size().min(new));
            self.dealloc(p, l);
        }
        n
    }
}
#[global_allocator]
static A: Tracking = Tracking;

/// Exact-capacity body: "ms10entrs" + SENTINEL (35 chars) [+ tail].
fn body(tail: &str) -> String {
    let mut v = Vec::with_capacity(64);
    v.extend_from_slice(b"ms10entrs");
    v.extend_from_slice(SENTINEL);
    v.extend_from_slice(tail.as_bytes());
    String::from_utf8(v).unwrap().as_str().into()
}

#[test]
fn fixdoc_ms_codex32_ctors_m1_unchecksummed_unscrubbed() {
    // Growth path (reserve_exact on a full-capacity String).
    let b = body("");
    let before = LEAKS.load(Ordering::SeqCst);
    let r = C::from_unchecksummed_string(b);
    drop(r);
    assert_eq!(
        LEAKS.load(Ordering::SeqCst),
        before,
        "growth path freed an unscrubbed copy of the body"
    );

    // Early `?` return ('b' is not a bech32 character).
    let b = body("b");
    let before = LEAKS.load(Ordering::SeqCst);
    let r = C::from_unchecksummed_string(b);
    assert!(r.is_err());
    drop(r);
    assert_eq!(
        LEAKS.load(Ordering::SeqCst),
        before,
        "early-return path freed an unscrubbed body"
    );
}
