use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Rgb48Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgb48Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Rgb48Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.rgb48().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Rgb48Sink for CountingSink {}

#[test]
fn rgb48_walker_visits_every_row_once() {
  // width=4, stride=12 (3*4), height=4 → plane needs 48 u16 elements
  let buf = std::vec![0u16; 12 * 4];
  let frame = Rgb48Frame::new(&buf, 4, 4, 12);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  rgb48_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 12); // width * 3 u16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression for the LE-only custom sink spelling. The
// generated `$sink<const BE: bool = false>` carries an LE default so
// downstream callers can keep writing `impl Rgb48Sink for MySink`
// (no `<false>`) and `S: Rgb48Sink` bounds. This mirrors the fix for
// codex high-severity finding on `walker_macro.rs:242`.
#[test]
fn rgb48_sink_le_default_compiles_without_const_arg() {
  // `impl Rgb48Sink for CountingSink` (above) would already fail to
  // compile if the LE default regressed; this test additionally pins
  // the bare-bound form `S: Rgb48Sink` and confirms it monomorphizes
  // to the LE walker.
  fn walks_le<S: Rgb48Sink>(frame: &Rgb48Frame<'_>, sink: &mut S) -> Result<(), S::Error> {
    rgb48_to(frame, true, KernelMatrix::Bt709, sink)
  }

  let buf = std::vec![0u16; 12 * 4];
  let frame = Rgb48Frame::new(&buf, 4, 4, 12);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  walks_le(&frame, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
}

// Compile-pass regression for the codex finding (PR #105 review). Switching
// from `walker!(packed)` to `walker!(packed_be)` would otherwise change the
// public `rgb48_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), which breaks downstream callers using the previous
// explicit sink spelling `rgb48_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature.
#[test]
fn rgb48_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Rgb48Sink>() {
    let _: fn(&crate::frame::Rgb48LeFrame<'_>, bool, KernelMatrix, &mut S) -> Result<(), S::Error> =
      rgb48_to::<S>;
  }
}
