#![feature(f128)]
//! Maximum ulp deviation of three binary128 implementations of each function,
//! measured against MPFR.
//!
//! The three lanes are [`metallic`]'s own `*q` (correctly rounded, so this lane
//! is the harness' own sanity check — it must never read above 0.5), glibc's
//! `*f128` (what nightly's `f128` methods lower to) and GCC's libquadmath `*q`.
//! Both competitors are faithful-only, so the number that matters is how far
//! past a half ulp they go on a population close to the one the benches time.
//!
//! **Metric.** For a sample `x` let `v = f(x)` evaluated in MPFR at
//! [`WORK`] bits.  MPFR rounds every operation correctly, so `v` is within
//! `2^-300` *relative* of the exact value whatever the argument does —
//! cancellation inside the function is MPFR's problem, not ours — and with
//! `E` the binade exponent of `v` (`v ∈ [2^E, 2^(E+1))`),
//!
//! ```text
//! ulp_exp = max(E − 112, −16494)      (the floor is binary128's subnormal ulp)
//! err(a)  = |a − v| / 2^ulp_exp
//! ```
//!
//! evaluated in MPFR and only then narrowed to `f64`, leaves the oracle's own
//! slip at `2^-187` ulp.  Charging the ulp of the *exact* value, not of the
//! returned one, is CORE-MATH's convention and keeps `err ≤ 0.5` equivalent to
//! correct rounding, binade boundaries included — save at an exact midpoint,
//! where both neighbours read `0.5` and only ties-to-even separates them.
//! [`self_check`] pins that
//! scaling against three exact midpoints — above 1, below 1, and on the
//! subnormal grid — each of which must read exactly `0.5`.
//!
//! Samples where `v` is zero or non-finite carry no ulp and are dropped
//! (counted, not silent).  A lane that answers ±∞ where rounding `v` really
//! does overflow binary128 is *correct* and is counted as an exact hit; a lane
//! that answers NaN or the wrong infinity, or a finite value where the answer
//! overflows, is counted as **bogus** — the worst failure there is, so it is
//! never allowed to hide inside a small maximum.  Draws are a local
//! SplitMix64, so a rerun reproduces the table.
//!
//! ```text
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//!     --example f128_ulp_survey -- 20000 sinq tanq
//! ```

use rug::Float;
use rug::ops::Pow as _;
use std::cmp::Ordering;
use std::io::Write as _;

/// Working precision of the oracle: 187 bits past binary128.
const WORK: u32 = 300;

/// Binary128's smallest ulp: the subnormal grid step `2^-16494`.
const SUBNORMAL_ULP: i32 = -16494;

const LANES: [&str; 3] = ["metallic", "glibc", "quadmath"];

// ---------------------------------------------------------------- provenance --

/// Resolving the competitors through the dynamic loader instead of by `extern`
/// declaration, and checking where each entry point actually came from.
///
/// A plain `unsafe extern "C" fn sqrtf128` does **not** reach glibc: Rust's
/// `compiler_builtins` links its own `LOCAL HIDDEN sqrtf128` (libm's generic
/// integer square root) into every binary, the static linker binds the
/// reference to that copy, and `nm -D` on the example then shows no
/// `sqrtf128@GLIBC_2.26` reference at all — the "glibc" column was Rust's own
/// code, which is why it agreed with `metallic` bit for bit.  `dlsym` on an
/// explicit handle cannot be captured that way, and `dladdr` proves for every
/// symbol which shared object answered.
mod dynamic {
    use core::ffi::{c_char, c_int, c_void};
    use core::ptr::{null, null_mut};
    use std::ffi::{CStr, CString};

    #[repr(C)]
    struct Info {
        fname: *const c_char,
        fbase: *mut c_void,
        sname: *const c_char,
        saddr: *mut c_void,
    }

