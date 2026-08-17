use super::*;

#[test]
fn dimensions_construction_and_accessors() {
  let d = Dimensions::new(1920, 1080);
  assert_eq!(d.width(), 1920);
  assert_eq!(d.height(), 1080);
  assert!(!d.is_zero());
  assert!(Dimensions::default().is_zero());
}

#[test]
fn dimensions_builder() {
  let d = Dimensions::new(0, 0).with_width(640).with_height(480);
  assert_eq!(d.width(), 640);
  assert_eq!(d.height(), 480);
}

#[cfg(feature = "std")]
#[test]
fn dimensions_display() {
  assert_eq!(std::format!("{}", Dimensions::new(1920, 1080)), "1920x1080");
}

#[test]
fn rect_construction_and_accessors() {
  let r = Rect::new(10, 20, 1280, 720);
  assert_eq!(r.x(), 10);
  assert_eq!(r.y(), 20);
  assert_eq!(r.width(), 1280);
  assert_eq!(r.height(), 720);
}

#[test]
fn rect_builder_chains() {
  let r = Rect::default()
    .with_x(8)
    .with_y(8)
    .with_width(640)
    .with_height(360);
  assert_eq!((r.x(), r.y(), r.width(), r.height()), (8, 8, 640, 360));
}

#[test]
fn rotation_defaults_and_as_str() {
  assert!(matches!(Rotation::default(), Rotation::D0));
  assert_eq!(Rotation::D0.as_str(), "0");
  assert_eq!(Rotation::D90.as_str(), "90");
  assert_eq!(Rotation::D180.as_str(), "180");
  assert_eq!(Rotation::D270.as_str(), "270");
  assert!(Rotation::D90.is_d_90());
}

#[test]
fn rotation_u32_round_trip_and_escape() {
  for r in [Rotation::D0, Rotation::D90, Rotation::D180, Rotation::D270] {
    assert_eq!(Rotation::from_u32(r.to_u32().unwrap()), Some(r));
  }
  assert_eq!(Rotation::from_u32(0), Some(Rotation::D0));
  assert_eq!(Rotation::from_u32(3), Some(Rotation::D270));
  // Unrecognised → rejected, never a silent collapse to D0.
  assert_eq!(Rotation::from_u32(99), None);
}

/// The escape carries a name, and has no numeric spelling. Needs the
/// allocator — at the no-alloc tier the vocabulary is closed.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn rotation_escape_keeps_its_name() {
  let odd = Rotation::other("45");
  assert_eq!(odd.as_str(), "45");
  assert_eq!(odd.to_u32(), None);
  assert_eq!("45".parse(), Ok(odd));
}

#[test]
fn sample_aspect_ratio_default_is_square() {
  let s = SampleAspectRatio::default();
  assert_eq!(s.num(), 1);
  assert_eq!(s.den().get(), 1);
  assert!(s.is_square());
}

#[test]
fn sample_aspect_ratio_construction_and_builders() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let s = SampleAspectRatio::new(40, nz(33));
  assert_eq!(s.num(), 40);
  assert_eq!(s.den().get(), 33);
  assert!(!s.is_square());
  let s2 = SampleAspectRatio::default().with_num(16).with_den(nz(9));
  assert_eq!((s2.num(), s2.den().get()), (16, 9));
  let mut s3 = SampleAspectRatio::default();
  s3.set_num(4).set_den(nz(3));
  assert_eq!((s3.num(), s3.den().get()), (4, 3));
}

#[cfg(feature = "std")]
#[test]
fn sample_aspect_ratio_display() {
  let nz = core::num::NonZeroI64::new(11).unwrap();
  assert_eq!(std::format!("{}", SampleAspectRatio::new(10, nz)), "10:11");
}

#[test]
fn plane_holds_owned_buffer() {
  let p: Plane<[u8; 4]> = Plane::new([1, 2, 3, 4], 4);
  assert_eq!(p.stride(), 4);
  assert_eq!(p.data_ref(), &[1, 2, 3, 4]);
  let raw = p.into_data();
  assert_eq!(raw, [1, 2, 3, 4]);
}

#[test]
fn plane_holds_borrowed_buffer() {
  let backing = [10u8, 20, 30, 40];
  let p: Plane<&[u8]> = Plane::new(&backing[..], 2);
  assert_eq!(p.stride(), 2);
  assert_eq!(*p.data_ref(), &[10, 20, 30, 40][..]);
}

