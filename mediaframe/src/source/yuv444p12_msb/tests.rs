use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
// (cf. `gbrp12_msb::tests`): the macro emits an LE-only `yuv444p12_msb_to`
// wrapper alongside the const-generic `yuv444p12_msb_to_endian`.
#[test]
fn yuv444p12_msb_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuv444p12MsbSink>() {
    let _: fn(
      &crate::frame::Yuv444p12MsbLeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuv444p12_msb_to::<S>;
  }
}
