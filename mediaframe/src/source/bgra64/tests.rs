use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Bgra64Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_width: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Bgra64Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Bgra64Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_width = row.bgra64().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Bgra64Sink for CountingSink {}

#[test]
fn bgra64_walker_visits_every_row_once() {
  let buf = std::vec![0u16; 16 * 4];
  let frame = Bgra64Frame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_width: 0,
    last_row_idx: 0,
  };
  bgra64_to(&frame, true, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_width, 16);
  assert_eq!(sink.last_row_idx, 3);
}
