// ── serde / arbitrary / quickcheck wiring ──
//
// These five types sat outside every feature matrix while their
// `frame::` siblings were in all of them, and `lib.rs` states that
// arbitrary "mirrors the surface covered by serde" — an invariant that
// was true only because both were absent. The three matrices move
// together or that sentence stops being true.

#[cfg(all(feature = "serde", feature = "std"))]
#[test]
fn raw_vocabulary_round_trips_through_serde() {
  fn round_trip<T>(v: &T)
  where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
  {
    let json = serde_json::to_string(v).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(*v, back, "round-trip mismatch via {json}");
  }

  // The enums ride their slug, like every other vocabulary.
  for p in [
    BayerPattern::Rggb,
    BayerPattern::Bggr,
    BayerPattern::Grbg,
    BayerPattern::Gbrg,
  ] {
    round_trip(&p);
  }
  assert_eq!(
    serde_json::to_string(&BayerPattern::Rggb).unwrap(),
    "\"rggb\""
  );
  round_trip(&BayerDemosaic::Bilinear);
  for c in [WbChannel::R, WbChannel::G, WbChannel::B] {
    round_trip(&c);
  }
  // Closed vocabularies: an unrecognised slug is an error, not an
  // invented value.
  assert!(serde_json::from_str::<BayerPattern>("\"xtrans\"").is_err());
  assert!(serde_json::from_str::<WbChannel>("\"y\"").is_err());

  round_trip(&WhiteBalance::neutral());
  round_trip(&WhiteBalance::try_new(2.1, 1.0, 1.45).unwrap());
  round_trip(&ColorCorrectionMatrix::identity());
}

