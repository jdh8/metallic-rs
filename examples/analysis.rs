#![cfg_attr(feature = "f128", feature(f128))]
//! Probes for `tools/analysis.py`: out-of-line C-ABI entry points for
//! `llvm-mca`, and a sweep that a coverage-instrumented build turns into
//! accurate-leg counts.
//!
//! Every public function gets one `#[inline(never)] extern "C"` wrapper
//! exported as `metallic_<name>` — an unmangled label the walker in
//! `tools/analysis.py asm` starts from in the emitted assembly, and that
//! `llvm-mca` then prices.  (A bare `exp` or `fma` would interpose libm's own
//! for the CORE-MATH side of the comparison, hence the prefix.)
//!
//! The same wrapper is what `analysis <name> <samples>` calls, so the analysed
//! code is the exercised code: the sweep runs it — and `core_math::<name>`
//! beside it where CORE-MATH binds the function — over finite inputs, feeding
//! every result through `std::hint::black_box`.  Built with `-C instrument-coverage`
//! (Rust) and `-fprofile-instr-generate -fcoverage-mapping` (CORE-MATH's C),
//! the run leaves behind a profile from which `tools/analysis.py legs` reads
//! how often each side entered its accurate leg.
//!
//! ```text
//! analysis list                # one `<name> <prec> <arity> <core_math>` per probe
//! analysis <name> <samples>    # sweep; prints `finite=<n>`
//! analysis call <name> <x> <y> <z>   # both sides once, on one argument
//! python3 tools/analysis.py all   # the full report, ANALYSIS.md
//! ```
//!
//! `call` is what `tools/analysis.py asm` traces under gdb to find the fast
//! path: the instructions one representative argument executes, on both
//! sides.  The numbers are converted to the probe's precision; `y` is the
//! second operand, `z` the third of `fma*`; `ldexp*` reads its exponent from
//! `y`.
//!
//! Inputs are representation-uniform over the finite values — every finite bit
//! pattern equally likely, non-finite draws redrawn — from a [`SplitMix64`]
//! reseeded per run, so a rerun repeats the sweep.  `samples == 0` is the
//! exhaustive sweep of all 2³² patterns and exists only for the f32 univariate
//! probes; anywhere else it is refused with exit status 2.

use std::process::exit;

/// `sincos*`'s two results as a C-ABI aggregate.
#[repr(C)]
pub struct Pair<T>(pub T, pub T);

/// `frexp*`'s significand and exponent as a C-ABI aggregate.
#[repr(C)]
pub struct Frexp<T>(pub T, pub i32);

/// One public function: the row `list` prints, and the sweep `run` drives.
struct Probe {
    /// Public name, e.g. `exp`.
    name: &'static str,
    /// `"f32"`, `"f64"` or `"f128"`.
    prec: &'static str,
    /// Operands drawn per sample: `ldexp*` counts as 2, `sincos*`/`frexp*` as 1.
    arity: u8,
    /// Whether `core_math::<name>` exists and is swept alongside.
    core_math: bool,
    /// `sweep(samples)`, returning the number of finite inputs evaluated.
    run: fn(u64) -> u64,
    /// The wrapper, then `core_math::<name>` if bound, once on `(x, y, z)`.
    once: fn(f64, f64, f64),
}

// -------------------------------------------------------------------- draws --

/// SplitMix64 with a fixed seed, so every sweep replays the same stream.
struct SplitMix64(u64);

impl SplitMix64 {
    const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

    const fn new() -> Self {
        Self(Self::SEED)
    }

