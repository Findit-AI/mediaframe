use super::*;
use crate::{PixelSink, frame::Rgbaf16LeFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgbaf16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, _row: Rgbaf16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    Ok(())
  }
}
impl Rgbaf16Sink<false> for CountingSink {}

// Compile-pass regression mirroring the `packed_be` arm guarantee on the
// sibling Rgbf16 source: the macro emits an LE-only `rgbaf16_to` wrapper
// alongside the const-generic `rgbaf16_to_endian` so explicit-turbofish
// callers like `rgbaf16_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn rgbaf16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Rgbaf16Sink>() {
    let _: fn(&crate::frame::Rgbaf16LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      rgbaf16_to::<S>;
  }
}

#[test]
fn rgbaf16_walker_visits_every_row_once() {
  // width=4, stride=16 (4*4), height=4 → plane needs 64 f16 elements
  let buf = std::vec![half::f16::ZERO; 16 * 4];
  let frame = Rgbaf16LeFrame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink { rows_seen: 0 };
  rgbaf16_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
}
