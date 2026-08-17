use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Y210Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Y210Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Y210Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Y210Sink for CountingSink {}

#[test]
fn y210_walker_visits_every_row_once() {
  let buf = std::vec![0u16; 8 * 4];
  let frame = Y210Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  y210_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 8);
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression for the codex finding (PR #105 review,
// `packed_be_y2xx` arm). Switching the Y2xx walker macro from a single
// `walker:` field to the `packed_be_y2xx` arm without an LE wrapper would
// change the public `y210_to` signature from one generic param (`S`) to
// two (`S, const BE: bool`), breaking downstream callers using the
// explicit sink spelling `y210_to::<MySink>(...)`. Function-position
// const-generic defaults aren't allowed, so the macro emits an LE-only
// wrapper preserving the original signature; this test pins it.
#[test]
fn y210_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Y210Sink>() {
    let _: fn(
      &crate::frame::Y210LeFrame<'_>,
      bool,
      crate::color::KernelMatrix,
      &mut S,
    ) -> Result<(), S::Error> = y210_to::<S>;
  }
}
