use super::*;

// Compile-pass regression for the codex round-1 finding on PR #106
// (`planar1_bits_be` arm). See `gray9::tests` for the full rationale.
#[test]
fn gray10_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gray10Sink>() {
    let _: fn(&crate::frame::Gray10LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      gray10_to::<S>;
  }
}
