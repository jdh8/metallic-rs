mod atanh;
mod cbrt;
mod exp;
mod exp10;
mod exp2;
mod exp_m1;
mod frexp;
mod hypot;
mod ldexp;
mod ln;
mod ln_1p;
mod log;
mod log10;
mod log2;
mod powf;
mod round;

criterion::criterion_main!(
    atanh::benches,
    cbrt::benches,
    exp::benches,
    exp2::benches,
    exp10::benches,
    exp_m1::benches,
    frexp::benches,
    hypot::benches,
    ldexp::benches,
    ln::benches,
    ln_1p::benches,
    log::benches,
    log10::benches,
    log2::benches,
    powf::benches,
    round::benches,
);

#[macro_export]
macro_rules! bench {
    ($bench:expr, $criterion:expr, $f:expr) => {
        $bench($criterion, stringify!($f), $f);
    };
}