/// The float structs deserialize through `try_new`, so the wire cannot
/// mint a value the constructor refuses. A NaN gain or a pathological
/// coefficient would otherwise propagate through the fused per-pixel
/// transform and silently blacken unrelated channels.
#[cfg(all(feature = "serde", feature = "std"))]
#[test]
fn raw_float_structs_validate_on_deserialize() {
  assert!(serde_json::from_str::<WhiteBalance>(r#"{"r":1.0,"g":1.0,"b":1.0}"#).is_ok());
  assert!(serde_json::from_str::<WhiteBalance>(r#"{"r":-1.0,"g":1.0,"b":1.0}"#).is_err());
  // JSON has no NaN literal; a non-finite gain arrives as `null`, which
  // the `f32` deserializer itself refuses — either way it never lands.
  assert!(serde_json::from_str::<WhiteBalance>(r#"{"r":null,"g":1.0,"b":1.0}"#).is_err());

  assert!(serde_json::from_str::<ColorCorrectionMatrix>("[[1,0,0],[0,1,0],[0,0,1]]").is_ok());
  assert!(
    serde_json::from_str::<ColorCorrectionMatrix>("[[1e30,0,0],[0,1,0],[0,0,1]]").is_err(),
    "a coefficient past MAX_COEFFICIENT_ABS must be rejected"
  );
}

use super::*;

#[test]
fn variants_construct_and_compare() {
  assert_eq!(BayerPattern::Bggr, BayerPattern::Bggr);
  assert_ne!(BayerPattern::Bggr, BayerPattern::Rggb);
}

#[test]
fn is_variant_helpers_work() {
  assert!(BayerPattern::Bggr.is_bggr());
  assert!(!BayerPattern::Bggr.is_rggb());
}

#[cfg(feature = "std")]
#[test]
fn copy_and_hash() {
  use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
  };
  let p = BayerPattern::Grbg;
  let _copy = p; // doesn't move
  let mut h = DefaultHasher::new();
  p.hash(&mut h);
  let _ = h.finish();
}

#[cfg(feature = "std")]
#[test]
fn as_str_matches_display() {
  use std::format;
  for v in [
    BayerPattern::Bggr,
    BayerPattern::Rggb,
    BayerPattern::Grbg,
    BayerPattern::Gbrg,
  ] {
    assert_eq!(v.as_str(), format!("{v}"));
  }
  assert_eq!(BayerPattern::Bggr.as_str(), "bggr");
}

/// The three closed bayer vocabularies round-trip through their slugs.
/// The `match` arms make the variant lists exhaustive by construction —
/// adding a variant stops this compiling until the list is updated.
#[test]
fn bayer_vocabularies_round_trip_through_their_slugs() {
  const fn _pattern_is_exhaustive(p: BayerPattern) {
    match p {
      BayerPattern::Bggr | BayerPattern::Rggb | BayerPattern::Grbg | BayerPattern::Gbrg => (),
    }
  }
  const fn _demosaic_is_exhaustive(d: BayerDemosaic) {
    match d {
      BayerDemosaic::Bilinear => (),
    }
  }
  const fn _channel_is_exhaustive(c: WbChannel) {
    match c {
      WbChannel::R | WbChannel::G | WbChannel::B => (),
    }
  }

  for pattern in [
    BayerPattern::Bggr,
    BayerPattern::Rggb,
    BayerPattern::Grbg,
    BayerPattern::Gbrg,
  ] {
    assert_eq!(pattern.as_str().parse(), Ok(pattern));
  }
  // Only one algorithm is wired up today, so this is a single value
  // rather than a loop; `_demosaic_is_exhaustive` above is what fails
  // when a second one lands.
  assert_eq!(
    BayerDemosaic::Bilinear.as_str().parse(),
    Ok(BayerDemosaic::Bilinear)
  );
  for channel in [WbChannel::R, WbChannel::G, WbChannel::B] {
    assert_eq!(channel.as_str().parse(), Ok(channel));
  }
}

#[test]
fn bayer_vocabularies_reject_anything_else() {
  let err: ParseBayerPatternError = "rgbg".parse::<BayerPattern>().unwrap_err();
  let _ = err;
  // Case folds.
  assert_eq!("BGGR".parse(), Ok(BayerPattern::Bggr));
  assert_eq!("Bilinear".parse(), Ok(BayerDemosaic::Bilinear));
  assert_eq!("R".parse(), Ok(WbChannel::R));
  assert!("".parse::<WbChannel>().is_err());

  // `BayerDemosaic`'s slug was the crate's one capitalised spelling.
  // It is lowercase now, like every other slug, and the parse gate
  // folds so the old spelling still reads.
  assert_eq!(BayerDemosaic::Bilinear.as_str(), "bilinear");
  assert_eq!("bilinear".parse(), Ok(BayerDemosaic::Bilinear));
}

/// The three RAW vocabularies are lowercase-canonical, collision-free
/// once folded, and read case-insensitively — the same law the coded
/// vocabularies are swept for. The variant lists are exhaustive **by
/// construction**: a wildcard-free `match` stops compiling when a
/// variant is added.
#[test]
fn raw_vocabularies_are_lowercase_canonical_and_fold() {
  fn exhaustive_pattern(p: BayerPattern) -> &'static str {
    match p {
      BayerPattern::Rggb => "rggb",
      BayerPattern::Bggr => "bggr",
      BayerPattern::Grbg => "grbg",
      BayerPattern::Gbrg => "gbrg",
    }
  }
  fn exhaustive_demosaic(d: BayerDemosaic) -> &'static str {
    match d {
      BayerDemosaic::Bilinear => "bilinear",
    }
  }
  fn exhaustive_channel(c: WbChannel) -> &'static str {
    match c {
      WbChannel::R => "r",
      WbChannel::G => "g",
      WbChannel::B => "b",
    }
  }

  let patterns = [
    BayerPattern::Rggb,
    BayerPattern::Bggr,
    BayerPattern::Grbg,
    BayerPattern::Gbrg,
  ];
  for (i, p) in patterns.iter().enumerate() {
    assert_eq!(p.as_str(), exhaustive_pattern(*p));
    assert!(!p.as_str().bytes().any(|b| b.is_ascii_uppercase()));
    for q in &patterns[..i] {
      assert!(
        !q.as_str().eq_ignore_ascii_case(p.as_str()),
        "two Bayer patterns fold onto {:?}",
        p.as_str()
      );
    }
  }
  assert_eq!(
    BayerDemosaic::Bilinear.as_str(),
    exhaustive_demosaic(BayerDemosaic::Bilinear)
  );

  let channels = [WbChannel::R, WbChannel::G, WbChannel::B];
  for (i, c) in channels.iter().enumerate() {
    assert_eq!(c.as_str(), exhaustive_channel(*c));
    assert!(!c.as_str().bytes().any(|b| b.is_ascii_uppercase()));
    for d in &channels[..i] {
      assert!(!d.as_str().eq_ignore_ascii_case(c.as_str()));
    }
  }

  // Case-insensitive both ways, and `Display` is the exact inverse.
  assert_eq!("RGGB".parse(), Ok(BayerPattern::Rggb));
  assert_eq!("GbRg".parse(), Ok(BayerPattern::Gbrg));
  assert_eq!("BILINEAR".parse(), Ok(BayerDemosaic::Bilinear));
  assert_eq!("B".parse(), Ok(WbChannel::B));
}