#[test]
fn plane_with_stride_builder() {
  let p = Plane::new([0u8; 2], 0).with_stride(64);
  assert_eq!(p.stride(), 64);
}

// ---------- VideoFrame -------------------------------------------------

use crate::{color::Info, pixel_format::PixelFormat};

#[test]
fn video_frame_construction_defaults() {
  let planes: [Plane<&[u8]>; 4] = [
    Plane::new(&[][..], 16),
    Plane::new(&[][..], 8),
    Plane::new(&[][..], 8),
    Plane::new(&[][..], 0),
  ];
  let vf = VideoFrame::new(Dimensions::new(16, 16), PixelFormat::Yuv420p, planes, 3);
  assert_eq!(vf.dimensions(), Dimensions::new(16, 16));
  assert_eq!(vf.width(), 16);
  assert_eq!(vf.height(), 16);
  assert_eq!(*vf.pixel_format_ref(), PixelFormat::Yuv420p);
  assert_eq!(vf.plane_count(), 3);
  assert!(vf.visible_rect().is_none());
  assert_eq!(vf.color(), Info::UNSPECIFIED);
}

#[test]
fn video_frame_planes_slice_uses_plane_count() {
  let planes: [Plane<u32>; 4] = [
    Plane::new(1, 0),
    Plane::new(2, 0),
    Plane::new(3, 0),
    Plane::new(4, 0),
  ];
  let vf = VideoFrame::new(Dimensions::new(2, 2), PixelFormat::Yuv420p, planes, 2);
  assert_eq!(vf.planes().len(), 2);
  assert_eq!(*vf.plane(0).unwrap().data_ref(), 1);
  assert_eq!(*vf.plane(1).unwrap().data_ref(), 2);
  assert!(vf.plane(2).is_none());
  assert!(vf.plane(7).is_none());
}

#[test]
#[should_panic(expected = "plane_count exceeds the fixed 4-plane array")]
fn video_frame_new_panics_on_plane_count_over_4() {
  let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
  let _ = VideoFrame::new(Dimensions::new(1, 1), PixelFormat::Yuv420p, planes, 5);
}

#[test]
fn video_frame_with_visible_rect_and_color_chain() {
  let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
  let vf = VideoFrame::new(Dimensions::new(8, 8), PixelFormat::Yuv420p, planes, 3)
    .with_visible_rect(Rect::new(0, 0, 6, 6));
  assert_eq!(vf.visible_rect(), Some(Rect::new(0, 0, 6, 6)));
}

// ---------- TimestampedFrame ------------------------------------------

#[test]
fn timestamped_frame_construction_defaults() {
  let tf: TimestampedFrame<&'static str> = TimestampedFrame::new("payload");
  assert!(tf.pts().is_none());
  assert!(tf.duration().is_none());
  assert_eq!(*tf.frame_ref(), "payload");
}

#[test]
fn timestamped_frame_into_frame_consumes() {
  let tf = TimestampedFrame::new(42u32);
  let raw = tf.into_frame();
  assert_eq!(raw, 42);
}

#[test]
fn timestamped_frame_pts_builder() {
  let tb = mediatime::Timebase::new(1, core::num::NonZeroI32::new(1000).unwrap());
  let ts = mediatime::Timestamp::new(1000, tb);
  let tf = TimestampedFrame::new(0u8).with_pts(ts).with_duration(ts);
  assert_eq!(tf.pts(), Some(ts));
  assert_eq!(tf.duration(), Some(ts));
}

#[test]
fn timestamped_frame_wraps_video_frame() {
  let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
  let vf = VideoFrame::new(Dimensions::new(4, 4), PixelFormat::Yuv420p, planes, 3);
  let tf = TimestampedFrame::new(vf);
  assert_eq!(tf.frame_ref().dimensions(), Dimensions::new(4, 4));
}

// ---------- Rational --------------------------------------------------

#[test]
fn rational_default_is_one_over_one() {
  let r = Rational::default();
  assert_eq!(r.num(), 1);
  assert_eq!(r.den().get(), 1);
  assert!(!r.is_zero());
}

