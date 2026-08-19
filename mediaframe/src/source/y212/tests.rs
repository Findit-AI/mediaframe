use super::*;
use crate::{PixelSink, frame::Y212Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Y212Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Y212Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Y212Sink for CountingSink {}

#[test]
fn y212_walker_visits_every_row_once() {
  let buf = std::vec![0u16; 8 * 4];
  let frame = Y212Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  y212_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 8);
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression for the codex finding (PR #105 review,
// `packed_be_y2xx` arm). See `y210::tests` for full rationale: the LE-only
// wrapper preserves the pre-Phase-4 single-generic public signature so
// explicit-turbofish callers like `y212_to::<MySink>(...)` keep compiling.
#[test]
fn y212_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Y212Sink>() {
    let _: fn(&crate::frame::Y212LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> = y212_to::<S>;
  }
}
