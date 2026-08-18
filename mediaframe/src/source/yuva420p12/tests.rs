use super::*;
use crate::color::KernelMatrix;

// Mirrors the yuva420p10 turbofish regression: the macro emits an
// LE-only `yuva420p12_to` wrapper alongside the const-generic
// `yuva420p12_to_endian`, so explicit-turbofish callers keep
// compiling.
#[test]
fn yuva420p12_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yuva420p12Sink>() {
    let _: fn(
      &crate::frame::Yuva420p12LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yuva420p12_to::<S>;
  }
}
