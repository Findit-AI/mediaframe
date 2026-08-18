use super::*;
use crate::PixelSink;
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_a_len: usize,
  last_row_idx: usize,
}

impl PixelSink for CountingSink {
  type Input<'r> = Gbrapf32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Gbrapf32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_a_len = row.a().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}

impl Gbrapf32Sink for CountingSink {}

// Compile-pass regression for the codex round-1 finding on PR #109
// (hand-written `gbrapf32_to`). See `gbrpf32::tests` for full rationale.
// BE-aware callers should use `gbrapf32_to_endian::<S, BE>` directly.
#[test]
fn gbrapf32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrapf32Sink>() {
    let _: fn(&Gbrapf32LeFrame<'_>, &mut S) -> Result<(), S::Error> = gbrapf32_to::<S>;
  }
}

#[test]
fn gbrapf32_walker_visits_every_row_once() {
  let buf = std::vec![1.0f32; 4 * 4];
  let frame = Gbrapf32LeFrame::try_new(&buf, &buf, &buf, &buf, 4, 4, 4, 4, 4, 4).unwrap();
  let mut sink = CountingSink {
    rows_seen: 0,
    last_a_len: 0,
    last_row_idx: 0,
  };
  gbrapf32_to(&frame, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_a_len, 4);
  assert_eq!(sink.last_row_idx, 3);
}
