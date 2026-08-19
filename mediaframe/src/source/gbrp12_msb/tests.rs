use super::*;

// Compile-pass regression mirroring the `planar3_bits_be` arm guarantee
// (cf. `gbrp10::tests`): the macro emits an LE-only `gbrp12_msb_to` wrapper
// alongside the const-generic `gbrp12_msb_to_endian`.
#[test]
fn gbrp12_msb_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrp12MsbSink>() {
    let _: fn(&crate::frame::Gbrp12MsbLeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      gbrp12_msb_to::<S>;
  }
}
