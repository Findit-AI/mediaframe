use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Rgba64Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Rgba64Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Rgba64Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.rgba64().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Rgba64Sink for CountingSink {}

#[test]
fn rgba64_walker_visits_every_row_once() {
  // width=4, stride=16 (4*4), height=4 → plane needs 64 u16 elements
  let buf = std::vec![0u16; 16 * 4];
  let frame = Rgba64Frame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  rgba64_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 16); // width * 4 u16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
