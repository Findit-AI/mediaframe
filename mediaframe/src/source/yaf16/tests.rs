use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Yaf16Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Yaf16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Yaf16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Yaf16Sink<false> for CountingSink {}

// Compile-pass regression mirroring the `packed_be` arm guarantee on the
// sibling Ya16 source: the macro emits an LE-only `yaf16_to` wrapper
// alongside the const-generic `yaf16_to_endian` so explicit-turbofish
// callers like `yaf16_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn yaf16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yaf16Sink>() {
    let _: fn(
      &crate::frame::Yaf16LeFrame<'_>,
      bool,
      crate::color::KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yaf16_to::<S>;
  }
}

#[test]
fn yaf16_walker_visits_every_row_once() {
  // 4 px × 2 f16 × 4 rows = 32 f16 elements (tight stride)
  let buf = std::vec![half::f16::ZERO; 32];
  let frame = Yaf16Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  yaf16_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 8); // width × 2 f16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
