//! Aspect-ratio projections and the SAR → display-size derivation.

use core::num::NonZeroI64;

use super::{Dimensions, Rect, SampleAspectRatio};

fn nz(v: i64) -> NonZeroI64 {
  NonZeroI64::new(v).expect("test denominator is non-zero")
}

fn ratio(d: Dimensions) -> Option<(i64, i64)> {
  d.aspect_ratio().map(|r| (r.num(), r.den().get()))
}

#[test]
fn the_storage_ratio_is_exact_and_unreduced() {
  let table = [
    (Dimensions::new(1920, 1080), Some((1920, 1080))),
    (Dimensions::new(720, 480), Some((720, 480))),
    // Not reduced: `Rational` stores what it is given.
    (Dimensions::new(2, 4), Some((2, 4))),
    // A zero *width* is a legitimate ratio of zero, not an absence.
    (Dimensions::new(0, 1080), Some((0, 1080))),
    // A zero *height* has no ratio at all.
    (Dimensions::new(1920, 0), None),
    (Dimensions::default(), None),
  ];
  for (d, expected) in table {
    assert_eq!(ratio(d), expected, "{d}");
  }
}

#[test]
fn a_rect_reports_its_own_ratio_regardless_of_origin() {
  let a = Rect::new(0, 0, 1440, 1080).aspect_ratio();
  let b = Rect::new(240, 17, 1440, 1080).aspect_ratio();
  assert_eq!(a, b);
  assert_eq!(a.map(|r| (r.num(), r.den().get())), Some((1440, 1080)));
  assert_eq!(Rect::new(0, 0, 16, 0).aspect_ratio(), None);
  assert_eq!(Rect::default().aspect_ratio(), None);
}

/// Rounding is FFmpeg's `av_rescale` (`AV_ROUND_NEAR_INF`):
/// `(a * b + c / 2) / c`, i.e. half away from zero.
#[test]
fn display_width_rounds_half_away_from_zero_like_av_rescale() {
  let table = [
    // (coded, sar num, sar den, expected display)
    // ITU-R BT.601 NTSC 16:9 — 873.2… rounds down.
    ((720, 480), 40, 33, Some((873, 480))),
    // NTSC DV 16:9 — 853.8… rounds down.
    ((720, 480), 32, 27, Some((853, 480))),
    // PAL DV 4:3 — 768.46… rounds down.
    ((720, 576), 16, 15, Some((768, 576))),
    // Anamorphic HDV — 1920.33… rounds down.
    ((1440, 1080), 4, 3, Some((1920, 1080))),
    // Square pixels are the identity.
    ((1920, 1080), 1, 1, Some((1920, 1080))),
    // The decisive pair: exactly .5 goes *away from zero*, where
    // banker's rounding would go to even. 0.5 → 1, not 0.
    ((1, 1), 1, 2, Some((1, 1))),
    // 2.5 → 3, not 2.
    ((5, 1), 1, 2, Some((3, 1))),
    // Just under .5 still goes down: 4 / 3 = 1.33… → 1.
    ((4, 2), 1, 3, Some((1, 2))),
  ];
  for ((w, h), num, den, expected) in table {
    let got = Dimensions::new(w, h)
      .display_size(SampleAspectRatio::new(num, nz(den)))
      .map(|d| (d.width(), d.height()));
    assert_eq!(got, expected, "{w}x{h} at {num}:{den}");
  }
}

/// FFmpeg's `0:1` means *unknown*, not *zero-wide*. There is no display
/// size to derive from it, so the caller is told rather than handed a
/// zero-width raster.
#[test]
fn an_unknown_sar_has_no_display_size() {
  assert_eq!(
    Dimensions::new(720, 480).display_size(SampleAspectRatio::new(0, nz(1))),
    None
  );
}

#[test]
fn a_display_width_past_u32_is_refused_rather_than_wrapped() {
  let huge = SampleAspectRatio::new(i64::MAX, nz(1));
  assert_eq!(Dimensions::new(u32::MAX, 8).display_size(huge), None);
  // The boundary itself still fits.
  assert_eq!(
    Dimensions::new(u32::MAX, 8).display_size(SampleAspectRatio::default()),
    Some(Dimensions::new(u32::MAX, 8))
  );
}

/// Height is never touched: the derivation upsamples the narrow axis
/// rather than downsampling the wide one, so no detail is discarded.
#[test]
fn the_vertical_axis_is_left_alone() {
  for (num, den) in [(40, 33), (33, 40), (1, 1), (2, 1)] {
    let out = Dimensions::new(720, 480)
      .display_size(SampleAspectRatio::new(num, nz(den)))
      .expect("a known SAR always derives a size here");
    assert_eq!(out.height(), 480, "{num}:{den}");
  }
}
