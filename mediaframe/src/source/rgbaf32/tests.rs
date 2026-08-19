use super::*;
use crate::{PixelSink, frame::Rgbaf32LeFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgbaf32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, _row: Rgbaf32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    Ok(())
  }
}
impl Rgbaf32Sink<false> for CountingSink {}

// Compile-pass regression mirroring the `packed_be` arm guarantee on the
// sibling Rgbf32 source: the macro emits an LE-only `rgbaf32_to` wrapper
// alongside the const-generic `rgbaf32_to_endian` so explicit-turbofish
// callers like `rgbaf32_to::<MySink>(...)` keep compiling (function-position
// const-generic defaults aren't allowed).
#[test]
fn rgbaf32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Rgbaf32Sink>() {
    let _: fn(&crate::frame::Rgbaf32LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      rgbaf32_to::<S>;
  }
}

#[test]
fn rgbaf32_walker_visits_every_row_once() {
  // width=4, stride=16 (4*4), height=4 → plane needs 64 f32 elements
  let buf = std::vec![0.0_f32; 16 * 4];
  let frame = Rgbaf32LeFrame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink { rows_seen: 0 };
  rgbaf32_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
}
