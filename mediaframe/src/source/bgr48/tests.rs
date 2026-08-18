use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Bgr48Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Bgr48Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Bgr48Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.bgr48().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Bgr48Sink for CountingSink {}

#[test]
fn bgr48_walker_visits_every_row_once() {
  let buf = std::vec![0u16; 12 * 4];
  let frame = Bgr48Frame::new(&buf, 4, 4, 12);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  bgr48_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 12);
  assert_eq!(sink.last_row_idx, 3);
}
