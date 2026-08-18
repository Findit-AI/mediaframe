use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Vyu444Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Vyu444Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Vyu444Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Vyu444Sink for CountingSink {}

#[test]
fn vyu444_walker_visits_every_row_once() {
  // 4 px × 3 channels × 4 rows = 48 bytes
  let buf = std::vec![0u8; 4 * 3 * 4];
  let frame = Vyu444Frame::new(&buf, 4, 4, 12);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  vyu444_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 12); // width × 3 bytes per row
  assert_eq!(sink.last_row_idx, 3);
}