    fn word(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// A precision the sweep can draw finite values of.
trait Draw: Copy {
    /// A finite value, every finite bit pattern equally likely.
    fn draw(rng: &mut SplitMix64) -> Self;

    /// Every finite value in bit order — only f32 is small enough.
    fn all_finite() -> Option<impl Iterator<Item = Self>>;
}

impl Draw for f32 {
    /// The high 32 bits of one word.
    fn draw(rng: &mut SplitMix64) -> Self {
        loop {
            let x = Self::from_bits((rng.word() >> 32) as u32);
            if x.is_finite() {
                return x;
            }
        }
    }

    fn all_finite() -> Option<impl Iterator<Item = Self>> {
        Some(
            (0..=u32::MAX)
                .map(Self::from_bits)
                .filter(|x| x.is_finite()),
        )
    }
}

impl Draw for f64 {
    /// One word.
    fn draw(rng: &mut SplitMix64) -> Self {
        loop {
            let x = Self::from_bits(rng.word());
            if x.is_finite() {
                return x;
            }
        }
    }

    fn all_finite() -> Option<impl Iterator<Item = Self>> {
        None::<std::iter::Empty<Self>>
    }
}

#[cfg(feature = "f128")]
impl Draw for f128 {
    /// Two words, the first one low.
    fn draw(rng: &mut SplitMix64) -> Self {
        loop {
            let low = rng.word();
            let high = rng.word();
            let x = Self::from_bits(u128::from(low) | (u128::from(high) << 64));
            if x.is_finite() {
                return x;
            }
        }
    }

    fn all_finite() -> Option<impl Iterator<Item = Self>> {
        None::<std::iter::Empty<Self>>
    }
}

// ------------------------------------------------------------------- sweeps --

/// One sweep per wrapper shape.  Each calls the `metallic_<name>` wrapper and,
/// when given, the CORE-MATH twin on the same operands, and black-boxes every
/// result so nothing is optimised away.
mod sweep {
    use super::{Draw, Frexp, Pair, SplitMix64};
    use std::hint::black_box;
    use std::ops::Add;

    /// `samples == 0` outside the f32 univariate probes.
    pub fn refuse_exhaustive() -> ! {
        eprintln!("analysis: samples = 0 (exhaustive) is valid only for f32 univariate probes");
        std::process::exit(2)
    }

    /// `samples` fresh draws from a reseeded generator, each handed to `eval`.
    fn sampled<A>(
        samples: u64,
        draw: impl Fn(&mut SplitMix64) -> A,
        mut eval: impl FnMut(A),
    ) -> u64 {
        if samples == 0 {
            refuse_exhaustive();
        }
        let mut rng = SplitMix64::new();
        for _ in 0..samples {
            eval(draw(&mut rng));
        }
        samples
    }

    /// One operand per sample; `samples == 0` walks every finite value instead.
    fn univariate<T: Draw>(samples: u64, mut eval: impl FnMut(T)) -> u64 {
        if samples != 0 {
            return sampled(samples, T::draw, eval);
        }
        let Some(all) = T::all_finite() else {
            refuse_exhaustive()
        };
        let mut finite = 0;
        for x in all {
            eval(x);
            finite += 1;
        }
        finite
    }

    fn pair<T: Draw>(rng: &mut SplitMix64) -> (T, T) {
        (T::draw(rng), T::draw(rng))
    }

    fn triple<T: Draw>(rng: &mut SplitMix64) -> (T, T, T) {
        (T::draw(rng), T::draw(rng), T::draw(rng))
    }

    /// A value and an `ldexp` exponent uniform over `-2200..=2200`, the
    /// benches' band.
    fn scaled<T: Draw>(rng: &mut SplitMix64) -> (T, i32) {
        let x = T::draw(rng);
        // `% 4401` leaves a value that always fits an `i32`.
        let n = (rng.word() % 4401) as i32 - 2200;
        (x, n)
    }

    pub fn unary<T: Draw>(samples: u64, m: extern "C" fn(T) -> T, c: Option<fn(T) -> T>) -> u64 {
        univariate(samples, |x| {
            black_box(m(x));
            if let Some(c) = c {
                black_box(c(x));
            }
        })
    }

    /// Replay the tanpi draws in its tiny band. Its database gate merges with
    /// the polynomial fallbacks, so aggregate coverage cannot separate them.
    pub fn tanpi_tiny(samples: u64) -> u64 {
        univariate(samples, |x: f64| {
            if x.abs() < 1.0 / ((1_u64 << 53) as f64) {
                black_box(core_math::tanpi(x));
            }
        })
    }

    pub fn binary<T: Draw>(
        samples: u64,
        m: extern "C" fn(T, T) -> T,
        c: Option<fn(T, T) -> T>,
    ) -> u64 {
        sampled(samples, pair::<T>, |(x, y)| {
            black_box(m(x, y));
            if let Some(c) = c {
                black_box(c(x, y));
            }
        })
    }

    pub fn ternary<T: Draw>(samples: u64, m: extern "C" fn(T, T, T) -> T) -> u64 {
        sampled(samples, triple::<T>, |(x, y, a)| {
            black_box(m(x, y, a));
        })
    }

    /// The CORE-MATH side folds its pair as `s + c`.
    pub fn sincos<T: Draw + Add<Output = T>>(
        samples: u64,
        m: extern "C" fn(T) -> Pair<T>,
        c: Option<fn(T) -> (T, T)>,
    ) -> u64 {
        univariate(samples, |x| {
            black_box(m(x));
            if let Some(c) = c {
                let (s, k) = c(x);
                black_box(s + k);
            }
        })
    }

    /// Only the significand is kept.
    pub fn frexp<T: Draw>(samples: u64, m: extern "C" fn(T) -> Frexp<T>) -> u64 {
        univariate(samples, |x| {
            black_box(m(x).0);
        })
    }

    pub fn ldexp<T: Draw>(samples: u64, m: extern "C" fn(T, i32) -> T) -> u64 {
        sampled(samples, scaled::<T>, |(x, n)| {
            black_box(m(x, n));
        })
    }
}

// ------------------------------------------------------------------- probes --

/// The out-of-line C-ABI wrapper of one shape: `metallic::$name` behind the
/// unmangled label `$wrap`.  (`export_name = concat!(…)` is not accepted, so
/// the label is spelled out at every call site.)
macro_rules! wrapper {
    (unary, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t) -> $t {
            metallic::$name(x)
        }
    };
    (binary, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t, y: $t) -> $t {
            metallic::$name(x, y)
        }
    };
    (ternary, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t, y: $t, a: $t) -> $t {
            metallic::$name(x, y, a)
        }
    };
    (sincos, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t) -> Pair<$t> {
            let (s, c) = metallic::$name(x);
            Pair(s, c)
        }
    };
    (frexp, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t) -> Frexp<$t> {
            let (m, e) = metallic::$name(x);
            Frexp(m, e)
        }
    };
    (ldexp, $t:ty, $name:ident, $wrap:ident) => {
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn $wrap(x: $t, n: i32) -> $t {
            metallic::$name(x, n)
        }
    };
}

