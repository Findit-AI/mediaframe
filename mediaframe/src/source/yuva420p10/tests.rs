use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #110
// (`planar4_bits_be` arm). The macro emits an LE-only `yuva420p10_to`
// wrapper alongside the const-generic `yuva420p10_to_endian` so
// explicit-turbofish callers like `yuva420p10_to::<MySink>(...)` keep
// compiling.
#[test]
fn yuva420p10_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuva420p10Sink>() {
    let _: fn(
      &crate::frame::Yuva420p10LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuva420p10_to::<S>;
  }
}
