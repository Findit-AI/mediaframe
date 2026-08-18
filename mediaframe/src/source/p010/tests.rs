use super::*;

// Compile-pass regression for the codex round-1 finding on PR #110
// (`semi_planar_be` arm). The macro emits an LE-only `p010_to` wrapper
// alongside the const-generic `p010_to_endian` so explicit-turbofish
// callers like `p010_to::<MySink>(...)` keep compiling.
#[test]
fn p010_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: P010Sink>() {
    let _: fn(&crate::frame::P010LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> = p010_to::<S>;
  }
}
