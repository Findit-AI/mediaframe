use super::*;
use crate::{PixelSink, frame::Grayf16Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_y_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Grayf16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Grayf16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_y_len = row.y().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Grayf16Sink<false> for CountingSink {}

#[test]
fn grayf16_walker_visits_every_row_once() {
  // 4 px × 4 rows = 16 f16 elements (tight stride)
  let buf = std::vec![half::f16::from_f32(0.5); 16];
  let frame = Grayf16Frame::new(&buf, 4, 4, 4);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_y_len: 0,
    last_row_idx: 0,
  };
  grayf16_to(&frame, false, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_y_len, 4); // width f16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression mirroring the `planar1_be` arm guarantee on the
// sibling Grayf32 source: the macro emits an LE-only `grayf16_to` wrapper
// alongside the const-generic `grayf16_to_endian` so explicit-turbofish
// callers like `grayf16_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn grayf16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Grayf16Sink>() {
    let _: fn(&crate::frame::Grayf16LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      grayf16_to::<S>;
  }
}
