use super::*;
use crate::{PixelSink, frame::VuyaFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = VuyaRow<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: VuyaRow<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl VuyaSink for CountingSink {}

#[test]
fn vuya_walker_visits_every_row_once() {
  // 4 px × 4 channels × 4 rows = 64 bytes
  let buf = std::vec![0u8; 4 * 4 * 4];
  let frame = VuyaFrame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  vuya_to(&frame, false, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 16); // width × 4 bytes per row
  assert_eq!(sink.last_row_idx, 3);
}
