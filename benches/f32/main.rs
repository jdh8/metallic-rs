mod cbrt;
mod exp;
mod exp10;
mod exp2;
mod exp_m1;
mod frexp;
mod ldexp;

criterion::criterion_main!(
    cbrt::benches,
    exp::benches,
    exp2::benches,
    exp10::benches,
    exp_m1::benches,
    frexp::benches,
    ldexp::benches
);

#[macro_export]
macro_rules! bench {
    ($bench:expr, $criterion:expr, $f:expr) => {
        $bench($criterion, stringify!($f), $f);
    };
}
