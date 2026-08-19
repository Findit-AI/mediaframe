use super::*;

// Compile-pass regression for the codex round-1 finding on PR #106
// (`planar1_be` arm). Switching the Gray16 walker macro from `planar1`
// to `planar1_be` without an LE wrapper would change the public
// `gray16_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), breaking downstream callers using the explicit
// sink spelling `gray16_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature; this test pins it.
#[test]
fn gray16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gray16Sink>() {
    let _: fn(&crate::frame::Gray16LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      gray16_to::<S>;
  }
}
