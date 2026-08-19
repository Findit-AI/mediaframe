use super::*;
use crate::{PixelSink, frame::Nv20Frame};
use core::convert::Infallible;

struct CountingSink {
  rows_seen: usize,
  last_y_len: usize,
  last_uv_len: usize,
  last_row_idx: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Nv20Row<'r>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    Ok(())
  }
  fn process(&mut self, row: Nv20Row<'_>) -> Result<(), Infallible> {
    self.rows_seen += 1;
    self.last_y_len = row.y().len();
    self.last_uv_len = row.uv().len();
    self.last_row_idx = row.row();
    Ok(())
  }
}
impl Nv20Sink for CountingSink {}

#[test]
fn nv20_walker_visits_every_row_once() {
  // 8×4 frame. 4:2:2 → chroma is half-width (4 pairs = 8 u16) at
  // full height (4 rows). Y is 8 u16 × 4 rows.
  let y = std::vec![0u16; 8 * 4];
  let uv = std::vec![0u16; 8 * 4];
  let frame = Nv20Frame::new(&y, &uv, 8, 4, 8, 8);
  let mut sink = CountingSink {
    rows_seen: 0,
    last_y_len: 0,
    last_uv_len: 0,
    last_row_idx: 0,
  };
  nv20_to(&frame, true, &mut sink).unwrap();
  assert_eq!(sink.rows_seen, 4);
  assert_eq!(sink.last_y_len, 8); // full-width Y
  assert_eq!(sink.last_uv_len, 8); // half-width interleaved = width u16
  assert_eq!(sink.last_row_idx, 3);
}

// Compile-pass regression mirroring the `semi_planar_be` arm guarantee
// (cf. `p010_to_explicit_turbofish_one_generic_compiles`): the macro
// emits an LE-only `nv20_to` wrapper alongside the const-generic
// `nv20_to_endian` so explicit-turbofish callers like
// `nv20_to::<MySink>(...)` keep compiling.
#[test]
fn nv20_to_explicit_turbofish_one_generic_compiles() {
  #[allow(clippy::type_complexity)]
  fn _check<S: Nv20Sink>() {
    let _: fn(&crate::frame::Nv20LeFrame<'_>, bool, &mut S) -> Result<(), S::Error> = nv20_to::<S>;
  }
}

/// The matrix reaches the rows from the **sink**, not from a parameter.
///
/// This is the whole point of the reshape: `nv20_to` no longer takes a
/// `KernelMatrix`, so the only way `Bt2020Ncl` can appear on a row is
/// that the walker asked this sink for it. A sink that says nothing
/// gets `Unspecified` — the documented BT.709 posture — rather than
/// whatever a caller happened to pass beside it.
#[test]
fn the_matrix_comes_from_the_sink() {
  use crate::color::KernelMatrix;

  struct MatrixSink {
    declares: KernelMatrix,
    seen: std::vec::Vec<KernelMatrix>,
  }
  impl PixelSink for MatrixSink {
    type Input<'r> = Nv20Row<'r>;
    type Error = Infallible;
    fn kernel_matrix(&self) -> KernelMatrix {
      self.declares
    }
    fn process(&mut self, row: Nv20Row<'_>) -> Result<(), Infallible> {
      self.seen.push(row.matrix());
      Ok(())
    }
  }
  impl Nv20Sink for MatrixSink {}

  let y = std::vec![0u16; 8 * 4];
  let uv = std::vec![0u16; 8 * 4];
  let frame = Nv20Frame::new(&y, &uv, 8, 4, 8, 8);

  let mut declared = MatrixSink {
    declares: KernelMatrix::Bt2020Ncl,
    seen: std::vec::Vec::new(),
  };
  nv20_to(&frame, true, &mut declared).unwrap();
  assert_eq!(declared.seen, std::vec![KernelMatrix::Bt2020Ncl; 4]);

  // The `CountingSink` above overrides nothing, so it takes the
  // default — and the default is a stated posture, not an accident.
  let mut silent = CountingSink {
    rows_seen: 0,
    last_y_len: 0,
    last_uv_len: 0,
    last_row_idx: 0,
  };
  assert_eq!(silent.kernel_matrix(), KernelMatrix::Unspecified);
  nv20_to(&frame, true, &mut silent).unwrap();
}

/// The `#[doc(hidden)]` door builds a row without a frame, which is the
/// one thing `pub(crate) new` took away and the only reason the door
/// exists — an out-of-tree kernel-parity suite drives a single kernel
/// this way. Exercised here so it cannot rot untested: nothing in this
/// crate needs it (every in-tree row comes from a walker), so without
/// this test the door would ship unproven.
#[test]
fn the_hidden_door_builds_a_row_without_a_frame() {
  use crate::color::KernelMatrix;

  struct OneRowSink {
    matrix: Option<KernelMatrix>,
    full_range: Option<bool>,
    y_len: usize,
  }
  impl PixelSink for OneRowSink {
    type Input<'r> = Nv20Row<'r>;
    type Error = Infallible;
    fn process(&mut self, row: Nv20Row<'_>) -> Result<(), Infallible> {
      self.matrix = Some(row.matrix());
      self.full_range = Some(row.full_range());
      self.y_len = row.y().len();
      Ok(())
    }
  }
  impl Nv20Sink for OneRowSink {}

  let y = std::vec![0u16; 8];
  let uv = std::vec![0u16; 8];
  let mut sink = OneRowSink {
    matrix: None,
    full_range: None,
    y_len: 0,
  };

  // No frame, no walker — exactly the shape a kernel-parity test uses.
  sink
    .process(Nv20Row::for_tests(
      &y,
      &uv,
      0,
      KernelMatrix::Bt2020Ncl,
      true,
    ))
    .unwrap();

  assert_eq!(sink.matrix, Some(KernelMatrix::Bt2020Ncl));
  assert_eq!(sink.full_range, Some(true));
  assert_eq!(sink.y_len, 8);
}
