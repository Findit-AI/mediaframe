use super::*;
use crate::{PixelSink, frame::V30XFrame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = V30XRow<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: V30XRow<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl V30XSink for CountingSink {}

#[test]
fn v30x_walker_visits_every_row_once() {
  let buf = std::vec![0u32; 4 * 4]; // 4 px × 4 rows = 16 u32 words
  let frame = V30XFrame::new(&buf, 4, 4, 4);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  v30x_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 4); // width u32 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