/// `Some(core_math::$name)` when the probe is marked `core_math`, else `None`.
macro_rules! oracle {
    ($sig:ty; $name:ident) => {
        None::<$sig>
    };
    ($sig:ty; $name:ident, core_math) => {
        Some(core_math::$name as $sig)
    };
}

/// The sweep of one probe as a plain `fn(u64) -> u64`.
macro_rules! run {
    (unary, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |samples| sweep::unary::<$t>(samples, $wrap, oracle!(fn($t) -> $t; $name $(, $cm)?))
    };
    (binary, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |samples| sweep::binary::<$t>(samples, $wrap, oracle!(fn($t, $t) -> $t; $name $(, $cm)?))
    };
    (ternary, $t:ty, $name:ident, $wrap:ident) => {
        |samples| sweep::ternary::<$t>(samples, $wrap)
    };
    (sincos, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |samples| sweep::sincos::<$t>(samples, $wrap, oracle!(fn($t) -> ($t, $t); $name $(, $cm)?))
    };
    (frexp, $t:ty, $name:ident, $wrap:ident) => {
        |samples| sweep::frexp::<$t>(samples, $wrap)
    };
    (ldexp, $t:ty, $name:ident, $wrap:ident) => {
        |samples| sweep::ldexp::<$t>(samples, $wrap)
    };
}

/// One call of the wrapper and of the oracle as a plain `fn(f64, f64, f64)`.
macro_rules! once {
    (unary, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |x, _, _| {
            std::hint::black_box($wrap(x as $t));
            if let Some(o) = oracle!(fn($t) -> $t; $name $(, $cm)?) {
                std::hint::black_box(o(x as $t));
            }
        }
    };
    (binary, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |x, y, _| {
            std::hint::black_box($wrap(x as $t, y as $t));
            if let Some(o) = oracle!(fn($t, $t) -> $t; $name $(, $cm)?) {
                std::hint::black_box(o(x as $t, y as $t));
            }
        }
    };
    (ternary, $t:ty, $name:ident, $wrap:ident) => {
        |x, y, z| {
            std::hint::black_box($wrap(x as $t, y as $t, z as $t));
        }
    };
    (sincos, $t:ty, $name:ident, $wrap:ident $(, $cm:ident)?) => {
        |x, _, _| {
            std::hint::black_box($wrap(x as $t));
            if let Some(o) = oracle!(fn($t) -> ($t, $t); $name $(, $cm)?) {
                std::hint::black_box(o(x as $t));
            }
        }
    };
    (frexp, $t:ty, $name:ident, $wrap:ident) => {
        |x, _, _| {
            std::hint::black_box($wrap(x as $t));
        }
    };
    (ldexp, $t:ty, $name:ident, $wrap:ident) => {
        |x, y, _| {
            std::hint::black_box($wrap(x as $t, y as i32));
        }
    };
}

macro_rules! arity {
    (unary) => {
        1
    };
    (binary) => {
        2
    };
    (ternary) => {
        3
    };
    (sincos) => {
        1
    };
    (frexp) => {
        1
    };
    (ldexp) => {
        2
    };
}

