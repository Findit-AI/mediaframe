use super::*;
use crate::PixelSink;
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_g_len: usize,
  last_a_len: usize,
  last_row_idx: usize,
}

impl PixelSink for CountingSink {
  type Input<'r> = Gbrap32Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Gbrap32Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_g_len = row.g().len();
    self.last_a_len = row.a().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}

impl Gbrap32Sink for CountingSink {}

#[test]
fn gbrap32_walker_visits_every_row_once() {
  let buf = std::vec![0xDEAD_BEEFu32; 4 * 4];
  let frame = Gbrap32LeFrame::new(&buf, &buf, &buf, &buf, 4, 4, 4, 4, 4, 4);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_g_len: 0,
    last_a_len: 0,
    last_row_idx: 0,
  };
  gbrap32_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_g_len, 4);
  assert_eq!(sink.last_a_len, 4);
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression: the hand-written `gbrap32_to` keeps the
// single-generic signature so explicit-turbofish callers compile (mirrors
// `gbrapf32::tests`). BE-aware callers should use `gbrap32_to_endian`.
#[test]
fn gbrap32_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Gbrap32Sink>() {
    let _: fn(&Gbrap32LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> = gbrap32_to::<S>;
  }
}

/// The hand-written walkers migrated with the macro-generated ones —
/// `gbrap32_to` and its `_endian` helper read the matrix off the sink
/// too, and the LE wrapper forwards nothing extra.
#[test]
fn the_matrix_comes_from_the_sink() {
  use crate::color::KernelMatrix;

  struct MatrixSink {
    seen: std::vec::Vec<KernelMatrix>,
  }
  impl PixelSink for MatrixSink {
    type Input<'r> = Gbrap32Row<'r>;
    type Error = Infallible;
    fn kernel_matrix(&self) -> KernelMatrix {
      KernelMatrix::Smpte240m
    }
    fn process(&mut self, row: Gbrap32Row<'_>) -> Result<(), Infallible> {
      self.seen.push(row.matrix());
      Ok(())
    }
  }
  impl Gbrap32Sink for MatrixSink {}

  let buf = std::vec![0u32; 4 * 4];
  let frame = Gbrap32LeFrame::new(&buf, &buf, &buf, &buf, 4, 4, 4, 4, 4, 4);

  let mut via_wrapper = MatrixSink {
    seen: std::vec::Vec::new(),
  };
  gbrap32_to(&frame, false, &mut via_wrapper).unwrap();
  assert_eq!(via_wrapper.seen, std::vec![KernelMatrix::Smpte240m; 4]);

  let mut via_endian = MatrixSink {
    seen: std::vec::Vec::new(),
  };
  gbrap32_to_endian::<_, false>(&frame, false, &mut via_endian).unwrap();
  assert_eq!(via_endian.seen, std::vec![KernelMatrix::Smpte240m; 4]);
}
