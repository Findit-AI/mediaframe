use super::*;
use crate::{PixelSink, frame::Rgbf16LeFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgbf16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, _row: Rgbf16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    Ok(())
  }
}
impl Rgbf16Sink for CountingSink {}

// Compile-pass regression for the LE-only custom sink spelling. The
// generated `$sink<const BE: bool = false>` carries an LE default so
// downstream callers can keep writing `impl Rgbf16Sink for MySink`
// (no `<false>`) and `S: Rgbf16Sink` bounds.
#[test]
fn rgbf16_sink_le_default_compiles_without_const_arg() {
  fn walks_le<S: Rgbf16Sink>(frame: &Rgbf16LeFrame<'_>, sink: &mut S) -> Result<(), S::Error> {
    rgbf16_to(frame, true, sink)
  }

  let buf = std::vec![half::f16::ZERO; 12 * 4];
  let frame = Rgbf16LeFrame::new(&buf, 4, 4, 12);
  let mut sink = CountingSink { rows_seen: 0 };
  walks_le(&frame, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
}

// Compile-pass regression for the codex finding (PR #105 review). Switching
// from `walker!(packed)` to `walker!(packed_be)` would otherwise change the
// public `rgbf16_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), which breaks downstream callers using the previous
// explicit sink spelling `rgbf16_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature.
#[test]
fn rgbf16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Rgbf16Sink>() {
    let _: fn(&crate::frame::Rgbf16LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> =
      rgbf16_to::<S>;
  }
}
