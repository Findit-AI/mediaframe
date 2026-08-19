use super::*;

// Compile-pass regression for the codex round-1 finding on PR #106
// (`planar1_bits_be` arm). Switching the Gray9 walker macro from
// `planar1_bits` to `planar1_bits_be` without an LE wrapper would change
// the public `gray9_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), breaking downstream callers using the explicit
// sink spelling `gray9_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature; this test pins it.
#[test]
fn gray9_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gray9Sink>() {
    let _: fn(&crate::frame::Gray9LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      gray9_to::<S>;
  }
}
