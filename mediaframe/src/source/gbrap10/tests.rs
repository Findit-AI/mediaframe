use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #109
// (`planar4_bits_be` arm). See `gbrp10::tests` for the full rationale.
// BE-aware callers should use `gbrap10_to_endian::<S, BE>` directly.
#[test]
fn gbrap10_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrap10Sink>() {
    let _: fn(
      &crate::frame::Gbrap10LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = gbrap10_to::<S>;
  }
}
