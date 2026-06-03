# metallic

This crate provides C math functions written from scratch in Rust, aiming for
correct rounding (error ≤ 0.5 ulp) and performance comparable to or better than
the system math library.

After updating the codebase, please

- Format the code with `cargo fmt`.
- Run the tests with `cargo test`.
- Propose a clear and descriptive commit message.

Note: do **not** run tests with `--all-features`.  The `_no_fma` feature
disables FMA usage and would cause tests to exercise a different code path than
the default build, producing misleading results.
