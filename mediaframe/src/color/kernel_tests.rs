//! The kernel-selector door: which matrices and gamuts cross it, which
//! are refused, and the `Copy` the closed selectors buy back.

use super::{
  DcpTargetGamut, KernelGamut, KernelMatrix, Matrix, UnsupportedKernelGamutError,
  UnsupportedKernelMatrixError,
};

const fn assert_copy<T: Copy>() {}

/// Every [`Matrix`] this build names, paired with the [`KernelMatrix`]
/// it exchanges for — or [`None`] when no kernel tabulates coefficients
/// for it. The roster is the commit's contract: not one variant more,
/// not one fewer.
fn roster() -> [(Matrix, Option<KernelMatrix>); 18] {
  [
    (Matrix::Rgb, None),
    (Matrix::Bt601, Some(KernelMatrix::Bt601)),
    (Matrix::Bt709, Some(KernelMatrix::Bt709)),
    (Matrix::Unspecified, Some(KernelMatrix::Unspecified)),
    (Matrix::Fcc, Some(KernelMatrix::Fcc)),
    (Matrix::Bt470Bg, Some(KernelMatrix::Bt470Bg)),
    (Matrix::Smpte170M, Some(KernelMatrix::Smpte170M)),
    (Matrix::Smpte240m, Some(KernelMatrix::Smpte240m)),
    (Matrix::YCgCo, Some(KernelMatrix::YCgCo)),
    (Matrix::Bt2020Ncl, Some(KernelMatrix::Bt2020Ncl)),
    (Matrix::Bt2020Cl, None),
    (Matrix::Smpte2085, None),
    (
      Matrix::ChromaDerivedNcl,
      Some(KernelMatrix::ChromaDerivedNcl),
    ),
    (Matrix::ChromaDerivedCl, None),
    (Matrix::Ictcp, None),
    (Matrix::IptC2, None),
    (Matrix::YCgCoRe, None),
    (Matrix::YCgCoRo, None),
  ]
}

#[test]
fn the_door_admits_exactly_the_tabulated_matrices() {
  let mut admitted = 0;
  for (m, expected) in roster() {
    match expected {
      Some(k) => {
        assert_eq!(
          KernelMatrix::try_from(&m),
          Ok(k),
          "{m} must exchange for a kernel matrix"
        );
        admitted += 1;
      }
      None => assert_eq!(
        KernelMatrix::try_from(&m),
        Err(UnsupportedKernelMatrixError),
        "{m} has no kernel coefficients and must be refused"
      ),
    }
  }
  assert_eq!(admitted, 10, "the closed set has exactly ten members");
}

/// Widening is total and injective, so a value that crossed the door
/// can always be spelled back as the descriptor it came from.
#[test]
fn widening_round_trips_every_kernel_matrix() {
  for (_, expected) in roster() {
    let Some(k) = expected else { continue };
    assert_eq!(KernelMatrix::try_from(&Matrix::from(k)), Ok(k));
  }
}

/// `Unspecified` reaches the kernels today and they resolve it
/// themselves. Refusing it at the door would be a behaviour change, not
/// a tightening — this test is the tripwire on that.
#[test]
fn unspecified_keeps_crossing_the_door() {
  assert_eq!(
    KernelMatrix::try_from(&Matrix::default()),
    Ok(KernelMatrix::Unspecified)
  );
}

/// The GBR identity is not a YCbCr matrix; no kernel has coefficients
/// for it, so it is refused rather than silently resolved.
#[test]
fn the_gbr_identity_is_refused() {
  assert_eq!(
    KernelMatrix::try_from(&Matrix::Rgb),
    Err(UnsupportedKernelMatrixError)
  );
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn an_unnamed_matrix_is_refused() {
  assert_eq!(
    KernelMatrix::try_from(&Matrix::other("ACEScct")),
    Err(UnsupportedKernelMatrixError)
  );
}

#[test]
fn the_gamut_door_admits_exactly_the_three_with_a_luma_basis() {
  for (g, k) in [
    (DcpTargetGamut::DciP3, KernelGamut::DciP3),
    (DcpTargetGamut::Rec709, KernelGamut::Rec709),
    (DcpTargetGamut::Rec2020, KernelGamut::Rec2020),
  ] {
    assert_eq!(KernelGamut::try_from(&g), Ok(k));
    assert_eq!(KernelGamut::try_from(&DcpTargetGamut::from(k)), Ok(k));
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn an_unnamed_gamut_is_refused() {
  assert_eq!(
    KernelGamut::try_from(&DcpTargetGamut::other("aces-ap0")),
    Err(UnsupportedKernelGamutError)
  );
}

/// The point of closing the set: no heap payload, so the selector — and
/// with it every row type that carries one — is `Copy` again.
#[test]
fn the_kernel_selectors_are_copy() {
  assert_copy::<KernelMatrix>();
  assert_copy::<KernelGamut>();
  assert_copy::<UnsupportedKernelMatrixError>();
  assert_copy::<UnsupportedKernelGamutError>();
}

/// One row type per `walker!` topology plus the hand-written pair, so
/// the `Copy` the closed selector bought back cannot silently regress.
#[test]
fn the_row_types_are_copy() {
  #[cfg(feature = "yuv-planar")]
  assert_copy::<crate::source::Yuv420pRow<'static>>();
  #[cfg(feature = "yuv-semi-planar")]
  assert_copy::<crate::source::Nv12Row<'static>>();
  #[cfg(feature = "yuv-444-packed")]
  assert_copy::<crate::source::Ayuv64Row<'static>>();
  #[cfg(feature = "gbr")]
  assert_copy::<crate::source::GbrpRow<'static>>();
  #[cfg(feature = "gbr")]
  assert_copy::<crate::source::Gbrap32Row<'static>>();
  #[cfg(feature = "mono")]
  assert_copy::<crate::source::MonoblackRow<'static>>();
  #[cfg(feature = "xyz")]
  assert_copy::<crate::source::Xyz12Row<'static, false>>();
}
