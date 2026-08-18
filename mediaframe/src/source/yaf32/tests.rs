use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Yaf32Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Yaf32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Yaf32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Yaf32Sink<false> for CountingSink {}

// Compile-pass regression mirroring the `packed_be` arm guarantee on the
// sibling Ya16 source: the macro emits an LE-only `yaf32_to` wrapper
// alongside the const-generic `yaf32_to_endian` so explicit-turbofish
// callers like `yaf32_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn yaf32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Yaf32Sink>() {
    let _: fn(
      &crate::frame::Yaf32LeFrame<'_>,
      bool,
      crate::color::KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = yaf32_to::<S>;
  }
}

#[test]
fn yaf32_walker_visits_every_row_once() {
  // 4 px × 2 f32 × 4 rows = 32 f32 elements (tight stride)
  let buf = std::vec![0.0f32; 32];
  let frame = Yaf32Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  yaf32_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 8); // width × 2 f32 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
