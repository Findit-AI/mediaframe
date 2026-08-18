use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Ya8Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Ya8Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Ya8Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Ya8Sink for CountingSink {}

#[test]
fn ya8_walker_visits_every_row_once() {
  // 4 px × 2 bytes × 4 rows = 32 bytes (tight stride)
  let buf = std::vec![0u8; 32];
  let frame = Ya8Frame::new(&buf, 4, 4, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  ya8_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 8); // width × 2 bytes per row
  assert_eq!(sink.last_row_idx, 3);
}
