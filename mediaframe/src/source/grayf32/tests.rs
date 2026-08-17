use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Grayf32Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_y_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Grayf32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Grayf32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_y_len = row.y().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Grayf32Sink<false> for CountingSink {}

#[test]
fn grayf32_walker_visits_every_row_once() {
  // 4 px × 4 rows = 16 f32 elements (tight stride)
  let buf = std::vec![0.5f32; 16];
  let frame = Grayf32Frame::new(&buf, 4, 4, 4);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_y_len: 0,
    last_row_idx: 0,
  };
  grayf32_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_y_len, 4); // width f32 elements per row
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression for the codex round-1 finding on PR #106
// (`planar1_be` arm). Switching the Grayf32 walker macro from `planar1`
// to `planar1_be` without an LE wrapper would change the public
// `grayf32_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), breaking downstream callers using the explicit
// sink spelling `grayf32_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature; this test pins it.
#[test]
fn grayf32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Grayf32Sink>() {
    let _: fn(
      &crate::frame::Grayf32LeFrame<'_>,
      bool,
      KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = grayf32_to::<S>;
  }
}
