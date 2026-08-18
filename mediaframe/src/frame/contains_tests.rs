//! The containment predicate, boundary by boundary.

use super::{Dimensions, Rect};

const CODED: Dimensions = Dimensions::new(1920, 1080);

#[test]
fn a_rect_flush_with_an_edge_is_contained() {
  let table = [
    // full-frame
    Rect::new(0, 0, 1920, 1080),
    // flush right
    Rect::new(480, 0, 1440, 1080),
    // flush bottom
    Rect::new(0, 80, 1920, 1000),
    // flush at both far corners
    Rect::new(1919, 1079, 1, 1),
  ];
  for r in table {
    assert!(CODED.contains(&r), "{r:?} ends on an edge, not past one");
  }
}

#[test]
fn one_pixel_past_an_edge_is_not_contained() {
  let table = [
    // one column too wide
    Rect::new(0, 0, 1921, 1080),
    // one row too tall
    Rect::new(0, 0, 1920, 1081),
    // right size, origin shifted one column right
    Rect::new(481, 0, 1440, 1080),
    // right size, origin shifted one row down
    Rect::new(0, 81, 1920, 1000),
    // the far corner, one pixel out on each axis
    Rect::new(1920, 1079, 1, 1),
    Rect::new(1919, 1080, 1, 1),
  ];
  for r in table {
    assert!(!CODED.contains(&r), "{r:?} reaches past an edge");
  }
}

/// An empty rectangle covers no pixel, so it has no pixel that could
/// fall outside — containment reduces to "is the origin inside the
/// closed raster".
#[test]
fn an_empty_rect_is_contained_wherever_its_origin_is() {
  // Origin anywhere within the closed raster, including on the far
  // edge, where an empty rect still touches nothing.
  for r in [
    Rect::default(),
    Rect::new(0, 0, 0, 1080),
    Rect::new(0, 0, 1920, 0),
    Rect::new(1920, 1080, 0, 0),
    Rect::new(960, 540, 0, 0),
  ] {
    assert!(CODED.contains(&r), "{r:?} is empty and inside");
  }

  // …and an empty rect whose origin is genuinely outside is not.
  for r in [Rect::new(1921, 0, 0, 0), Rect::new(0, 1081, 0, 0)] {
    assert!(!CODED.contains(&r), "{r:?} is empty but originates outside");
  }

  // The degenerate raster contains exactly the degenerate rect.
  assert!(Dimensions::default().contains(&Rect::default()));
  assert!(!Dimensions::default().contains(&Rect::new(0, 0, 1, 1)));
}

/// `x + width` past `u32::MAX` must not wrap into a false positive.
#[test]
fn an_overflowing_extent_is_not_contained() {
  let full = Dimensions::new(u32::MAX, u32::MAX);
  for r in [
    Rect::new(u32::MAX, 0, 1, 0),
    Rect::new(0, u32::MAX, 0, 1),
    Rect::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX),
  ] {
    assert!(!full.contains(&r), "{r:?} overflows its own extent");
  }
  // The largest non-overflowing rect is contained.
  assert!(full.contains(&Rect::new(0, 0, u32::MAX, u32::MAX)));
}

/// The predicate is `const`, so a descriptor's invariant can be pinned
/// at compile time rather than only asserted at run time.
#[test]
fn the_predicate_is_usable_in_const_context() {
  // Evaluated by the compiler — this would not build at all if
  // `contains` stopped being a `const fn`.
  const ANSWERS: [bool; 2] = [
    CODED.contains(&Rect::new(480, 0, 1440, 1080)),
    CODED.contains(&Rect::new(481, 0, 1440, 1080)),
  ];
  assert_eq!(ANSWERS, [true, false]);
}
