use super::*;
use crate::{PixelSink, color::DcpTargetGamut, frame::Xyz12Frame};
use core::convert::Infallible;

#[test]
fn unnamed_gamut_has_no_kernel_gamut() {
  #[cfg(any(feature = "std", feature = "alloc"))]
  {
    assert!(KernelGamut::try_from(&DcpTargetGamut::other("aces-ap0")).is_err());
    assert!(KernelGamut::try_from(&DcpTargetGamut::other("")).is_err());
  }
  assert_eq!(
    KernelGamut::try_from(&DcpTargetGamut::Rec709),
    Ok(KernelGamut::Rec709)
  );
  assert_eq!(
    KernelGamut::try_from(&DcpTargetGamut::DciP3),
    Ok(KernelGamut::DciP3)
  );
  assert_eq!(
    KernelGamut::try_from(&DcpTargetGamut::Rec2020),
    Ok(KernelGamut::Rec2020)
  );
}

#[test]
fn every_kernel_gamut_has_luma_weights_summing_to_unity() {
  for g in [
    KernelGamut::Rec709,
    KernelGamut::DciP3,
    KernelGamut::Rec2020,
  ] {
    let (r, gr, b) = luma_weights_q15_for_gamut(g);
    // Q15 unity is 32768; rounding the three coefficients
    // independently can leave the sum one LSB short or long.
    assert!(
      (r + gr + b - 32768).abs() <= 1,
      "{g:?} luma weights must sum to Q15 unity"
    );
  }
}

struct CountingSink {
  began: bool,
  rows: usize,
}
impl PixelSink for CountingSink {
  type Input<'r> = Xyz12Row<'r, false>;
  type Error = Infallible;
  fn begin_frame(&mut self, _w: u32, _h: u32) -> Result<(), Infallible> {
    self.began = true;
    Ok(())
  }
  fn process(&mut self, _row: Xyz12Row<'_, false>) -> Result<(), Infallible> {
    self.rows += 1;
    Ok(())
  }
}
impl Xyz12Sink for CountingSink {}

#[test]
fn xyz12_to_walks_rows() {
  let plane = [0u16; 3 * 2];
  let frame = Xyz12Frame::new(&plane, 1, 2, 3);
  let mut sink = CountingSink {
    began: false,
    rows: 0,
  };
  xyz12_to(&frame, &mut sink).unwrap();
  assert!(sink.began);
  assert_eq!(sink.rows, 2);
}

/// The target gamut reaches the rows from the **sink**, not from a
/// parameter — and the luma weights the walker derives follow it.
///
/// The gamut is an output axis and the sink is the output, so the sink
/// is the only thing asked. A sink that says nothing gets DCI-P3, the
/// theatrical decode a DCP master is mastered for.
#[test]
fn the_target_gamut_comes_from_the_sink() {
  struct GamutSink {
    declares: KernelGamut,
    seen: std::vec::Vec<(KernelGamut, (i32, i32, i32))>,
  }
  impl PixelSink for GamutSink {
    type Input<'r> = Xyz12Row<'r, false>;
    type Error = Infallible;
    fn process(&mut self, row: Xyz12Row<'_, false>) -> Result<(), Infallible> {
      self.seen.push((row.target_gamut(), row.luma_q15()));
      Ok(())
    }
  }
  impl Xyz12Sink for GamutSink {
    fn target_gamut(&self) -> KernelGamut {
      self.declares
    }
  }

  let plane = [0u16; 3 * 2];
  let frame = Xyz12Frame::new(&plane, 1, 2, 3);

  for declared in [
    KernelGamut::Rec709,
    KernelGamut::DciP3,
    KernelGamut::Rec2020,
  ] {
    let mut sink = GamutSink {
      declares: declared,
      seen: std::vec::Vec::new(),
    };
    xyz12_to(&frame, &mut sink).unwrap();
    let expected = (declared, luma_weights_q15_for_gamut(declared));
    assert_eq!(sink.seen, std::vec![expected; 2]);
  }

  // `CountingSink` overrides nothing: the stated default is DCI-P3.
  let silent = CountingSink {
    began: false,
    rows: 0,
  };
  assert_eq!(silent.target_gamut(), KernelGamut::DciP3);
}
