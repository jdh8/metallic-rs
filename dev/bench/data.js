window.BENCHMARK_DATA = {
  "lastUpdate": 1781202490737,
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
      },
      {
        "commit": {
          "author": {
            "email": "chen.pang.he@jdh8.org",
            "name": "Chen-Pang He",
            "username": "jdh8"
          },
          "committer": {
            "email": "chen.pang.he@jdh8.org",
            "name": "Chen-Pang He",
            "username": "jdh8"
          },
          "distinct": true,
          "id": "512830dd4f9e692ab649bfd8dfcfbbbac3007b54",
          "message": ".gitignore .claude by default",
          "timestamp": "2026-06-10T17:18:51+08:00",
          "tree_id": "a145129cf36f96b692b94ca44d7c7af7f953ed6d",
          "url": "https://github.com/jdh8/metallic-rs/commit/512830dd4f9e692ab649bfd8dfcfbbbac3007b54"
        },
        "date": 1781087082516,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acos",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acos",
            "value": 31,
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
            "value": 25,
            "range": "± 1",
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
            "value": 20,
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
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asin",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asin",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asin",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinf",
            "value": 19,
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinh",
            "value": 22,
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
            "value": 61,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan2",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 999,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan2",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanf",
            "value": 13,
            "range": "± 0",
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
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cbrt",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cbrt",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrtf",
            "value": 8,
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
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cos",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosf",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosh",
            "value": 22,
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 23,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cosh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::coshf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erf",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 46,
            "range": "± 0",
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10",
            "value": 51,
            "range": "± 2",
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
            "value": 24,
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
            "value": 12,
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expf",
            "value": 18,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expm1f",
            "value": 10,
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1f",
            "value": 27,
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
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 12,
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
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 28,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 25,
            "range": "± 0",
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log10",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10f",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 21,
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
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1p",
            "value": 23,
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
            "value": 22,
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log2f",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2f",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log2",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2f",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::logf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::logf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::logf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 57,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::pow",
            "value": 53,
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
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::powf",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::powf",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::powf",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::powf",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::round",
            "value": 6,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::roundf",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::round",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::roundf",
            "value": 10,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincosf",
            "value": 22,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincosf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin_cos",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincosf",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinf",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinh",
            "value": 23,
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
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinh",
            "value": 28,
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
            "value": 10,
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
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tan",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tan",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tan",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tan",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanf",
            "value": 51,
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
            "value": 15,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tanh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 79,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 111,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 51,
            "range": "± 1",
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
      },
      {
        "commit": {
          "author": {
            "email": "chen.pang.he@jdh8.org",
            "name": "Chen-Pang He",
            "username": "jdh8"
          },
          "committer": {
            "email": "chen.pang.he@jdh8.org",
            "name": "Chen-Pang He",
            "username": "jdh8"
          },
          "distinct": true,
          "id": "5d4d4e3b60e39eefb760da9ec7de00dde2f3db48",
          "message": "Replace erfc t-bridge (segments 3–4) with 1/x² parametrization\n\nFor x > 4: erfc(x) = y·exp(−x²)·P(y²), y = 1/x, where P is a\nminimax of erfcx(x)·x on u = 1/x² ∈ [0.00135, 0.0625].\nAccurate path: degree 34 (err 2^−111). Fast path: degree 19 (err 2^−74).\nReplaces the two-segment t-bridge (segments 3–4, degree 23/14 each).\n\nComputing y = 1/x via Newton refinement is much cheaper than the\nt-bridge's double-double 2/(2+x). Over x ∈ [−6, 28] this yields a\n31.6% throughput gain (46.7 → 31.9 ns), bringing metallic within 8%\nof CORE-MATH (29.5 ns).\n\nAlso bumps poly_dd's internal scratch CAP from 32 to 36 to accommodate\nthe 35-term accurate polynomial.",
          "timestamp": "2026-06-12T02:16:50+08:00",
          "tree_id": "2ad3d35de5c268d8ca6fdde867ddb1bd77bd799b",
          "url": "https://github.com/jdh8/metallic-rs/commit/5d4d4e3b60e39eefb760da9ec7de00dde2f3db48"
        },
        "date": 1781202490359,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acos",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acos",
            "value": 22,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acosh",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 23,
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
            "value": 20,
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinf",
            "value": 22,
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
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinf",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinh",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinhf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::asinh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinhf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan",
            "value": 35,
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
            "value": 1029,
            "range": "± 4",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanf",
            "value": 14,
            "range": "± 0",
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
            "value": 33,
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
            "value": 34,
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
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atanh",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanhf",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrt",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cbrt",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cbrt",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrtf",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrtf",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cbrt",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cbrtf",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cos",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cos",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cos",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cos",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosf",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::coshf",
            "value": 11,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::coshf",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 26,
            "range": "± 0",
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erfc",
            "value": 32,
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
            "value": 19,
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
            "value": 74,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10f",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 19,
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
            "value": 24,
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
            "value": 11,
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
            "value": 15,
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
            "range": "± 1",
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expm1f",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expm1f",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp_m1",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1f",
            "value": 25,
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
            "value": 7,
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
            "value": 17,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexp",
            "value": 22,
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
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 27,
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
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 25,
            "range": "± 0",
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log10",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10f",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1p",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln_1p",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1p",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1pf",
            "value": 26,
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 24,
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
            "value": 20,
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
            "value": 22,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 62,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::pow",
            "value": 55,
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
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::powf",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::powf",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::powf",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::powf",
            "value": 49,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::round",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::roundf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincosf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincosf",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin_cos",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincosf",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinf",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sin",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinf",
            "value": 52,
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
            "value": 27,
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::sinh",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinhf",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tan",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tan",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tan",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tan",
            "value": 93,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tanh",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanh",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanhf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tanh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 111,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgammaf",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgammaf",
            "value": 109,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}