use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #110
// (`planar4_be` arm). The macro emits an LE-only `yuva444p9_to` wrapper
// alongside the const-generic `yuva444p9_to_endian` so explicit-turbofish
// callers like `yuva444p9_to::<MySink>(...)` keep compiling.
#[test]
fn yuva444p9_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuva444p9Sink>() {
    let _: fn(
      &crate::frame::Yuva444p9LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuva444p9_to::<S>;
  }
}
