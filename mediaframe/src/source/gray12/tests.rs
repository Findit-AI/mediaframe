use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #106
// (`planar1_bits_be` arm). See `gray9::tests` for the full rationale.
#[test]
fn gray12_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gray12Sink>() {
    let _: fn(
      &crate::frame::Gray12LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = gray12_to::<S>;
  }
}
