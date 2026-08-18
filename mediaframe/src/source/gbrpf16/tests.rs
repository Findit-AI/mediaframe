use super::*;
use crate::PixelSink;
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_g_len: usize,
  last_row_idx: usize,
}

impl PixelSink for CountingSink {
  type Input<'r> = Gbrpf16Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Gbrpf16Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_g_len = row.g().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}

impl Gbrpf16Sink for CountingSink {}

// Compile-pass regression for the codex round-1 finding on PR #109
// (hand-written `gbrpf16_to`). See `gbrpf32::tests` for full rationale.
// BE-aware callers should use `gbrpf16_to_endian::<S, BE>` directly.
#[test]
fn gbrpf16_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrpf16Sink>() {
    let _: fn(&Gbrpf16LeFrame<'_>, &mut S) -> Result<(), S::Error> = gbrpf16_to::<S>;
  }
}

#[test]
fn gbrpf16_walker_visits_every_row_once() {
  let buf = std::vec![half::f16::ZERO; 4 * 4];
  let frame = Gbrpf16LeFrame::try_new(&buf, &buf, &buf, 4, 4, 4, 4, 4).unwrap();
  let mut sink = CountingSink {
    rows_seen: 0,
    last_g_len: 0,
    last_row_idx: 0,
  };
  gbrpf16_to(&frame, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_g_len, 4);
  assert_eq!(sink.last_row_idx, 3);
}
