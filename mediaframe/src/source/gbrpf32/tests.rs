use super::*;
use crate::PixelSink;
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_g_len: usize,
  last_row_idx: usize,
}

impl PixelSink for CountingSink {
  type Input<'r> = Gbrpf32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Gbrpf32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_g_len = row.g().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}

impl Gbrpf32Sink for CountingSink {}

// Compile-pass regression for the codex round-1 finding on PR #109
// (hand-written `gbrpf32_to`). The pre-Phase-4 signature was a single
// `<S>` generic; Phase 4 added `<S, const BE: bool>` to the inner
// const-generic helper, which would break downstream callers using the
// explicit `gbrpf32_to::<MySink>(...)` spelling. The LE-only
// `gbrpf32_to<S>` wrapper preserves source compatibility; BE-aware
// callers should use `gbrpf32_to_endian::<S, BE>` directly.
#[test]
fn gbrpf32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrpf32Sink>() {
    let _: fn(&Gbrpf32LeFrame<'_>, &mut S) -> Result<(), S::Error> = gbrpf32_to::<S>;
  }
}

#[test]
fn gbrpf32_walker_visits_every_row_once() {
  // 4 px × 4 rows, tight stride
  let buf = std::vec![0.5f32; 4 * 4];
  let frame = Gbrpf32LeFrame::try_new(&buf, &buf, &buf, 4, 4, 4, 4, 4).unwrap();
  let mut sink = CountingSink {
    rows_seen: 0,
    last_g_len: 0,
    last_row_idx: 0,
  };
  gbrpf32_to(&frame, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_g_len, 4);
  assert_eq!(sink.last_row_idx, 3);
}
