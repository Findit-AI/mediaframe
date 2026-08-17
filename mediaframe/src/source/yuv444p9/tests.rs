use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression for the codex round-1 finding on PR #110
// (`planar3_be` arm). The macro emits an LE-only `yuv444p9_to` wrapper
// alongside the const-generic `yuv444p9_to_endian` so explicit-turbofish
// callers like `yuv444p9_to::<MySink>(...)` keep compiling.
#[test]
fn yuv444p9_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuv444p9Sink>() {
    let _: fn(
      &crate::frame::Yuv444p9LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuv444p9_to::<S>;
  }
}
