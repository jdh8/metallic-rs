mod exp;
mod ldexp;

use criterion::criterion_main;

criterion_main!(exp::benches, ldexp::benches);

#[macro_export]
macro_rules! bench {
    ($criterion:expr, $f:expr) => {
        bench($criterion, stringify!($f), $f);
    };
}
