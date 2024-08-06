mod exp;
mod ldexp;

criterion::criterion_main!(exp::benches, ldexp::benches);

#[macro_export]
macro_rules! bench {
    ($bench:expr, $criterion:expr, $f:expr) => {
        $bench($criterion, stringify!($f), $f);
    };
}
