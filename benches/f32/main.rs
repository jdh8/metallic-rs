mod exp;
mod frexp;
mod ldexp;

criterion::criterion_main!(exp::benches, frexp::benches, ldexp::benches);

#[macro_export]
macro_rules! bench {
    ($bench:expr, $criterion:expr, $f:expr) => {
        $bench($criterion, stringify!($f), $f);
    };
}
