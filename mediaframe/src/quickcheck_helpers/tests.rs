use ::quickcheck::{Arbitrary, Gen};

/// Drives N rounds against a fresh `quickcheck::Gen` for a given `size`.
fn drive<F: FnMut(&mut Gen)>(size: usize, rounds: usize, mut body: F) {
  let mut g = Gen::new(size);
  for _ in 0..rounds {
    body(&mut g);
  }
}

#[test]
fn geo_location_invariant_lat_lon_in_range() {
  drive(64, 256, |g| {
    let geo = crate::capture::GeoLocation::arbitrary(g);
    assert!(
      (-90.0..=90.0).contains(&geo.lat()),
      "lat out of range: {}",
      geo.lat()
    );
    assert!(
      (-180.0..=180.0).contains(&geo.lon()),
      "lon out of range: {}",
      geo.lon()
    );
    if let Some(alt) = geo.altitude() {
      assert!(
        alt.is_finite(),
        "altitude must be finite when Some, got {alt}"
      );
    }
  });
}

#[test]
fn fingerprint_invariant_algorithm_non_empty() {
  drive(64, 256, |g| {
    let fp = crate::audio::Fingerprint::arbitrary(g);
    assert!(!fp.algorithm().is_empty(), "algorithm must be non-empty");
  });
}

#[test]
fn cover_art_invariant_mime_and_data_non_empty() {
  drive(64, 256, |g| {
    let c = crate::audio::CoverArt::arbitrary(g);
    assert!(!c.mime().is_empty(), "mime must be non-empty");
    assert!(!c.data().is_empty(), "data must be non-empty");
  });
}

#[test]
fn smoke_yields_values_for_representative_types() {
  drive(64, 64, |g| {
    let _ = crate::codec::VideoCodec::arbitrary(g);
    let _ = crate::color::Info::arbitrary(g);
    let _ = crate::frame::FrameRate::arbitrary(g);
    let _ = crate::lang::Language::arbitrary(g);
    let _ = crate::disposition::TrackDisposition::arbitrary(g);
    let _ = crate::audio::Tags::arbitrary(g);
  });
}

#[test]
fn reachability_small_coded_enums_hit_all_named() {
  use ::std::collections::HashSet;
  let mut br: HashSet<crate::audio::BitRateMode> = HashSet::new();
  let mut to: HashSet<crate::subtitle::TrackOrigin> = HashSet::new();
  drive(64, 2048, |g| {
    br.insert(crate::audio::BitRateMode::arbitrary(g));
    to.insert(crate::subtitle::TrackOrigin::arbitrary(g));
  });
  assert_eq!(br.len(), 3, "BitRateMode coverage: {br:?}");
  assert_eq!(to.len(), 3, "TrackOrigin coverage: {to:?}");
}

#[test]
fn reachability_rotation_hits_named_and_escape() {
  use crate::frame::Rotation;
  let mut saw_named = false;
  let mut saw_other = false;
  drive(64, 2048, |g| match Rotation::arbitrary(g) {
    Rotation::Other(_) => saw_other = true,
    _ => saw_named = true,
  });
  assert!(
    saw_named && saw_other,
    "Rotation missing arms: named={saw_named} other={saw_other}"
  );
}

// Every one of `SampleFormat`'s 12 named variants must be reachable —
// plus the `Other(_)` escape arm. A weaker "some named appears" check
// (Codex round-2 finding) would pass even if half the slug list were
// missing.
#[test]
fn reachability_sample_format_all_named_plus_arms() {
  use crate::audio::SampleFormat;
  use ::std::collections::HashSet;
  let mut named: HashSet<::std::string::String> = HashSet::new();
  let mut saw_other = false;
  drive(64, 4096, |g| match SampleFormat::arbitrary(g) {
    SampleFormat::Other(_) => saw_other = true,
    other => {
      named.insert(other.as_str().to_string());
    }
  });
  assert_eq!(
    named.len(),
    12,
    "missing named SampleFormat variants; observed: {named:?}"
  );
  assert!(saw_other, "SampleFormat: never observed `Other(_)`");
}