#[test]
fn rational_construction_builders_and_is_zero() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let r = Rational::new(30000, nz(1001));
  assert_eq!(r.num(), 30000);
  assert_eq!(r.den().get(), 1001);
  assert!(!r.is_zero());
  let z = Rational::new(0, nz(1));
  assert!(z.is_zero());
  let r2 = Rational::default().with_num(24).with_den(nz(1));
  assert_eq!((r2.num(), r2.den().get()), (24, 1));
  let mut r3 = Rational::default();
  r3.set_num(16).set_den(nz(9));
  assert_eq!((r3.num(), r3.den().get()), (16, 9));
}

#[cfg(feature = "std")]
#[test]
fn rational_display() {
  let nz = core::num::NonZeroI64::new(1001).unwrap();
  assert_eq!(std::format!("{}", Rational::new(30000, nz)), "30000/1001");
}

// ---------- Rational sign / width invariants --------------------------
//
// `num`/`den` were `u32`/`NonZeroU32`, where the types made every
// invariant unrepresentable. Under `i64`/`NonZeroI64` only
// "denominator is non-zero" is still enforced by the type; the sign
// half moved into `new`, so it needs pinning here.

#[test]
fn rational_rejects_negative_numerator() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  assert!(Rational::try_new(-1, nz(1)).is_none());
  assert!(Rational::try_new(i64::MIN, nz(1)).is_none());
  // `0` is a legal degenerate ratio (FFmpeg's "unknown" `0/1`).
  assert!(Rational::try_new(0, nz(1)).is_some());
}

#[test]
fn rational_rejects_negative_denominator() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  assert!(Rational::try_new(1, nz(-1)).is_none());
  assert!(Rational::try_new(1, nz(i64::MIN)).is_none());
}

#[test]
fn rational_zero_denominator_is_unrepresentable() {
  // Not a runtime check in `new` — `NonZeroI64` has no zero value at
  // all, so the state cannot be constructed to be rejected.
  assert!(core::num::NonZeroI64::new(0).is_none());
}

#[test]
#[should_panic(expected = "rational numerator must not be negative")]
fn rational_new_panics_on_negative_numerator() {
  let _ = Rational::new(-1, DEN_ONE);
}

#[test]
#[should_panic(expected = "rational denominator must be positive")]
fn rational_new_panics_on_negative_denominator() {
  let nz = core::num::NonZeroI64::new(-2).unwrap();
  let _ = Rational::new(1, nz);
}

#[test]
#[should_panic(expected = "rational numerator must not be negative")]
fn rational_set_num_routes_through_new() {
  // The four mutators are the invariant hole a direct field
  // assignment would leave open; each goes through `new`.
  let mut r = Rational::default();
  r.set_num(-1);
}

#[test]
#[should_panic(expected = "rational denominator must be positive")]
fn rational_set_den_routes_through_new() {
  let mut r = Rational::default();
  r.set_den(core::num::NonZeroI64::new(-3).unwrap());
}

#[test]
#[should_panic(expected = "rational numerator must not be negative")]
fn rational_with_num_routes_through_new() {
  let _ = Rational::default().with_num(-1);
}

#[test]
#[should_panic(expected = "rational denominator must be positive")]
fn rational_with_den_routes_through_new() {
  let _ = Rational::default().with_den(core::num::NonZeroI64::new(-3).unwrap());
}

#[test]
fn rational_accepts_i64_max_at_both_positions() {
  let nz = core::num::NonZeroI64::new(i64::MAX).unwrap();
  let r = Rational::new(i64::MAX, nz);
  assert_eq!(r.num(), i64::MAX);
  assert_eq!(r.den().get(), i64::MAX);
}

#[test]
fn rational_accepts_values_above_u32_max() {
  // The capability the widening buys: a numerator (and denominator)
  // the previous `u32` representation could not hold at all.
  let big = i64::from(u32::MAX) + 1;
  let nz = core::num::NonZeroI64::new(big).unwrap();
  let r = Rational::new(big, nz);
  assert_eq!(r.num(), big);
  assert_eq!(r.den().get(), big);
  // And through the semantic wrappers, which carry no width of their own.
  let sar = SampleAspectRatio::new(big, nz);
  assert_eq!((sar.num(), sar.den().get()), (big, big));
  assert_eq!(FrameRate::new(r, false).rate(), r);
}

#[test]
fn sample_aspect_ratio_fallible_path_is_try_new_plus_from() {
  // `SampleAspectRatio::new` panics like `Rational::new`; the
  // fallible route is the existing `From<Rational>`.
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let ok = Rational::try_new(40, nz(33)).map(SampleAspectRatio::from);
  assert_eq!(ok, Some(SampleAspectRatio::new(40, nz(33))));
  let bad = Rational::try_new(-40, nz(33)).map(SampleAspectRatio::from);
  assert!(bad.is_none());
}

