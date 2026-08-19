use super::*;

// Compile-pass regression for the codex round-1 finding on PR #109
// (`planar3_bits_be` arm). Switching from `walker!(planar3_bits)` to
// `walker!(planar3_bits_be)` would otherwise change the public
// `gbrp10_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), which breaks downstream callers using the
// previous explicit sink spelling `gbrp10_to::<MySink>(...)`.
// Function-position const-generic defaults aren't allowed, so the
// macro emits an LE-only wrapper preserving the original signature.
// BE-aware callers should use `gbrp10_to_endian::<S, BE>` directly.
#[test]
fn gbrp10_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrp10Sink>() {
    let _: fn(&crate::frame::Gbrp10LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      gbrp10_to::<S>;
  }
}
