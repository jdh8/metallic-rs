#[macro_export]
macro_rules! bench {
    ($name:ident, $callback:expr; $($input:expr),+) => {
        fn $name(criterion: &mut criterion::Criterion) {
            criterion.bench_function(stringify!($callback), |bencher| {
                bencher.iter(|| $callback($($input),+));
            });
        }
    };
    ($name:ident, $callback:expr) => {
        bench!($name, $callback; rand::random());
    };
    ($name:ident, $callback:expr, in $range:expr) => {
        bench!($name, $callback; rand::random_range($range));
    };
}