// ---------- SampleAspectRatio ↔ Rational interop ----------------------

#[test]
fn sample_aspect_ratio_rational_interop() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let sar = SampleAspectRatio::new(40, nz(33));
  let via_method: Rational = sar.as_rational();
  let via_from: Rational = Rational::from(sar);
  let via_into: Rational = sar.into();
  assert_eq!(via_method, Rational::new(40, nz(33)));
  assert_eq!(via_method, via_from);
  assert_eq!(via_from, via_into);
  // Default 1:1 SAR maps to the 1/1 Rational default.
  assert_eq!(
    SampleAspectRatio::default().as_rational(),
    Rational::default()
  );
}

#[test]
fn sample_aspect_ratio_rational_round_trip_both_ways() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  // SAR -> Rational -> SAR
  let sar = SampleAspectRatio::new(40, nz(33));
  let r: Rational = sar.into();
  let back: SampleAspectRatio = r.into();
  assert_eq!(back, sar);
  assert_eq!(sar.rational(), r);
  assert_eq!(sar.rational(), sar.as_rational());
  // Rational -> SAR -> Rational
  let r2 = Rational::new(16, nz(9));
  let s2 = SampleAspectRatio::from(r2);
  assert_eq!((s2.num(), s2.den().get()), (16, 9));
  assert_eq!(Rational::from(s2), r2);
}

#[test]
fn sample_aspect_ratio_default_is_one_to_one() {
  let d = SampleAspectRatio::default();
  assert_eq!((d.num(), d.den().get()), (1, 1));
  assert!(d.is_square());
  assert_eq!(d, SampleAspectRatio::new(1, DEN_ONE));
}

#[test]
fn sample_aspect_ratio_eq_and_hash_parity() {
  use core::hash::{Hash, Hasher};
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let a = SampleAspectRatio::new(40, nz(33));
  let b = SampleAspectRatio::default().with_num(40).with_den(nz(33));
  assert_eq!(a, b);

  fn h(s: &SampleAspectRatio) -> u64 {
    // `no_std`-friendly deterministic hasher (FNV-1a).
    struct Fnv(u64);
    impl Hasher for Fnv {
      fn finish(&self) -> u64 {
        self.0
      }
      fn write(&mut self, bytes: &[u8]) {
        for &x in bytes {
          self.0 = (self.0 ^ x as u64).wrapping_mul(0x0100_0000_01b3);
        }
      }
    }
    let mut hasher = Fnv(0xcbf2_9ce4_8422_2325);
    s.hash(&mut hasher);
    hasher.finish()
  }
  assert_eq!(h(&a), h(&b));
}

// ---------- FrameRate -------------------------------------------------

#[test]
fn frame_rate_default_is_one_over_one_cfr() {
  let fr = FrameRate::default();
  assert_eq!(fr.rate(), Rational::default());
  assert!(!fr.is_vfr());
}

#[test]
fn frame_rate_construction_and_builders() {
  let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
  let ntsc = Rational::new(30000, nz(1001));
  let fr = FrameRate::new(ntsc, false);
  assert_eq!(fr.rate(), ntsc);
  assert!(!fr.is_vfr());
  let vfr = FrameRate::default().with_rate(ntsc).with_is_vfr();
  assert_eq!(vfr.rate(), ntsc);
  assert!(vfr.is_vfr());
  let mut fr3 = FrameRate::default();
  fr3.set_rate(Rational::new(25, nz(1))).set_is_vfr();
  assert_eq!(fr3.rate(), Rational::new(25, nz(1)));
  assert!(fr3.is_vfr());
  // raw-wrapper + clear forms
  let fr4 = FrameRate::default().maybe_is_vfr(true);
  assert!(fr4.is_vfr());
  let mut fr5 = FrameRate::default();
  fr5.update_is_vfr(true);
  assert!(fr5.is_vfr());
  fr5.clear_is_vfr();
  assert!(!fr5.is_vfr());
}

// ---------- FieldOrder ------------------------------------------------

