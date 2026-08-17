use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Ya16Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Ya16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Ya16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Ya16Sink<false> for CountingSink {}

// Compile-pass regression for the codex / Copilot finding on PR #106
// (`packed_be` arm). Switching the Ya16 walker macro from `packed`
// to `packed_be` without an LE wrapper would change the public
// `ya16_to` signature from one generic param (`S`) to two
// (`S, const BE: bool`), breaking downstream callers using the explicit
// sink spelling `ya16_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature; this test pins it.
#[test]
fn ya16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Ya16Sink>() {
    let _: fn(
      &crate::frame::Ya16LeFrame<'_>,
      bool,
      crate::color::KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = ya16_to::<S>;
  }
}

#[test]
fn ya16_walker_visits_every_row_once() {
  // 4 px × 2 u16 × 4 rows = 32 u16 elements (tight stride)
  let buf = std::vec![0u16; 32];
  let frame = Ya16Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  ya16_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 8); // width × 2 u16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