    unsafe extern "C" {
        fn dlopen(file: *const c_char, mode: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
        fn dladdr(addr: *const c_void, info: *mut Info) -> c_int;
    }

    const RTLD_NOW: c_int = 2;

    /// The address of `name` inside `library`, asserted to live there.
    pub fn entry(library: &str, name: &str) -> *mut c_void {
        let file = CString::new(library).expect("library name");
        let symbol = CString::new(name).expect("symbol name");
        let mut info = Info {
            fname: null(),
            fbase: null_mut(),
            sname: null(),
            saddr: null_mut(),
        };

        // SAFETY: both strings are NUL-terminated, and every returned pointer
        // is checked before it is used.
        unsafe {
            let handle = dlopen(file.as_ptr(), RTLD_NOW);
            assert!(!handle.is_null(), "dlopen {library}");
            let addr = dlsym(handle, symbol.as_ptr());
            assert!(!addr.is_null(), "{name} is not in {library}");
            assert!(dladdr(addr, &raw mut info) != 0, "dladdr {name}");
            let from = CStr::from_ptr(info.fname).to_string_lossy();
            assert!(
                from.ends_with(library),
                "{name} resolved to {from}, not {library}"
            );
            addr
        }
    }
}

/// One module of lanes, each a lazily resolved entry point in `library`.
macro_rules! dynamic_lanes {
    (
        $(#[$meta:meta])*
        mod $module:ident from $library:literal { $($name:ident($($arg:ident: $t:ty),+);)+ }
    ) => {
        $(#[$meta])*
        mod $module {
            $(
                pub fn $name($($arg: $t),+) -> f128 {
                    static ENTRY: std::sync::OnceLock<unsafe extern "C" fn($($t),+) -> f128> =
                        std::sync::OnceLock::new();
                    let call = *ENTRY.get_or_init(|| {
                        let addr = super::dynamic::entry($library, stringify!($name));
                        // SAFETY: the C prototype of every symbol listed here
                        // is `f128 name(f128, …)`.
                        unsafe {
                            core::mem::transmute::<
                                *mut core::ffi::c_void,
                                unsafe extern "C" fn($($t),+) -> f128,
                            >(addr)
                        }
                    });
                    // SAFETY: resolved out of the library's own symbol table.
                    unsafe { call($($arg),+) }
                }
            )+
        }
    };
}

dynamic_lanes! {
    /// glibc's `_Float128` entry points — the symbols nightly's `f128` methods
    /// call through `std::sys::cmath`.  glibc has no `rsqrtf128`.
    mod glibc from "libm.so.6" {
        sqrtf128(x: f128);
        cbrtf128(x: f128);
        hypotf128(x: f128, y: f128);
        expf128(x: f128);
        exp2f128(x: f128);
        exp10f128(x: f128);
        expm1f128(x: f128);
        logf128(x: f128);
        log2f128(x: f128);
        log10f128(x: f128);
        log1pf128(x: f128);
        atan2f128(y: f128, x: f128);
        atanf128(x: f128);
        asinf128(x: f128);
        acosf128(x: f128);
        sinf128(x: f128);
        cosf128(x: f128);
        tanf128(x: f128);
        powf128(x: f128, y: f128);
    }
}

dynamic_lanes! {
    /// GCC's libquadmath, which ships neither `rsqrtq` nor `exp10q`, so those
    /// lanes stay empty.
    mod quadmath from "libquadmath.so.0" {
        sqrtq(x: f128);
        cbrtq(x: f128);
        hypotq(x: f128, y: f128);
        expq(x: f128);
        exp2q(x: f128);
        expm1q(x: f128);
        logq(x: f128);
        log2q(x: f128);
        log10q(x: f128);
        log1pq(x: f128);
        atan2q(y: f128, x: f128);
        atanq(x: f128);
        asinq(x: f128);
        acosq(x: f128);
        sinq(x: f128);
        cosq(x: f128);
        tanq(x: f128);
        powq(x: f128, y: f128);
    }
}

// ---------------------------------------------------------------- sampling --

const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << 112) - 1;
const BIAS: i32 = 16383;

/// SplitMix64 hash, the same one `tests/all/common.rs` samples with.
fn mix64(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Two [`mix64`] words stacked into the 128 bits a binary128 draw needs.
fn mix128(i: u64) -> u128 {
    u128::from(mix64(i)) | u128::from(mix64(i ^ 0x9E37_79B9_7F4A_7C15)) << 64
}

/// The two independent words a bivariate sample consumes.
fn mix128_pair(i: u64) -> (u128, u128) {
    (mix128(i << 1), mix128(i << 1 | 1))
}

/// Uniform exponent field `1..=0x7FFE` and uniform fraction: any positive
/// normal, `benches/bench128.rs`'s `positive_normal`.
fn positive_normal(w: u128) -> f128 {
    f128::from_bits(((w >> 112 & 0x7FFF) % 0x7FFE + 1) << 112 | w & MANTISSA)
}

/// [`positive_normal`] wearing the word's sign bit.
fn normal(w: u128) -> f128 {
    f128::from_bits(w & SIGN | positive_normal(w).to_bits())
}

/// Random sign and fraction with the exponent uniform over `lo..=hi` — the
/// benches' `Exponents` band, so the accuracy population is the timed one.
fn band(w: u128, lo: i32, hi: i32) -> f128 {
    let span = (hi - lo + 1) as u128;
    let e = (w >> 112 & 0x7FFF) % span + (lo + BIAS) as u128;
    f128::from_bits(w & SIGN | e << 112 | w & MANTISSA)
}

/// [`band`] forced positive.  `powq`'s base deviates from the bench here: a
/// negative base makes `x^y` NaN for all but the integer `y` the band never
/// draws, and every such sample would be thrown away unmeasured.
fn positive_band(w: u128, lo: i32, hi: i32) -> f128 {
    f128::from_bits(band(w, lo, hi).to_bits() & !SIGN)
}

/// Value-uniform over the closed `[-1, 1]`, 113 bits of resolution.  Drawing
/// 114 bits and folding them onto the `2^113 + 1` point grid is what lets the
/// endpoints themselves come up at all; `(w >> 15) % 2^113` never reaches `+1`.
fn unit_interval(w: u128) -> f128 {
    ((w >> 14) % ((1u128 << 113) + 1)) as f128 / (1u128 << 112) as f128 - 1.0
}

/// `±(1 − δ)` with `δ` log-uniform over `[2^-112, 1)`: the corner a
/// value-uniform draw cannot reach, since it spends only `2^-e` of its samples
/// within `2^-e` of an endpoint.  This is where `asin`/`acos` cancel in
/// `√(1 − x²)` and where both competitors are at their worst, so half the
/// arc-function draws come from here.
fn near_endpoint(w: u128) -> f128 {
    let e = (w >> 112 & 0x7F) % 112 + 1;
    let delta = f128::from_bits(((BIAS as u128) - e) << 112 | w & MANTISSA);
    f128::from_bits(w & SIGN | (1.0 - delta).to_bits())
}

/// The `asin`/`acos` population: [`unit_interval`] and [`near_endpoint`] in
/// equal measure.  It is *not* the bench's `Exponents(-20..=-1)`, which never
/// leaves `|x| < 1/2` and so never touches the reflection at all.
fn arc_argument(i: u64) -> f128 {
    let w = mix128(i);
    if i & 1 == 0 {
        unit_interval(w)
    } else {
        near_endpoint(w)
    }
}

/// `log1p`'s band, `benches/log1pq.rs`'s `argument`: a random sign and
/// significand at an exponent uniform over `[2^-114, 2^14)`, capped at `2^0`
/// when negative so `x > −1` holds by construction — the largest magnitude in
/// the `2^-1` binade is `1 − 2^-113`.  No rejection, hence no sampler that can
/// fail to terminate.
fn log1p_argument(i: u64) -> f128 {
    let w = mix128(i);
    band(w, -114, if w & SIGN == 0 { 13 } else { -1 })
}

// ------------------------------------------------------------------ survey --

/// The magnitude at which rounding to binary128 overflows: the midpoint
/// `2^16384 − 2^16270` between `f128::MAX` and `2^16384`, which ties to the
/// even `2^16384` and so is already infinite.
fn overflow_threshold() -> Float {
    (Float::with_val(WORK, 1) << 16384i32) - (Float::with_val(WORK, 1) << 16270i32)
}

/// What one lane accumulated over the sample stream.
struct Stat {
    max: f64,
    /// The sample attaining `max`; the second slot is unused for a unary entry.
    at: (f128, f128),
    /// Samples whose error exceeded a half ulp.
    misses: u64,
    /// Samples the lane answered with the wrong kind of value entirely: NaN,
    /// the wrong infinity, an infinity where the answer is finite, or a finite
    /// value where the answer overflows.
    bogus: u64,
    /// Samples that produced a comparable error at all.
    counted: u64,
}

impl Stat {
    const NEW: Self = Self {
        max: 0.0,
        at: (0.0, 0.0),
        misses: 0,
        bogus: 0,
        counted: 0,
    };

    /// Fold one sample: `a` is the lane's answer, `v` the oracle's value.
    fn push(&mut self, at: (f128, f128), a: f128, v: &Float, ulp_exp: i32, overflow: &Float) {
        // Rounding `v` overflows exactly at `overflow`, and there ±∞ *is* the
        // correctly rounded answer.  Everywhere else a non-finite answer — and
        // there, a finite one — is a failure no ulp count can express.
        let saturates = v.cmp_abs(overflow) != Some(Ordering::Less);
        if saturates || !a.is_finite() {
            let right = saturates && a.is_infinite() && (a > 0.0) == v.is_sign_positive();
            self.counted += u64::from(right);
            self.bogus += u64::from(!right);
            return;
        }

        // `v` is a 300-bit value and `a` a 113-bit one at most an ulp or so
        // away, so their difference spans well under `WORK` bits and lands
        // exactly; the scaling by a power of two adds nothing.  A lane wrong by
        // binades would round the difference, and would still read as the
        // enormous error it is.
        let mut d = Float::with_val(WORK, a);
        d -= v;
        d.abs_mut();
        d >>= ulp_exp;
        let err = d.to_f64();

        self.counted += 1;
        self.misses += u64::from(err > 0.5);
        if err > self.max {
            self.max = err;
            self.at = at;
        }
    }
}

/// The three lanes of one arity, in [`LANES`] order; `None` where the library
/// ships no such symbol.
type Lanes<F> = [Option<F>; 3];
type UnaryLane = fn(f128) -> f128;
type BinaryLane = fn(f128, f128) -> f128;

/// A survey entry is its sampler, its MPFR oracle, and the three lanes; the
/// arity is the only structural difference, so it is the only enum.
enum Kind {
    Unary {
        sample: fn(u64) -> f128,
        oracle: fn(&Float) -> Float,
        lanes: Lanes<UnaryLane>,
    },
    Binary {
        sample: fn(u64) -> (f128, f128),
        oracle: fn(&Float, &Float) -> Float,
        lanes: Lanes<BinaryLane>,
    },
}

struct Entry {
    name: &'static str,
    kind: Kind,
}

/// One entry's outcome: the per-lane statistics and the samples no lane could
/// be scored on, because the exact value is zero or not finite.
struct Outcome {
    stats: Lanes<Stat>,
    dropped: u64,
}

fn unary(
    name: &'static str,
    sample: fn(u64) -> f128,
    oracle: fn(&Float) -> Float,
    lanes: Lanes<UnaryLane>,
) -> Entry {
    let kind = Kind::Unary {
        sample,
        oracle,
        lanes,
    };
    Entry { name, kind }
}

fn binary(
    name: &'static str,
    sample: fn(u64) -> (f128, f128),
    oracle: fn(&Float, &Float) -> Float,
    lanes: Lanes<BinaryLane>,
) -> Entry {
    let kind = Kind::Binary {
        sample,
        oracle,
        lanes,
    };
    Entry { name, kind }
}

/// Run `n` samples of one entry through every live lane.
fn survey(entry: &Entry, n: u64) -> Outcome {
    // Salt the stream per function so two entries sharing a band do not share
    // their draws.
    let salt = entry.name.bytes().fold(0, |h, b| mix64(h ^ u64::from(b)));
    let overflow = overflow_threshold();

    let mut stats = match &entry.kind {
        Kind::Unary { lanes, .. } => lanes.map(|lane| lane.map(|_| Stat::NEW)),
        Kind::Binary { lanes, .. } => lanes.map(|lane| lane.map(|_| Stat::NEW)),
    };
    let mut dropped = 0;

    for i in 0..n {
        let i = salt.wrapping_add(i);
        let (at, v) = match &entry.kind {
            Kind::Unary { sample, oracle, .. } => {
                let x = sample(i);
                ((x, 0.0), oracle(&Float::with_val(WORK, x)))
            }
            Kind::Binary { sample, oracle, .. } => {
                let (x, y) = sample(i);
                let v = oracle(&Float::with_val(WORK, x), &Float::with_val(WORK, y));
                ((x, y), v)
            }
        };

        // `get_exp` is `None` exactly for zero, infinity and NaN — the values
        // that carry no ulp.  It reports `k` with `v = m·2^k`, `m ∈ [0.5, 1)`,
        // so the binade exponent is `k − 1`.
        let Some(k) = v.get_exp() else {
            dropped += 1;
            continue;
        };
        let ulp_exp = (k - 113).max(SUBNORMAL_ULP);

        for (lane, stat) in stats.iter_mut().enumerate() {
            let Some(stat) = stat.as_mut() else { continue };
            let a = match &entry.kind {
                Kind::Unary { lanes, .. } => lanes[lane].unwrap()(at.0),
                Kind::Binary { lanes, .. } => lanes[lane].unwrap()(at.0, at.1),
            };
            stat.push(at, a, &v, ulp_exp, &overflow);
        }
    }
    Outcome { stats, dropped }
}

// ------------------------------------------------------------------- table --

fn table() -> Vec<Entry> {
    vec![
        unary(
            "sqrtq",
            |i| positive_normal(mix128(i)),
            |x| Float::with_val(WORK, x).sqrt(),
            [
                Some(metallic::sqrtq),
                Some(glibc::sqrtf128),
                Some(quadmath::sqrtq),
            ],
        ),
        unary(
            "rsqrtq",
            |i| positive_normal(mix128(i)),
            |x| Float::with_val(WORK, x).recip_sqrt(),
            [Some(metallic::rsqrtq), None, None],
        ),
        unary(
            "cbrtq",
            |i| normal(mix128(i)),
            |x| Float::with_val(WORK, x).cbrt(),
            [
                Some(metallic::cbrtq),
                Some(glibc::cbrtf128),
                Some(quadmath::cbrtq),
            ],
        ),
        binary(
            "hypotq",
            |i| {
                let (u, v) = mix128_pair(i);
                (band(u, -12, 12), band(v, -12, 12))
            },
            |x, y| Float::with_val(WORK, x).hypot(y),
            [
                Some(metallic::hypotq),
                Some(glibc::hypotf128),
                Some(quadmath::hypotq),
            ],
        ),
        unary(
            "expq",
            |i| band(mix128(i), -120, 13),
            |x| Float::with_val(WORK, x).exp(),
            [
                Some(metallic::expq),
                Some(glibc::expf128),
                Some(quadmath::expq),
            ],
        ),
        unary(
            "exp2q",
            |i| band(mix128(i), -120, 13),
            |x| Float::with_val(WORK, x).exp2(),
            [
                Some(metallic::exp2q),
                Some(glibc::exp2f128),
                Some(quadmath::exp2q),
            ],
        ),
        unary(
            "exp10q",
            |i| band(mix128(i), -120, 13),
            |x| Float::with_val(WORK, x).exp10(),
            [Some(metallic::exp10q), Some(glibc::exp10f128), None],
        ),
        unary(
            "expm1q",
            |i| band(mix128(i), -114, 13),
            |x| Float::with_val(WORK, x).exp_m1(),
            [
                Some(metallic::expm1q),
                Some(glibc::expm1f128),
                Some(quadmath::expm1q),
            ],
        ),
        unary(
            "logq",
            |i| positive_normal(mix128(i)),
            |x| Float::with_val(WORK, x).ln(),
            [
                Some(metallic::logq),
                Some(glibc::logf128),
                Some(quadmath::logq),
            ],
        ),
        unary(
            "log2q",
            |i| positive_normal(mix128(i)),
            |x| Float::with_val(WORK, x).log2(),
            [
                Some(metallic::log2q),
                Some(glibc::log2f128),
                Some(quadmath::log2q),
            ],
        ),
        unary(
            "log10q",
            |i| positive_normal(mix128(i)),
            |x| Float::with_val(WORK, x).log10(),
            [
                Some(metallic::log10q),
                Some(glibc::log10f128),
                Some(quadmath::log10q),
            ],
        ),
        unary(
            "log1pq",
            log1p_argument,
            |x| Float::with_val(WORK, x).ln_1p(),
            [
                Some(metallic::log1pq),
                Some(glibc::log1pf128),
                Some(quadmath::log1pq),
            ],
        ),
        binary(
            "atan2q",
            |i| {
                let (u, v) = mix128_pair(i);
                (band(u, -20, 20), band(v, -20, 20))
            },
            |y, x| Float::with_val(WORK, y).atan2(x),
            [
                Some(metallic::atan2q),
                Some(glibc::atan2f128),
                Some(quadmath::atan2q),
            ],
        ),
        unary(
            "atanq",
            |i| band(mix128(i), -20, 20),
            |x| Float::with_val(WORK, x).atan(),
            [
                Some(metallic::atanq),
                Some(glibc::atanf128),
                Some(quadmath::atanq),
            ],
        ),
        unary(
            "asinq",
            arc_argument,
            |x| Float::with_val(WORK, x).asin(),
            [
                Some(metallic::asinq),
                Some(glibc::asinf128),
                Some(quadmath::asinq),
            ],
        ),
        unary(
            "acosq",
            arc_argument,
            |x| Float::with_val(WORK, x).acos(),
            [
                Some(metallic::acosq),
                Some(glibc::acosf128),
                Some(quadmath::acosq),
            ],
        ),
        unary(
            "sinq",
            |i| band(mix128(i), -20, 20),
            |x| Float::with_val(WORK, x).sin(),
            [
                Some(metallic::sinq),
                Some(glibc::sinf128),
                Some(quadmath::sinq),
            ],
        ),
        unary(
            "cosq",
            |i| band(mix128(i), -20, 20),
            |x| Float::with_val(WORK, x).cos(),
            [
                Some(metallic::cosq),
                Some(glibc::cosf128),
                Some(quadmath::cosq),
            ],
        ),
        unary(
            "tanq",
            |i| band(mix128(i), -20, 20),
            |x| Float::with_val(WORK, x).tan(),
            [
                Some(metallic::tanq),
                Some(glibc::tanf128),
                Some(quadmath::tanq),
            ],
        ),
        binary(
            "powq",
            |i| {
                let (u, v) = mix128_pair(i);
                (positive_band(u, -16, 16), band(v, -16, 8))
            },
            |x, y| Float::with_val(WORK, x).pow(y),
            [
                Some(metallic::powq),
                Some(glibc::powf128),
                Some(quadmath::powq),
            ],
        ),
    ]
}

// -------------------------------------------------------------------- main --

/// Pin the metric against cases whose answer is known by construction: an
/// exact midpoint reads exactly `0.5` on either side of a binade boundary and
/// on the subnormal grid, and an `f128` reaches MPFR unrounded.  A factor of
/// two anywhere in [`survey`]'s `ulp_exp` shows up here as `0.25` or `1.0`.
fn self_check() {
    let odd = u128::MAX >> 15;
    assert_eq!(
        Float::with_val(WORK, odd as f128),
        Float::with_val(WORK, odd),
        "f128 → MPFR is lossy"
    );

    let overflow = overflow_threshold();
    let err = |v: Float, a: f128| {
        let k = v.get_exp().expect("a finite nonzero midpoint");
        let mut stat = Stat::NEW;
        stat.push((a, 0.0), a, &v, (k - 113).max(SUBNORMAL_ULP), &overflow);
        stat.max
    };

    let one = || Float::with_val(WORK, 1);
    // 1 + 2^-113, the midpoint above 1, where the ulp is 2^-112.
    assert_eq!(err(one() + (one() >> 113i32), 1.0), 0.5, "midpoint above 1");
    // 1 − 2^-114, the midpoint below 1, one binade down, where it is 2^-113.
    assert_eq!(err(one() - (one() >> 114i32), 1.0), 0.5, "midpoint below 1");
    // 1.5·2^-16494, the midpoint above the least subnormal, where it floors.
    let tiny = Float::with_val(WORK, 3) >> 16495i32;
    assert_eq!(err(tiny, f128::from_bits(1)), 0.5, "subnormal midpoint");

    // The overflow boundary: ±∞ is right at and above it, wrong below.
    let mut stat = Stat::NEW;
    let below = Float::with_val(WORK, f128::MAX);
    stat.push((0.0, 0.0), f128::INFINITY, &overflow, 0, &overflow);
    stat.push((0.0, 0.0), f128::INFINITY, &below, 0, &overflow);
    assert_eq!((stat.counted, stat.bogus), (1, 1), "overflow boundary");
}

fn main() {
    self_check();

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (n, only) = match argv.split_first() {
        Some((first, rest)) if first.parse::<u64>().is_ok() => {
            (first.parse().expect("sample count"), rest)
        }
        _ => (20_000, &argv[..]),
    };

    let entries: Vec<Entry> = table()
        .into_iter()
        .filter(|e| only.is_empty() || only.iter().any(|s| s == e.name))
        .collect();

    println!("f128 ulp survey — {n} samples per function, deterministic SplitMix64 draws");
    println!("err = |a − v| / 2^max(E − 112, −16494), v the MPFR value at {WORK} bits and");
    println!("E its binade exponent (v ∈ [2^E, 2^(E+1))); the floor is binary128's subnormal");
    println!("ulp.  A correctly rounded lane never exceeds 0.5, which is what the metallic");
    println!("column checks: it validates the harness, it is not a result.\n");

    println!(
        "{:<8}{:>12}{:>12}{:>12}",
        "function", LANES[0], LANES[1], LANES[2]
    );

    let mut results = Vec::new();
    for entry in entries {
        let outcome = survey(&entry, n);
        print!("{:<8}", entry.name);
        for stat in &outcome.stats {
            match stat {
                Some(stat) => print!("{:>12.4}", stat.max),
                None => print!("{:>12}", "-"),
            }
        }
        println!();
        std::io::stdout().flush().expect("stdout");
        results.push((entry, outcome));
    }

    println!("\nworst case per lane (Debug prints the binary128 bit pattern):");
    for (entry, outcome) in &results {
        for (lane, stat) in LANES.iter().zip(&outcome.stats) {
            let Some(stat) = stat else { continue };
            let (x, y) = stat.at;
            let at = match entry.kind {
                Kind::Unary { .. } => format!("{x:?}"),
                Kind::Binary { .. } => format!("{x:?}, {y:?}"),
            };
            let wrong = 100.0 * stat.misses as f64 / stat.counted as f64;
            println!(
                "{:<8}{lane:<10}max {:.4}  wrong {wrong:7.3}% ({}/{})  bogus {:<5}  at {at}",
                entry.name, stat.max, stat.misses, stat.counted, stat.bogus,
            );
        }
        if outcome.dropped != 0 {
            println!(
                "{:<8}{:<10}{} samples had no exact value to charge an ulp against",
                entry.name, "(dropped)", outcome.dropped
            );
        }
    }
}
