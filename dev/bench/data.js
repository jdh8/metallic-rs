window.BENCHMARK_DATA = {
  "lastUpdate": 1781651649130,
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
          "id": "947e5e238f1570299c12e9129f4eb2d8f53da4e8",
          "message": "Single-instruction cell index in the atan/atan2 fast legs\n\nA float-to-usize `as` cast saturates, which lowers to two cvttsd2si plus a\nsign-mask/cmov chain ahead of the ATAN_TABLE load — the same find as in the\nasin/acos commit.  k = round(8q) with q in [0, 1] proves k in 0..=8, so\nto_int_unchecked::<i64> emits the single conversion.  Smaller effect than on\nasin/acos (the load overlaps the leg's division latency): atan 1.148x ->\n1.132x CORE-MATH same-run, atan2 unchanged.  The accurate `atan_dd` keeps the\nplain cast (cold path).\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T00:13:11+08:00",
          "tree_id": "d981482ed388c9157733749261c267c686200ae2",
          "url": "https://github.com/jdh8/metallic-rs/commit/947e5e238f1570299c12e9129f4eb2d8f53da4e8"
        },
        "date": 1781283094582,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 27,
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
            "value": 18,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acosh",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acosh",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 14,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acoshf",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asin",
            "value": 24,
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
            "value": 20,
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
            "value": 44,
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan",
            "value": 35,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan2",
            "value": 38,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 989,
            "range": "± 10",
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
            "value": 30,
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 34,
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
            "value": 19,
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
            "value": 30,
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
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cos",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cos",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 85,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 25,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 20,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erfc",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp",
            "value": 18,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10",
            "value": 14,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 20,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp2",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expf",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expf",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expf",
            "value": 13,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 30,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 10,
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
            "value": 22,
            "range": "± 0",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 46,
            "range": "± 0",
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
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10",
            "value": 32,
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
            "value": 26,
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
            "value": 21,
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
            "value": 22,
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
            "value": 20,
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
            "value": 64,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::pow",
            "value": 58,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "f64::powf",
            "value": 46,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "libm::pow",
            "value": 99,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::powf",
            "value": 34,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 36,
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
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 42,
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
            "value": 26,
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
            "value": 56,
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
            "range": "± 1",
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tan",
            "value": 39,
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
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 19,
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
            "value": 50,
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
            "value": 19,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tanh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 79,
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
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgammaf",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgammaf",
            "value": 108,
            "range": "± 1",
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
          "id": "b1c9f863e0c75206f8ff18a8985e012262bf47f7",
          "message": "Ordered Fast2Sum folds in the f64 lgamma Stirling fast leg\n\n`lgamma_stirling_fast` closed its Stirling assembly\n`(z−½)·ln z − z + ½ln(2π) + tail` with three renormalizing double-double\n`Add`s, all on the critical path *after* `ln_fast` — the dominant cost for\nthe ~91% of the benchmark range (`z ≥ 8`) that takes the direct-Stirling\nleg.  For `z ≥ 8` the lead `(z−½)·ln z ≥ 15.6` dominates each later term and\nevery running partial sum stays above the next, so the folds become ordered\nFast2Sums (`DoubleDouble::add_ordered`, no renormalization), shortening the\nserial chain; the dropped ≈2⁻¹⁰⁵ slack is far inside the `LGAMMA_FAST_ERR`\n(2⁻⁵⁶) gate.  The new `fold_ordering::lgamma_stirling` test proves the three\norderings on the real arithmetic across the whole `[8, LGAMMA_FAST_BOUND]`\nfast-leg band.\n\nlgamma 26.2 → 21.5 ns (1.84× → 1.51× vs libm, same-run, FMA on); accurate\nfallback untouched, the 4M-point MPFR sweep and the hard-to-round corpus\nremain bit-exact.\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T01:22:28+08:00",
          "tree_id": "4248fddb0500ba7cc3aab8c35396e9b4c389b843",
          "url": "https://github.com/jdh8/metallic-rs/commit/b1c9f863e0c75206f8ff18a8985e012262bf47f7"
        },
        "date": 1781285687885,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 24,
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
            "value": 30,
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
            "value": 15,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 26,
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
            "value": 19,
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
            "range": "± 1",
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinhf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinhf",
            "value": 13,
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
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 34,
            "range": "± 0",
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
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 1002,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 33,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atan",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanh",
            "value": 37,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanh",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atanh",
            "value": 43,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 44,
            "range": "± 1",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cbrt",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 19,
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
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cos",
            "value": 31,
            "range": "± 1",
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
            "value": 18,
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
            "value": 20,
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
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 42,
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
            "value": 15,
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
            "value": 21,
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
            "value": 16,
            "range": "± 1",
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
            "value": 17,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp2",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expf",
            "value": 12,
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
            "value": 23,
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
            "value": 27,
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
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::frexpf",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::frexpf",
            "value": 5,
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
            "value": 19,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 10,
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
            "value": 21,
            "range": "± 0",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 19,
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
            "range": "± 1",
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 33,
            "range": "± 9",
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
            "value": 26,
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
            "value": 18,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 31,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2f",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log2",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2f",
            "value": 19,
            "range": "± 1",
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
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 20,
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
            "value": 58,
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
            "value": 76,
            "range": "± 7",
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::round",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::round",
            "value": 12,
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
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 28,
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 92,
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
            "value": 26,
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
            "value": 56,
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
            "value": 39,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 79,
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
            "value": 50,
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
          "id": "4bf01e9b73fc56d68d7382908def9d4b961b26eb",
          "message": "Exact-z single-f64 reduction for the f64 ln fast leg\n\nReplace the double-double reduction in `ln_fast` with CORE-MATH-style\nexact arithmetic, closing the structural gap behind the whole log\nfamily (#5 item 5) and most of lgamma's residual (#9):\n\n- 128 `LnCell`s (32-byte aligned, 4 KiB): per-cell reciprocal `r` on\n  the 2^-8 grid chosen so `z = fma(r, m, -1)` is *exact* — a pure\n  lattice argument (`r·m` is a multiple of 2^-60 and `|z| <= 2^-7`\n  keeps it within 2^53 multiples), verified per cell in exact rational\n  arithmetic by the new `tools/gen_ln_exact_f64.py`, max |z| 2^-7.48.\n- `l1 = -ln r` on the 2^-42 grid, so `fma(e, LN2_HI, l1)` is exact and\n  the assembly collapses to one Fast2Sum with `z` (orderings proven by\n  the generator for the tight `e = 0` / `e = -1` cells) plus a\n  plain-f64 `z²·Q(z)` tail in the low word.  Leg error <2^-66 absolute\n  (was ≈2^-68 with ~4 ops more serial depth); all Ziv gates unchanged.\n- `log`/`log2`/`log10`/`log1p` consume the raw unnormalized pair\n  (CM-style, the gate needs no renormalization); `ln_fast` keeps the\n  normalized contract for pow/hyp/lgamma.\n- log1p: exact-z table branch with the `dz = r·δ` low-word fold; the\n  small-|x| branch gains a correction-scaled gate `eps = x²·2^-48`\n  (the flat 2^-56 relative gate fell back to the dd kernel on ~9% of\n  random small inputs) and a `|x| < 2^-54` return-x fast exit.\n- lgamma: `LGAMMA_FAST_BOUND` 1024 -> 256 so the Stirling gate keeps\n  its margin against the looser ln leg (t·2^-66 <= 2^-58); consumer\n  error-budget docs updated in pow/hyp/gamma.\n\nAccurate tiers are bit-identical: `ln_dd`, `dint::ln_accurate`, and\nlog1p's fallback still use the old `log_reduce`/`L_TABLE` path.\n\nSame-run CORE-MATH ratios (ns, -Ctarget-cpu=x86-64-v3):\n  log    16.7 -> 14.7 vs 14.7  (1.13x -> 0.99x, ties CM)\n  log2   18.3 -> 15.1 vs 14.3  (1.27x -> 1.05x)\n  log10  18.1 -> 14.9 vs 15.9  (1.15x -> 0.94x, beats CM)\n  log1p  21.3 -> 17.1 vs 15.0  (1.45x -> 1.14x)\n  atanh  16.9 -> 14.4 vs 13.2  (1.25x -> 1.09x)\n  asinh  12.7 -> 11.8 vs 10.4  (1.21x -> 1.14x)\n  acosh  14.2 -> 12.7 vs 12.4  (1.16x -> 1.02x)\n  pow    29.5 vs CM 29.3       (1.01x)\n  lgamma 21.3 -> 18.8 vs libm 14.0  (1.50x -> 1.34x)\n\nFull suite green: per-function ~134M-point bit sweeps + dense near-1 /\nnear-0 sweeps against the CORE-MATH oracle, corpus cases bit-exact.\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T03:12:10+08:00",
          "tree_id": "6cc54516e130e3a49fcdd10a7237e77e2184798b",
          "url": "https://github.com/jdh8/metallic-rs/commit/4bf01e9b73fc56d68d7382908def9d4b961b26eb"
        },
        "date": 1781292247045,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 27,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 22,
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
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acoshf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acoshf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acosh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acoshf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asin",
            "value": 25,
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
            "value": 22,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 17,
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
            "value": 44,
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
            "value": 1030,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 38,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atan",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanh",
            "value": 26,
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 34,
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
            "range": "± 1",
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
            "value": 19,
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
            "value": 35,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 20,
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
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 42,
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
            "value": 17,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10f",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10f",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10f",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 20,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp2",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 14,
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
            "value": 11,
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
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::frexpf",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::frexpf",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypot",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypot",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypot",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypotf",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexp",
            "value": 21,
            "range": "± 0",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 35,
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 25,
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
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log10",
            "value": 25,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 20,
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
            "value": 26,
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
            "range": "± 1",
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
            "value": 24,
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
            "value": 20,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 56,
            "range": "± 1",
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
            "value": 5,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 36,
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
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 39,
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
            "value": 85,
            "range": "± 0",
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
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinf",
            "value": 26,
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
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinh",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sinh",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinh",
            "value": 22,
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tan",
            "value": 39,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 76,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 44,
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
            "range": "± 3",
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
          "id": "8d1cb707ba8035f569fcbd4e7dc854548b0e2263",
          "message": "Make f64 lgamma correctly rounded (triple-double ln/sin port + sound reflection gate)\n\nlgamma had the most misses of any function (268,133): the double-double\naccurate path (lgamma_dd via the dd ln_sum, ≈2⁻¹⁰⁵) fell short of ties up to\n64 bits past the round bit — worst in the z<0.5 / near-integer reflection where\nthe sin(πz) cancellation amplifies the deficit.\n\nLift the cold accurate path to triple-double (the tgamma/atan2/pow route),\nreusing the TripleDouble in gamma.rs:\n- ln_td — port of CORE-MATH's as_logd_accurate (≈2⁻¹²⁵); tables in the new\n  gamma_td_tables.rs (regenerated by tools/gen_lgamma_td.py).\n- ln_abs_sinpi_td — relatively-accurate ln|sin πz| on metallic's exact\n  r = z − q/2 reduction, result-anchored as ln|r| + ln(π·S(r²)) to dodge\n  subnormal underflow.\n- td central table (ln of TGAMMA_TD), Stirling tail, recurrence logs.\n- Load-bearing details: the ln(1+r) offset in ln_sum_td carried as a\n  double-double (a bare f64 caps it at ≈2⁻¹⁰⁷); a cancellation-robust\n  td_add_exact (VecSum distillation) for the reflection/near-root combines;\n  result-anchored Taylor series near the roots z=1,2.\n\nFix a latent unsound reflection gate (the erf-gate class): for near-subnormal\nsin(πz) the fast dd sin loses its low word, but the gate only budgeted the\nlnΓ(1−z) leg — e.g. lgamma(1e-320) was off by 6.5e-5. Add a LGAMMA_SIN_RELIABLE\nguard and a permanent lgamma_reflection_fast_leg_is_sound MPFR test. Remove the\ndead dd accurate path. Special-case ladder and tgamma left untouched.\n\nThis was the last RED function: every f64 worst_cases gate is now green (#6).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T19:44:20+08:00",
          "tree_id": "7c4563c8fe565a3f6a35903ff3cbfc735af53ec6",
          "url": "https://github.com/jdh8/metallic-rs/commit/8d1cb707ba8035f569fcbd4e7dc854548b0e2263"
        },
        "date": 1781438935092,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 27,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 24,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 24,
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
            "value": 19,
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
            "value": 24,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinf",
            "value": 13,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 17,
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
            "value": 16,
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
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 35,
            "range": "± 0",
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
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 800,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan2",
            "value": 26,
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
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanh",
            "value": 23,
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
            "value": 33,
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
            "value": 18,
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
            "range": "± 1",
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
            "value": 40,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 91,
            "range": "± 1",
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 15,
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
            "value": 21,
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
            "value": 10,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 22,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 50,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erfc",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp",
            "value": 15,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 16,
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
            "range": "± 1",
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
            "value": 17,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::exp2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2",
            "value": 15,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expf",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expf",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::exp",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expf",
            "value": 13,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 10,
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
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexp",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexpf",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 49,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma",
            "value": 28,
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
            "range": "± 1",
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
            "value": 25,
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
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10",
            "value": 28,
            "range": "± 1",
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
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log10f",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1p",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln_1p",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1p",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1pf",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1pf",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln_1p",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1pf",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log2",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 19,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::logf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::logf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 57,
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
            "value": 100,
            "range": "± 0",
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
            "value": 5,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 41,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 92,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincos",
            "value": 42,
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
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinf",
            "value": 26,
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
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinh",
            "value": 23,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sinh",
            "value": 17,
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
            "value": 27,
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
            "value": 43,
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
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tan",
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tan",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanf",
            "value": 42,
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
            "value": 102,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma",
            "value": 54,
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
            "value": 44,
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
          "id": "d751bf05a13c2ed7c0daf28f98a5bbee985c65a7",
          "message": "Shrink f64 trig Dint accurate kernels from Taylor to minimax\n\nThe accurate-path sin/cos kernels (SIN_DINT, COS_DINT in trig.rs) were Taylor\nseries in u = r², needing 16 and 17 terms to push the truncation below the Dint\narithmetic floor at u = (π/4)².  A minimax (near-minimax Chebyshev) fit reaches\nthe same 2⁻¹³⁰ truncation in 14 terms each, so poly_dint spends two / three fewer\nDint::mul per evaluation — the accurate path costs one Dint::mul per term and is\nthe shared kernel behind sin, cos, sincos and tan.\n\nMeasured on the forced accurate path this trims it ~15% (e.g. sin's Dint\nreconstruction ~175 → ~149 ns).  End-to-end on the representation-uniform bench it\nis within run-to-run noise, since the Ziv fast leg only falls back ~2-3% of the\ntime — but it is a strict improvement: smaller table, fewer ops, and lower latency\nfor inputs near a multiple of π/2, which fall back more often.\n\nCorrect rounding is preserved by construction.  The minimax truncation (sin\n≈2⁻¹³⁹, cos ≈2⁻¹³⁴) and the Dint-quantized coefficient error (≈2⁻¹³³) both stay\nbelow the existing path's accuracy and ~2²¹× under the hardest .wc tie; the\ngenerator now re-measures and asserts the quantized error.  The strict\nworst_cases CR gates for sin/cos/sincos/tan stay bit-exact vs core-math 1.1.1.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T00:39:29+08:00",
          "tree_id": "a3ec366f516c3ff803adecb8dbddb4bee4f8141f",
          "url": "https://github.com/jdh8/metallic-rs/commit/d751bf05a13c2ed7c0daf28f98a5bbee985c65a7"
        },
        "date": 1781462960494,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acos",
            "value": 30,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acosf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acos",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 24,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 22,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acoshf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acoshf",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acosh",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acoshf",
            "value": 11,
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
            "value": 13,
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
            "value": 20,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 17,
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
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 37,
            "range": "± 1",
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
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 1032,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 38,
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
            "value": 12,
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
            "value": 28,
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanhf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanhf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atanh",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanhf",
            "value": 27,
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
            "value": 18,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 21,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cosh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::coshf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erf",
            "value": 22,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 46,
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
            "value": 16,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10",
            "value": 59,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10f",
            "value": 11,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp2",
            "value": 15,
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
            "range": "± 1",
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::expm1",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::expm1",
            "value": 18,
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
            "value": 6,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypot",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypot",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypotf",
            "value": 13,
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
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexp",
            "value": 22,
            "range": "± 0",
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
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma",
            "value": 27,
            "range": "± 0",
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
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 39,
            "range": "± 7",
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
            "value": 26,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 24,
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
            "value": 22,
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
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 25,
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
            "value": 57,
            "range": "± 0",
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
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::powf",
            "value": 34,
            "range": "± 1",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincos",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 97,
            "range": "± 2",
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
            "value": 58,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sinf",
            "value": 50,
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
            "value": 17,
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
            "range": "± 1",
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
            "value": 44,
            "range": "± 1",
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
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 19,
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
            "value": 49,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 72,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 111,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 50,
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
            "range": "± 6",
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
          "id": "b34c480b9b3e7822ebbe4a839f129fd4bc35671d",
          "message": "Speed up f64 cosh/sinh by dropping the negligible e⁻ˣ term for large |x|\n\ncosh/sinh build ½(eˣ ± e⁻ˣ) = 2^(q−1)·(m ± 2⁻²q/m) from the two-level eˣ\nmantissa m∈[1,2), and combine_fast always forms the e⁻ˣ correction\nt = 2⁻²q/m with an f64 division (plus a double-double add).  But that\ncorrection is c ≈ m·e⁻²ˣ relative to the result: once q ≥ 35 (|x| ≳ 35·ln2\n≈ 24.3) it is ≤ 2⁻⁷⁰ — far below both the fast leg's own ≈2⁻⁶⁴·⁷ slip and\nthe 2⁻⁶² HYP_ZIV_EPS gate — so ½(eˣ ± e⁻ˣ) rounds identically to ½eˣ.\n\nShort-circuit there: gate the mantissa m directly (= ½eˣ) and skip the\ndivision + double-double add.  On a value-uniform [−710, 710] sweep ~96% of\ninputs clear the cutoff, so the division leaves the hot path for almost the\nwhole bench.  The gate is unchanged (the dropped 2⁻⁷⁰ is negligible against\nits margin, so soundness and the fallback rate match the combine path), and\nthe rare Ziv straddle still defers to the accurate leg, which forms the full\ne⁻ˣ.  q ≥ 33 already suffices; 35 keeps the leg's full margin.\n\nSame-run, -Ctarget-cpu=x86-64-v3, 3s/1s, vs core-math 1.1.1 (box loaded, so\nabsolutes are inflated but the same-run ratio holds):\n  cosh 12.81 → 9.19 ns (−28%), ratio 1.39× → 1.00×\n  sinh 13.88 → 10.56 ns (−24%), ratio 1.33× → 1.01×\nBoth move from the slow band to parity with CORE-MATH.\n\nCorrect rounding preserved for both: test_{cosh,sinh}_worst_cases (strict CR\ngate vs the core-math 1.1.1 oracle on the .wc corpora), _worst_faithful, the\n2M-point dense + full-range bit-stepping sweep, and the 5M-point MPFR\ncross-check (--features mpfr) are all bit-exact.  combine_fast and the\naccurate legs are untouched.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T22:10:18+08:00",
          "tree_id": "3f584bcac401ecee265d7ee3a589b19cd041e22a",
          "url": "https://github.com/jdh8/metallic-rs/commit/b34c480b9b3e7822ebbe4a839f129fd4bc35671d"
        },
        "date": 1781533785092,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acos",
            "value": 30,
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
            "value": 18,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 24,
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
            "value": 24,
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
            "value": 19,
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
            "value": 26,
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
            "value": 13,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 18,
            "range": "± 1",
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::asinh",
            "value": 18,
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
            "value": 43,
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
            "range": "± 0",
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
            "value": 38,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 992,
            "range": "± 7",
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
            "value": 28,
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 35,
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 19,
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
            "range": "± 1",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 87,
            "range": "± 1",
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 21,
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::cosh",
            "value": 18,
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
            "value": 22,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::erfc",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::erfc",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp",
            "value": 16,
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
            "value": 23,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp10",
            "value": 14,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp10",
            "value": 60,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10f",
            "value": 11,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 16,
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
            "value": 12,
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
            "value": 19,
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
            "range": "± 1",
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
            "value": 6,
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
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypot",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypot",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypotf",
            "value": 13,
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
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexp",
            "value": 21,
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
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 47,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 27,
            "range": "± 1",
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln",
            "value": 24,
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
            "value": 26,
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
            "value": 26,
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
            "value": 24,
            "range": "± 0",
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
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 25,
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
            "value": 19,
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
            "value": 52,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::pow",
            "value": 53,
            "range": "± 2",
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
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 85,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 50,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincos",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 92,
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
            "value": 58,
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
            "value": 17,
            "range": "± 1",
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
            "value": 93,
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
            "value": 19,
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
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 44,
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
            "range": "± 1",
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
          "id": "89c997c90952680eaadcc9aab2962ff84c416509",
          "message": "docs(skill): dedupe program-math-functions, refresh stale refs, add laziness ladder\n\nConsolidate rules that were restated across SKILL.md and the reference files\n(FMA discipline, the RN-only/beat-CORE-MATH advantage, the crate::poly\nconventions, the --all-features warning) into one home plus pointers, and\ncompress the 78-line bivariate hard-to-round generation block to its reframing\nand tool pointers.\n\nRefresh references that pointed at dead code: Sum -> DoubleDouble, the\nsrc/f32_/ + per-family-file layout (no mod.rs/kernel.rs), and the real homes of\nfast_ldexp / rem_pio2 / normalize. Fix the --all-features rationale: the blocker\nis the _no_fma feature exercising a different code path, not a non-existent\ncore-math feature.\n\nAdd a \"do the least that clears 0.5 ulp\" laziness ladder and cross-reference its\nrungs from the spots that previously each re-derived \"only where needed\".\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-17T01:15:45+08:00",
          "tree_id": "09655e94c4669a3b09a2cb6ae44573eac7a35307",
          "url": "https://github.com/jdh8/metallic-rs/commit/89c997c90952680eaadcc9aab2962ff84c416509"
        },
        "date": 1781633558569,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 24,
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
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosf",
            "value": 18,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosf",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::acosh",
            "value": 24,
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
            "value": 21,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acoshf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::acosh",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acoshf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asin",
            "value": 25,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 17,
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
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 34,
            "range": "± 0",
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 1012,
            "range": "± 8",
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
            "value": 12,
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
            "value": 16,
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
            "value": 28,
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanh",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atanhf",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atanhf",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::atanh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atanhf",
            "value": 21,
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
            "value": 18,
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
            "value": 34,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosf",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 21,
            "range": "± 1",
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
            "value": 23,
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
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 52,
            "range": "± 2",
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
            "value": 15,
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 16,
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
            "value": 11,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::exp2",
            "value": 15,
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
            "value": 11,
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
            "value": 17,
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
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::expm1f",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::frexp",
            "value": 6,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::hypot",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypotf",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexp",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexp",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::ldexpf",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::ldexpf",
            "value": 20,
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
            "name": "core_math::lgamma",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgammaf",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log",
            "value": 32,
            "range": "± 9",
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
            "value": 26,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 20,
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
            "value": 26,
            "range": "± 0",
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1pf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln_1p",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1pf",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log2",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 25,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::logf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::logf",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 57,
            "range": "± 0",
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
            "range": "± 1",
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
            "value": 34,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::round",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::round",
            "value": 12,
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
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincos",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 91,
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
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sinh",
            "value": 17,
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
            "value": 37,
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
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 19,
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
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanh",
            "value": 14,
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
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanhf",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::tanh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanhf",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma",
            "value": 72,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma",
            "value": 54,
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
            "value": 50,
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
          "id": "d002099329e0c6523c128e4dc2100b84b6b64927",
          "message": "bench: re-band misleading f64 benches onto kernel-active magnitude bands\n\nSeveral f64 benches used an input distribution that does not exercise the\nfunction's kernel, so the reported metallic/CORE-MATH ratio measured the\nwrong code path (issue #5 \"range problems\"):\n\n- sin/cos/tan `..`     ~49% hit the |x|<2^-27 fast return, most of the rest\n                       the huge-arg Payne-Hanek path (rem_pio2 switches at\n                       x >= 2^20); the small-angle kernel was barely sampled.\n- tanh `..`            ~99.6% landed in the small-series leg or the >=20\n                       saturation; the poly/table kernel was never timed.\n- asinh `..`           ~half small-series, the rest the ln(2x) asymptotic.\n- atan2 `.., ..`       CORE-MATH ran its 192-bit accurate path on most pairs\n                       (~600 ns), so the ~0.04x ratio was uninterpretable.\n- log1p `-1.0..`       ~38% in the |x|<2^-54 fast return.\n- hypot `.., ..`       independent exponents -> one arg dominates -> the fast\n                       |larger| path, plus inf/NaN.\n\nRe-band each onto the existing bench::Exponents / PositiveExponents\nlog-uniform magnitude lever (already used by atan and pow) so every draw\nlands in the kernel; document the band and the fast-return cutoff inline.\n\nAdd a name-carrying bench! arm and per-band split benches for tgamma and\nlgamma (reflect / recur / stirling) to surface the per-band gaps the single\nvalue-uniform mean hides.\n\nThis resets the historical ns/ratios for these benches: the old open-ended\nranges are not comparable to the new banded ones. A first re-measurement\nshows atan2 is actually ~1.38x (metallic slower, CORE-MATH off its accurate\npath) and tanh ~1.7x, where the old benches implied a 25x win and a 0.97x\n\"beat\" respectively.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-17T06:28:21+08:00",
          "tree_id": "4dda7a802b014a6e0ff828bda2c94dfe41fcdecb",
          "url": "https://github.com/jdh8/metallic-rs/commit/d002099329e0c6523c128e4dc2100b84b6b64927"
        },
        "date": 1781651647522,
        "tool": "cargo",
        "benches": [
          {
            "name": "metallic::acos",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acos",
            "value": 26,
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
            "value": 24,
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::acosh",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::acosh",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::acosh",
            "value": 30,
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
            "value": 19,
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
            "value": 25,
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
            "range": "± 1",
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinf",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::asin",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::asinh",
            "value": 47,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::asinh",
            "value": 36,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::asinh",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::asinh",
            "value": 41,
            "range": "± 1",
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
            "value": 19,
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
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::atan2",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::atan2",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::atan2",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::atan2",
            "value": 62,
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
            "value": 27,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cbrt",
            "value": 19,
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
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cos",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cos",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cos",
            "value": 38,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::cosh",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::cosh",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::cosh",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::cosh",
            "value": 21,
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
            "value": 22,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::erfc",
            "value": 52,
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
            "value": 16,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp10",
            "value": 16,
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
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::exp2",
            "value": 16,
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
            "value": 25,
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
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::exp2f",
            "value": 14,
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
            "value": 17,
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
            "value": 8,
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
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypot",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::hypot",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::hypot",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::hypotf",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::hypotf",
            "value": 14,
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
            "value": 21,
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
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma_reflect",
            "value": 144,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma_reflect",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma_reflect",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma_recur",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma_recur",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma_recur",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgamma_stirling",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgamma_stirling",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgamma_stirling",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::lgammaf",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::lgammaf",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::lgammaf",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log",
            "value": 25,
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
            "value": 26,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log10f",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::log10",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log10f",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::log1p",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log1p",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::ln_1p",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log1p",
            "value": 28,
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
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::log2",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::log2",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::log2",
            "value": 25,
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
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::logf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f32::ln",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::logf",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::pow",
            "value": 57,
            "range": "± 0",
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
            "value": 34,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sin",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sin",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sin",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincos",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::sincos",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::sin_cos",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::sincos",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::sincosf",
            "value": 21,
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
            "value": 17,
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tan",
            "value": 58,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tan",
            "value": 50,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tan",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tan",
            "value": 40,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tanf",
            "value": 21,
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
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tanh",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "f64::tanh",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tanh",
            "value": 43,
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
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma",
            "value": 54,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma_reflect",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma_reflect",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma_reflect",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma_recur",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma_recur",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma_recur",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgamma_stirling",
            "value": 64,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "core_math::tgamma_stirling",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "libm::tgamma_stirling",
            "value": 111,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "metallic::tgammaf",
            "value": 51,
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