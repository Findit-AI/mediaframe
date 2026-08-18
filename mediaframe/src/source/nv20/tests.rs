use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Nv20Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_y_len: usize,
  last_uv_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Nv20Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Nv20Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_y_len = row.y().len();
    self.last_uv_len = row.uv().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Nv20Sink for CountingSink {}

#[test]
fn nv20_walker_visits_every_row_once() {
  // 8×4 frame. 4:2:2 → chroma is half-width (4 pairs = 8 u16) at
  // full height (4 rows). Y is 8 u16 × 4 rows.
  let y = std::vec![0u16; 8 * 4];
  let uv = std::vec![0u16; 8 * 4];
  let frame = Nv20Frame::new(&y, &uv, 8, 4, 8, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_y_len: 0,
    last_uv_len: 0,
    last_row_idx: 0,
  };
  nv20_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_y_len, 8); // full-width Y
  assert_eq!(sink.last_uv_len, 8); // half-width interleaved = width u16
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression mirroring the `semi_planar_be` arm guarantee
// (cf. `p010_to_explicit_turbofish_one_generic_compiles`): the macro
// emits an LE-only `nv20_to` wrapper alongside the const-generic
// `nv20_to_endian` so explicit-turbofish callers like
// `nv20_to::<MySink>(...)` keep compiling.
#[test]
fn nv20_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Nv20Sink>() {
    let _: fn(&crate::frame::Nv20LeFrame<'_>, bool, KernelMatrix, &mut S) -> Result<(), S::Error> =
      nv20_to::<S>;
  }
}
