# metallic

This crate provides C math functions written from scratch in Rust, aiming for
correct rounding (error ≤ 0.5 ulp) and performance comparable to or better than
the system math library.

After updating the codebase, please

- Format the code with `cargo fmt`.
- Run the tests with `cargo test`.
- Propose a clear and descriptive commit message.

Note: do **not** run tests with `--all-features`.  Enabling the optional
`core-math` feature replaces several metallic functions (notably the `f32`
trigonometric functions and `f32::powf`) with CORE-MATH's implementations.
Running the test suite with that feature would silently exercise CORE-MATH
instead of metallic for those functions, defeating the purpose of the tests.