#[test]
fn field_order_default_is_unknown_and_as_str() {
  assert_eq!(FieldOrder::default(), FieldOrder::Unknown);
  assert_eq!(FieldOrder::Unknown.as_str(), "unknown");
  // FFmpeg names its own absence, so `"unknown"` round-trips exactly —
  // it is a variant, not the old payload-collapsing escape.
  assert_eq!("unknown".parse(), Ok(FieldOrder::Unknown));
  assert_eq!(FieldOrder::Progressive.as_str(), "progressive");
  assert_eq!(FieldOrder::Tt.as_str(), "tt");
  assert_eq!(FieldOrder::Bb.as_str(), "bb");
  assert_eq!(FieldOrder::Tb.as_str(), "tb");
  assert_eq!(FieldOrder::Bt.as_str(), "bt");
  assert!(FieldOrder::Progressive.is_progressive());
}

#[test]
fn field_order_u32_round_trip_and_escape() {
  for f in [
    FieldOrder::Unknown,
    FieldOrder::Progressive,
    FieldOrder::Tt,
    FieldOrder::Bb,
    FieldOrder::Tb,
    FieldOrder::Bt,
  ] {
    assert_eq!(FieldOrder::from_u32(f.to_u32().unwrap()), Some(f));
  }
  assert_eq!(FieldOrder::from_u32(1), Some(FieldOrder::Progressive));
  assert_eq!(FieldOrder::from_u32(5), Some(FieldOrder::Bt));
  // FFmpeg's own UNKNOWN sentinel (0) decodes to the named variant.
  assert_eq!(FieldOrder::from_u32(0), Some(FieldOrder::Unknown));
  assert_eq!(FieldOrder::from_u32(99), None);
}

// ---------- StereoMode ------------------------------------------------

#[test]
fn stereo_mode_default_is_mono_and_as_str() {
  assert_eq!(StereoMode::default(), StereoMode::Mono);
  assert_eq!(StereoMode::Mono.as_str(), "mono");
  assert_eq!(StereoMode::SideBySide.as_str(), "side-by-side");
  assert_eq!(StereoMode::Columns.as_str(), "columns");
  assert!(StereoMode::Mono.is_mono());
}

