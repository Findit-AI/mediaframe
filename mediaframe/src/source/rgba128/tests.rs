use super::*;
use crate::{PixelSink, frame::Rgba128Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgba128Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Rgba128Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.rgba128().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Rgba128Sink<false> for CountingSink {}

#[test]
fn rgba128_walker_visits_every_row_once() {
  // width=4, stride=16 (4*4), height=4 → plane needs 64 u32 elements
  let buf = std::vec![0u32; 16 * 4];
  let frame = Rgba128Frame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  rgba128_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 16); // width * 4 u32 elements per row
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression mirroring the `packed_be` arm guarantee on the
// sibling Rgba64 source: the macro emits an LE-only `rgba128_to` wrapper
// alongside the const-generic `rgba128_to_endian` so explicit-turbofish
// callers like `rgba128_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn rgba128_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Rgba128Sink>() {
    let _: fn(&crate::frame::Rgba128LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      rgba128_to::<S>;
  }
}