// The range-weighted large coded enums must reach a broad set of named
// codes — `arb_via_code!` (uniform `u32`) hit the named range for
// `Matrix` / `Primaries` essentially never (Codex round-2 finding).
#[test]
fn reachability_range_weighted_enums_hit_named_codes() {
  use ::std::collections::HashSet;
  let mut matrix: HashSet<u32> = HashSet::new();
  let mut primaries: HashSet<u32> = HashSet::new();
  let mut transfer: HashSet<u32> = HashSet::new();
  let mut pixel: HashSet<u32> = HashSet::new();
  drive(64, 8192, |g| {
    matrix.extend(crate::color::Matrix::arbitrary(g).to_u32());
    primaries.extend(crate::color::Primaries::arbitrary(g).to_u32());
    transfer.extend(crate::color::Transfer::arbitrary(g).to_u32());
    pixel.extend(crate::pixel_format::PixelFormat::arbitrary(g).to_u32());
  });
  let in_range = |s: &HashSet<u32>, max: u32| s.iter().filter(|&&c| c <= max).count();
  assert!(
    in_range(&matrix, 17) >= 3,
    "Matrix named-range coverage too low: {matrix:?}"
  );
  // `Matrix::Bt601` is the domain-extension variant at `DOMAIN_EXT_BASE`
  // — it is in the curated slug list precisely so the generator reaches
  // it; the numeric generator it replaced hit it once in 8.6 billion.
  assert!(
    matrix.contains(&crate::color::DOMAIN_EXT_BASE),
    "Matrix::Bt601 (DOMAIN_EXT_BASE) never generated"
  );
  assert!(
    in_range(&primaries, 22) >= 3,
    "Primaries named-range coverage too low: {primaries:?}"
  );
  assert!(
    in_range(&transfer, 18) >= 3,
    "Transfer named-range coverage too low: {transfer:?}"
  );
  // PixelFormat draws from a curated 6-slug list plus the escape.
  assert!(
    in_range(&pixel, 947) >= 3,
    "PixelFormat named-range coverage too low: {} distinct",
    in_range(&pixel, 947)
  );
}

#[test]
fn generated_vocabulary_values_round_trip_through_their_name() {
  drive(64, 128, |g| {
    macro_rules! rt {
      ($ty:path) => {{
        let v = <$ty>::arbitrary(g);
        assert_eq!(v.as_str().parse::<$ty>(), Ok(v.clone()), "{v:?}");
      }};
    }
    rt!(crate::color::Matrix);
    rt!(crate::pixel_format::PixelFormat);
    rt!(crate::frame::Rotation);
    let d = crate::disposition::TrackDisposition::arbitrary(g);
    assert_eq!(
      crate::disposition::TrackDisposition::from_u32(d.to_u32()),
      d
    );
  });
}

// `Tags.language` (`Option<Language>`) was omitted from the generator
// (Codex round-4 finding) — every generated `Tags` had `language == None`.
// Both the absent (`None`) and present (`Some(_)`) states must be
// reachable. `Language` has no empty value, so `Some` is unconditionally
// wire-canonical.
#[test]
fn reachability_tags_language_hits_none_and_some() {
  let mut saw_none = false;
  let mut saw_some = false;
  drive(64, 1024, |g| {
    match crate::audio::Tags::arbitrary(g).language() {
      None => saw_none = true,
      Some(_) => saw_some = true,
    }
  });
  assert!(saw_none, "Tags.language never generated `None`");
  assert!(saw_some, "Tags.language never generated `Some(_)`");
}