#[test]
fn stereo_mode_u32_round_trip_and_escape() {
  for s in [
    StereoMode::Mono,
    StereoMode::SideBySide,
    StereoMode::TopBottom,
    StereoMode::FrameSequence,
    StereoMode::Checkerboard,
    StereoMode::SideBySideQuincunx,
    StereoMode::Lines,
    StereoMode::Columns,
  ] {
    assert_eq!(StereoMode::from_u32(s.to_u32().unwrap()), Some(s));
  }
  assert_eq!(StereoMode::from_u32(0), Some(StereoMode::Mono));
  assert_eq!(StereoMode::from_u32(7), Some(StereoMode::Columns));
  assert_eq!(StereoMode::from_u32(99), None);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn stereo_mode_escape_keeps_its_name() {
  let vendor = StereoMode::other("Anaglyph");
  assert_eq!(vendor.as_str(), "anaglyph");
  assert_eq!(vendor.to_u32(), None);
  assert_eq!("anaglyph".parse(), Ok(vendor));
}

/// Every named variant of the three coded frame enums must survive
/// `as_str()` → `FromStr`, with no shared slugs.
#[test]
fn every_named_frame_enum_variant_round_trips_through_its_slug() {
  macro_rules! sweep {
    ($ty:ty) => {{
      let mut named = 0usize;
      let mut codes = [0u32; 32];
      for code in 0..=1024u32 {
        let Some(value) = <$ty>::from_u32(code) else {
          continue;
        };
        let slug = value.as_str();
        assert_eq!(
          slug.parse::<$ty>(),
          Ok(value.clone()),
          "{} slug {slug:?} does not parse back to {value:?}",
          stringify!($ty)
        );
        assert!(
          !slug.bytes().any(|b| b.is_ascii_uppercase()),
          "{} slug {slug:?} is not lowercase-canonical",
          stringify!($ty)
        );
        {
          let mut upper = [0u8; 64];
          let n = slug.len();
          upper[..n].copy_from_slice(slug.as_bytes());
          upper[..n].make_ascii_uppercase();
          let upper = core::str::from_utf8(&upper[..n]).unwrap();
          assert_eq!(
            upper.parse::<$ty>(),
            Ok(value.clone()),
            "{} does not fold {upper:?} onto {slug:?}",
            stringify!($ty)
          );
        }
        for prior in codes.iter().take(named) {
          let prior = <$ty>::from_u32(*prior).expect("recorded code names a variant");
          assert_ne!(
            prior.as_str(),
            slug,
            "{} has two variants spelled {slug:?}",
            stringify!($ty)
          );
        }
        codes[named] = code;
        named += 1;
      }
      assert!(
        named > 0,
        "{} sweep found no named variants",
        stringify!($ty)
      );
    }};
  }

  sweep!(Rotation);
  sweep!(FieldOrder);
  sweep!(StereoMode);
}

#[test]
fn field_order_names_its_own_unknown() {
  // `"unknown"` is a *name* now, not a payload-collapsing arm: on
  // `FieldOrder` it is FFmpeg's own variant.
  assert_eq!("unknown".parse(), Ok(FieldOrder::Unknown));
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn frame_enum_escape_keeps_the_name_it_was_given() {
  // Elsewhere `"unknown"` rides the escape like any other name this
  // build does not enumerate.
  assert_eq!("unknown".parse(), Ok(Rotation::other("unknown")));
  assert_eq!("unknown".parse(), Ok(StereoMode::other("unknown")));
  assert_eq!(
    "not-a-rotation".parse::<Rotation>().unwrap().as_str(),
    "not-a-rotation"
  );
}

/// The geometry types render an injective form, so `FromStr` is a true
/// inverse of `Display` for every value — not only the named ones.
// `std::format!` needs the allocator; these types themselves are
// available at the no-alloc tier, where the round trip is untestable.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn geometry_display_round_trips_through_from_str() {
  use core::num::NonZeroI64;

  let nz = |n: i64| NonZeroI64::new(n).unwrap();

  for dims in [
    Dimensions::default(),
    Dimensions::new(1920, 1080),
    Dimensions::new(u32::MAX, u32::MAX),
  ] {
    assert_eq!(std::format!("{dims}").parse(), Ok(dims));
  }

  for ratio in [
    Rational::default(),
    Rational::new(30_000, nz(1001)),
    Rational::new(0, nz(1)),
    Rational::new(i64::MAX, nz(i64::MAX)),
  ] {
    assert_eq!(std::format!("{ratio}").parse(), Ok(ratio));
  }

  for sar in [
    SampleAspectRatio::default(),
    SampleAspectRatio::new(40, nz(33)),
    SampleAspectRatio::new(16, nz(9)),
  ] {
    assert_eq!(std::format!("{sar}").parse(), Ok(sar));
  }
}

/// The separators are part of each type's contract: a SAR is written
/// `a:b` and a bare ratio `a/b`, so neither accepts the other's form.
#[test]
fn geometry_separators_are_not_interchangeable() {
  assert!("40/33".parse::<SampleAspectRatio>().is_err());
  assert!("40:33".parse::<Rational>().is_err());
  assert!("1920X1080".parse::<Dimensions>().is_err());
}

/// Parsing routes through `Rational::try_new`, so it cannot mint a
/// value the constructor rejects — the invariant has exactly one gate.
#[test]
fn geometry_parsing_cannot_bypass_the_constructor_invariant() {
  // `num < 0` and `den <= 0` are what `Rational::try_new` refuses.
  assert!("-5/4".parse::<Rational>().is_err());
  assert!("5/-4".parse::<Rational>().is_err());
  assert!("5/0".parse::<Rational>().is_err());
  assert!("-1:1".parse::<SampleAspectRatio>().is_err());

  // A ratio fails for two reasons a caller reports differently, so
  // unlike the name vocabularies its error carries which.
  assert_eq!(
    "-5/4".parse::<Rational>().unwrap_err().kind(),
    RatioParseKind::OutOfRange
  );
  assert_eq!(
    "not-a-ratio".parse::<Rational>().unwrap_err().kind(),
    RatioParseKind::Malformed
  );
  assert_eq!(
    "-1:1".parse::<SampleAspectRatio>().unwrap_err().kind(),
    RatioParseKind::OutOfRange
  );
}

#[test]
fn geometry_rejects_malformed_input() {
  for bad in ["", "1920", "1920x", "x1080", "1920x1080x1", "axb", " 1x2"] {
    assert!(
      bad.parse::<Dimensions>().is_err(),
      "{bad:?} should not parse as Dimensions"
    );
  }
  for bad in ["", "30000", "30000/", "/1001", "a/b", "1/2/3"] {
    assert!(
      bad.parse::<Rational>().is_err(),
      "{bad:?} should not parse as Rational"
    );
  }
}
