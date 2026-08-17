use super::*;
use crate::color::KernelMatrix;

// Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
// (cf. `gbrp10::tests`): the macro emits an LE-only `gbrp10_msb_to` wrapper
// alongside the const-generic `gbrp10_msb_to_endian` so explicit-turbofish
// callers like `gbrp10_msb_to::<MySink>(...)` keep compiling.
#[test]
fn gbrp10_msb_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrp10MsbSink>() {
    let _: fn(
      &crate::frame::Gbrp10MsbLeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = gbrp10_msb_to::<S>;
  }
}
