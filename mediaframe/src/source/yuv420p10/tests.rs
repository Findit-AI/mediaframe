use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #110
// (`planar3_bits_be` arm). The macro emits an LE-only `yuv420p10_to`
// wrapper alongside the const-generic `yuv420p10_to_endian` so
// explicit-turbofish callers like `yuv420p10_to::<MySink>(...)` keep
// compiling.
#[test]
fn yuv420p10_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuv420p10Sink>() {
    let _: fn(
      &crate::frame::Yuv420p10LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuv420p10_to::<S>;
  }
}
