use super::*;
use crate::{PixelSink, frame::V210Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = V210Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: V210Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.v210().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl V210Sink for CountingSink {}

#[test]
fn v210_walker_visits_every_row_once() {
  let buf = std::vec![0u8; 16 * 4];
  let frame = V210Frame::new(&buf, 6, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  v210_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 16);
  assert_eq!(sink.last_row_idx, 3);
}