// Arbitrary-generated values must survive a serde round-trip unchanged
// (Codex round-4 finding). Every helper here produces only *canonical*
// values — string construction routes through `FromStr`, so an `Other`
// can never hold a named slug, and `from_u32` codes are canonical.
#[cfg(feature = "serde")]
#[test]
fn arbitrary_values_survive_serde_round_trip() {
  drive(64, 4096, |g| {
    let sf = crate::audio::SampleFormat::arbitrary(g);
    let json = serde_json::to_string(&sf).unwrap();
    let back: crate::audio::SampleFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sf, "SampleFormat lost identity via serde: {json}");

    let vc = crate::codec::VideoCodec::arbitrary(g);
    let json = serde_json::to_string(&vc).unwrap();
    let back: crate::codec::VideoCodec = serde_json::from_str(&json).unwrap();
    assert_eq!(back, vc, "VideoCodec lost identity via serde: {json}");

    // `Loudness` is the serde-derived composite struct with `f32` fields —
    // the round-trip only holds if every field is finite (Codex round-5).
    let ld = crate::audio::Loudness::arbitrary(g);
    let json = serde_json::to_string(&ld).unwrap();
    let back: crate::audio::Loudness = serde_json::from_str(&json).unwrap();
    assert_eq!(back, ld, "Loudness lost identity via serde: {json}");

    // The RAW-development structs deserialize through `try_new`, so a
    // generated value that the constructor would refuse fails here.
    #[cfg(feature = "bayer")]
    {
      let wb = crate::frame::WhiteBalance::arbitrary(g);
      let json = serde_json::to_string(&wb).unwrap();
      let back: crate::frame::WhiteBalance = serde_json::from_str(&json).unwrap();
      assert_eq!(back, wb, "WhiteBalance lost identity via serde: {json}");

      let ccm = crate::frame::ColorCorrectionMatrix::arbitrary(g);
      let json = serde_json::to_string(&ccm).unwrap();
      let back: crate::frame::ColorCorrectionMatrix = serde_json::from_str(&json).unwrap();
      assert_eq!(
        back, ccm,
        "ColorCorrectionMatrix lost identity via serde: {json}"
      );

      let bp = crate::frame::BayerPattern::arbitrary(g);
      let json = serde_json::to_string(&bp).unwrap();
      let back: crate::frame::BayerPattern = serde_json::from_str(&json).unwrap();
      assert_eq!(back, bp, "BayerPattern lost identity via serde: {json}");
    }
  });
}
/// Parsing is **idempotent through the text form**: whatever a value
/// renders is a spelling that parses back to that same value. This is
/// the property the case-folding gate buys — without it a value could
/// render a spelling its own parser does not accept, which is exactly
/// what `BayerDemosaic`'s capitalised slug did.
///
/// Driven over arbitrary strings rather than a slug list, so the escape
/// arm is covered as heavily as the named ones.
#[test]
fn parse_is_idempotent_through_the_text_form() {
  macro_rules! idempotent {
    ($ty:path, $s:expr) => {{
      let once: $ty = $s.parse().unwrap();
      let twice: $ty = once.as_str().parse().unwrap();
      assert_eq!(
        once,
        twice,
        "{} renders {:?}, which does not parse back to it",
        stringify!($ty),
        once.as_str()
      );
    }};
  }
  drive(1234, 2048, |g| {
    let s = super::arb_string(g);
    idempotent!(crate::codec::VideoCodec, s);
    idempotent!(crate::codec::AudioCodec, s);
    idempotent!(crate::codec::SubtitleCodec, s);
    idempotent!(crate::container::Format, s);
    idempotent!(crate::subtitle::Format, s);
    idempotent!(crate::audio::ChannelLayout, s);
    idempotent!(crate::audio::SampleFormat, s);
    idempotent!(crate::audio::ContainerFormat, s);
    idempotent!(crate::color::Matrix, s);
    idempotent!(crate::color::Primaries, s);
    idempotent!(crate::color::Transfer, s);
    idempotent!(crate::color::DynamicRange, s);
    idempotent!(crate::color::ChromaLocation, s);
    idempotent!(crate::color::DcpTargetGamut, s);
    idempotent!(crate::pixel_format::PixelFormat, s);
    idempotent!(crate::frame::Rotation, s);
    idempotent!(crate::frame::FieldOrder, s);
    idempotent!(crate::frame::StereoMode, s);
  });
}

/// …and folding is what makes one name one value: an arbitrary string
/// and its uppercase spelling parse to the *same* value, so the derived
/// `Eq` / `Hash` never split a name across two entries.
#[test]
fn one_name_is_one_value_whatever_its_case() {
  macro_rules! folds {
    ($ty:path, $s:expr) => {{
      let plain: $ty = $s.parse().unwrap();
      let shouted: $ty = $s.to_ascii_uppercase().parse().unwrap();
      assert_eq!(plain, shouted, "{} split a name by case", stringify!($ty));
    }};
  }
  drive(4321, 2048, |g| {
    let s = super::arb_string(g);
    folds!(crate::codec::VideoCodec, s);
    folds!(crate::container::Format, s);
    folds!(crate::audio::SampleFormat, s);
    folds!(crate::color::Matrix, s);
    folds!(crate::pixel_format::PixelFormat, s);
    folds!(crate::frame::StereoMode, s);
  });
}

/// Both halves of the 5.x pair are reachable from the curated seeds.
///
/// FFmpeg hands the unqualified `"5.0"` / `"5.1"` to the **back**-speaker
/// layouts, so a seed list carrying only the short spellings generates
/// `N5Point0Back` / `N5Point1Back` and never their side siblings —
/// which is the state 0.4.0's slug swap left behind until the two
/// qualified spellings were seeded alongside. Assert all four, so
/// trimming the list silently narrows nothing.
#[test]
fn reachability_channel_layout_reaches_both_five_point_pairs() {
  use crate::audio::ChannelLayout;
  use ::std::collections::HashSet;

  let mut seen: HashSet<ChannelLayout> = HashSet::new();
  drive(64, 4096, |g| {
    seen.insert(ChannelLayout::arbitrary(g));
  });

  for wanted in [
    ChannelLayout::N5Point0,
    ChannelLayout::N5Point0Back,
    ChannelLayout::N5Point1,
    ChannelLayout::N5Point1Back,
  ] {
    assert!(
      seen.contains(&wanted),
      "{wanted:?} ({:?}) is unreachable from the curated ChannelLayout seeds",
      wanted.as_str()
    );
  }
}