macro_rules! has {
    () => {
        false
    };
    (core_math) => {
        true
    };
}

/// One precision's block: emits every wrapper and the `fn $probes() -> Vec<Probe>`
/// listing them in the order written.
///
/// Each line is `<name> = <wrapper>: <shape> [+ core_math];`, the wrapper
/// label spelled out because `export_name` cannot be built by `concat!`.
macro_rules! probes {
    ($t:ty => $probes:ident { $($name:ident = $wrap:ident : $shape:ident $(+ $cm:ident)?;)+ }) => {
        $(wrapper!($shape, $t, $name, $wrap);)+

        fn $probes() -> Vec<Probe> {
            vec![$(Probe {
                name: stringify!($name),
                prec: stringify!($t),
                arity: arity!($shape),
                core_math: has!($($cm)?),
                run: run!($shape, $t, $name, $wrap $(, $cm)?),
                once: once!($shape, $t, $name, $wrap $(, $cm)?),
            }),+]
        }
    };
}

probes! {
    f32 => probes_f32 {
        acosf = metallic_acosf: unary + core_math;
        acoshf = metallic_acoshf: unary + core_math;
        acospif = metallic_acospif: unary + core_math;
        asinf = metallic_asinf: unary + core_math;
        asinhf = metallic_asinhf: unary + core_math;
        asinpif = metallic_asinpif: unary + core_math;
        atan2f = metallic_atan2f: binary + core_math;
        atan2pif = metallic_atan2pif: binary + core_math;
        atanf = metallic_atanf: unary + core_math;
        atanhf = metallic_atanhf: unary + core_math;
        atanpif = metallic_atanpif: unary + core_math;
        cbrtf = metallic_cbrtf: unary + core_math;
        compoundf = metallic_compoundf: binary + core_math;
        cosf = metallic_cosf: unary + core_math;
        coshf = metallic_coshf: unary + core_math;
        cospif = metallic_cospif: unary + core_math;
        erfcf = metallic_erfcf: unary + core_math;
        erff = metallic_erff: unary + core_math;
        exp2f = metallic_exp2f: unary + core_math;
        exp2m1f = metallic_exp2m1f: unary + core_math;
        exp10f = metallic_exp10f: unary + core_math;
        exp10m1f = metallic_exp10m1f: unary + core_math;
        expf = metallic_expf: unary + core_math;
        expm1f = metallic_expm1f: unary + core_math;
        fmaf = metallic_fmaf: ternary;
        frexpf = metallic_frexpf: frexp;
        hypotf = metallic_hypotf: binary + core_math;
        ldexpf = metallic_ldexpf: ldexp;
        lgammaf = metallic_lgammaf: unary + core_math;
        log1pf = metallic_log1pf: unary + core_math;
        log2f = metallic_log2f: unary + core_math;
        log2p1f = metallic_log2p1f: unary + core_math;
        log10f = metallic_log10f: unary + core_math;
        log10p1f = metallic_log10p1f: unary + core_math;
        logf = metallic_logf: unary + core_math;
        powf = metallic_powf: binary + core_math;
        roundf = metallic_roundf: unary;
        rsqrtf = metallic_rsqrtf: unary + core_math;
        sincosf = metallic_sincosf: sincos + core_math;
        sinf = metallic_sinf: unary + core_math;
        sinhf = metallic_sinhf: unary + core_math;
        sinpif = metallic_sinpif: unary + core_math;
        tanf = metallic_tanf: unary + core_math;
        tanhf = metallic_tanhf: unary + core_math;
        tanpif = metallic_tanpif: unary + core_math;
        tgammaf = metallic_tgammaf: unary + core_math;
    }
}

