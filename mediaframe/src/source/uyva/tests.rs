use super::*;
use crate::{PixelSink, frame::UyvaFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = UyvaRow<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: UyvaRow<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl UyvaSink for CountingSink {}

#[test]
fn uyva_walker_visits_every_row_once() {
  // 4 px × 4 channels × 4 rows = 64 bytes
  let buf = std::vec![0u8; 4 * 4 * 4];
  let frame = UyvaFrame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  uyva_to(&frame, false, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 16); // width × 4 bytes per row
  assert_eq!(sink.last_row_idx, 3);
}
