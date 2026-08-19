use super::*;

// Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
// (cf. `gbrp10_msb::tests`): the macro emits an LE-only `yuv444p10_msb_to`
// wrapper alongside the const-generic `yuv444p10_msb_to_endian` so
// explicit-turbofish callers like `yuv444p10_msb_to::<MySink>(...)` keep
// compiling.
#[test]
fn yuv444p10_msb_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuv444p10MsbSink>() {
    let _: fn(&crate::frame::Yuv444p10MsbLeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      yuv444p10_msb_to::<S>;
  }
}
