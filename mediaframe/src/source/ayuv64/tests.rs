use super::*;
use crate::{PixelSink, color::KernelMatrix, frame::Ayuv64Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_packed_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Ayuv64Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Ayuv64Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_packed_len = row.packed().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Ayuv64Sink for CountingSink {}

#[test]
fn ayuv64_walker_visits_every_row_once() {
  // 4 px × 4 channels × 4 rows = 64 u16 elements
  let buf = std::vec![0u16; 4 * 4 * 4];
  let frame = Ayuv64Frame::new(&buf, 4, 4, 16);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_packed_len: 0,
    last_row_idx: 0,
  };
  ayuv64_to(&frame, false, KernelMatrix::Bt709, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_packed_len, 16); // width × 4 u16 elements per row
  assert_eq!(sink.last_row_idx, 3);
}
