window.BENCHMARK_DATA = {
  "lastUpdate": 1781039857425,
  "repoUrl": "https://github.com/jdh8/metallic-rs",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Chen-Pang He",
            "username": "jdh8",
            "email": "chen.pang.he@jdh8.org"
          },
          "committer": {
            "name": "Chen-Pang He",
            "username": "jdh8",
            "email": "chen.pang.he@jdh8.org"
          },
          "id": "a02551c04c4eb8e915faa054e80095c89516c5e3",
          "message": "Funnel FMA through crate::fma / crate::fast_mul_add; deny suboptimal_flops\n\nDisallowing the builtin mul_add pushed code toward raw `a * b + c`, which\nclippy's suboptimal_flops flags — and whose autofix suggests the very\nmul_add that is banned. Break the deadlock by routing every FMA through the\ncrate wrappers and making both lints hard errors:\n\n- hyp.rs `ratio_1ps` / erf.rs `erfc_eval`: exact residual extractions use\n  crate::fma (bit-identical on FMA targets), correction terms use\n  crate::fast_mul_add. Clears all 9 warnings.\n- lib.rs: deny clippy::suboptimal_flops and clippy::disallowed_methods.\n- Document the rule in CLAUDE.md and refresh the program-math-functions\n  skill (crate::mul_add was renamed to crate::fast_mul_add; EFT guidance\n  now points at crate::fma/fmaf instead of the disallowed builtin).\n\ncargo clippy clean; full test suite passes.",
          "timestamp": "2026-06-09T19:07:25Z",
          "url": "https://github.com/jdh8/metallic-rs/commit/a02551c04c4eb8e915faa054e80095c89516c5e3"
        },
        "date": 1781039856741,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acos",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acos",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acosf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acos",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acosh",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acosh",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acoshf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acoshf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acosh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acoshf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asin",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asin",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asin",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asin",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::asin",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinh",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinhf",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinhf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::asinh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinhf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 34,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan2",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 1030,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan2",
            "value": 33,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanf",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atan",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanh",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanh",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atanh",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanhf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanhf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atanh",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanhf",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrt",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cbrt",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cbrt",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrtf",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrtf",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cbrt",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cbrtf",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cos",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cos",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cos",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cos",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosf",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosh",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::coshf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::coshf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cosh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::coshf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erf",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erf",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 46,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erfc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::exp",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10f",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10f",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10f",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp2",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::exp2",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2f",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp2f",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp2",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expm1",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expm1",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::exp_m1",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expm1f",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expm1f",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp_m1",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1f",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::frexp",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::frexp",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::frexpf",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::frexpf",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypot",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypot",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypot",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexp",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexp",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexpf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 46,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgammaf",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log10",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10f",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1p",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln_1p",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1p",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1pf",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1pf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln_1p",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1pf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log2",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log2f",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2f",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log2",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2f",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::logf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::logf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::logf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 62,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::pow",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::powf",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::pow",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::powf",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::powf",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::powf",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::powf",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::round",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::round",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::round",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::roundf",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::round",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::roundf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 86,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincosf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin_cos",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincosf",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinf",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinf",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinh",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sinh",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinh",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinhf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sinh",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinhf",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tan",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tan",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tan",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tan",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tan",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanf",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tanh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanhf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanhf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tanh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgammaf",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgammaf",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}