probes! {
    f64 => probes_f64 {
        acos = metallic_acos: unary + core_math;
        acosh = metallic_acosh: unary + core_math;
        acospi = metallic_acospi: unary + core_math;
        asin = metallic_asin: unary + core_math;
        asinh = metallic_asinh: unary + core_math;
        asinpi = metallic_asinpi: unary + core_math;
        atan = metallic_atan: unary + core_math;
        atan2 = metallic_atan2: binary + core_math;
        atan2pi = metallic_atan2pi: binary + core_math;
        atanh = metallic_atanh: unary + core_math;
        atanpi = metallic_atanpi: unary + core_math;
        cbrt = metallic_cbrt: unary + core_math;
        compound = metallic_compound: binary;
        cos = metallic_cos: unary + core_math;
        cosh = metallic_cosh: unary + core_math;
        cospi = metallic_cospi: unary + core_math;
        erf = metallic_erf: unary + core_math;
        erfc = metallic_erfc: unary + core_math;
        exp = metallic_exp: unary + core_math;
        exp2 = metallic_exp2: unary + core_math;
        exp2m1 = metallic_exp2m1: unary + core_math;
        exp10 = metallic_exp10: unary + core_math;
        exp10m1 = metallic_exp10m1: unary + core_math;
        expm1 = metallic_expm1: unary + core_math;
        fma = metallic_fma: ternary;
        frexp = metallic_frexp: frexp;
        hypot = metallic_hypot: binary + core_math;
        ldexp = metallic_ldexp: ldexp;
        lgamma = metallic_lgamma: unary + core_math;
        log = metallic_log: unary + core_math;
        log1p = metallic_log1p: unary + core_math;
        log2 = metallic_log2: unary + core_math;
        log2p1 = metallic_log2p1: unary + core_math;
        log10 = metallic_log10: unary + core_math;
        log10p1 = metallic_log10p1: unary + core_math;
        pow = metallic_pow: binary + core_math;
        round = metallic_round: unary;
        rsqrt = metallic_rsqrt: unary + core_math;
        sin = metallic_sin: unary + core_math;
        sincos = metallic_sincos: sincos + core_math;
        sinh = metallic_sinh: unary + core_math;
        sinpi = metallic_sinpi: unary + core_math;
        tan = metallic_tan: unary + core_math;
        tanh = metallic_tanh: unary + core_math;
        tanpi = metallic_tanpi: unary + core_math;
        tgamma = metallic_tgamma: unary + core_math;
    }
}

#[cfg(feature = "f128")]
probes! {
    f128 => probes_f128 {
        acosq = metallic_acosq: unary + core_math;
        asinq = metallic_asinq: unary + core_math;
        atan2q = metallic_atan2q: binary + core_math;
        atanq = metallic_atanq: unary + core_math;
        cbrtq = metallic_cbrtq: unary + core_math;
        cosq = metallic_cosq: unary;
        exp2q = metallic_exp2q: unary + core_math;
        exp10q = metallic_exp10q: unary + core_math;
        expm1q = metallic_expm1q: unary + core_math;
        expq = metallic_expq: unary + core_math;
        hypotq = metallic_hypotq: binary + core_math;
        log1pq = metallic_log1pq: unary;
        log2q = metallic_log2q: unary;
        log10q = metallic_log10q: unary;
        logq = metallic_logq: unary + core_math;
        powq = metallic_powq: binary;
        rsqrtq = metallic_rsqrtq: unary + core_math;
        sinq = metallic_sinq: unary;
        sqrtq = metallic_sqrtq: unary + core_math;
        tanq = metallic_tanq: unary;
    }
}

/// All probes: f32, then f64, then (with `f128`) binary128.
fn probes() -> Vec<Probe> {
    let mut v = probes_f32();
    v.extend(probes_f64());
    #[cfg(feature = "f128")]
    v.extend(probes_f128());
    v
}

// ---------------------------------------------------------------------- cli --

fn usage() -> ! {
    eprintln!(
        "usage: analysis list\n       analysis <name> <samples>    (0 = exhaustive; f32 univariate only)\n       analysis call <name> <x> <y> <z>"
    );
    exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let probes = probes();
    match args.as_slice() {
        [list] if list == "list" => {
            for p in &probes {
                println!(
                    "{} {} {} {}",
                    p.name,
                    p.prec,
                    p.arity,
                    u8::from(p.core_math)
                );
            }
        }
        [call, name, x, y, z] if call == "call" => {
            let (Ok(x), Ok(y), Ok(z)) = (x.parse::<f64>(), y.parse::<f64>(), z.parse::<f64>())
            else {
                usage()
            };
            let Some(p) = probes.iter().find(|p| p.name == name) else {
                eprintln!("analysis: unknown function `{name}`; see `analysis list`");
                exit(2)
            };
            (p.once)(x, y, z);
        }
        [name, samples] => {
            let Ok(samples) = samples.parse::<u64>() else {
                usage()
            };
            if name == "tanpi-tiny" {
                println!("finite={}", sweep::tanpi_tiny(samples));
                return;
            }
            let Some(p) = probes.iter().find(|p| p.name == name) else {
                eprintln!("analysis: unknown function `{name}`; see `analysis list`");
                exit(2)
            };
            if samples == 0 && !(p.prec == "f32" && p.arity == 1) {
                sweep::refuse_exhaustive();
            }
            println!("finite={}", (p.run)(samples));
        }
        _ => usage(),
    }
}
