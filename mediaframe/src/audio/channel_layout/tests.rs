use super::*;
use ::std::string::ToString;

#[test]
fn every_named_variant_round_trips() {
  for slug in [
    "mono",
    "stereo",
    "2.1",
    "3.0",
    "3.0(back)",
    "3.1",
    "quad",
    "5.0",
    "5.0(side)",
    "5.1",
    "5.1(side)",
    "6.0",
    "6.1",
    "7.0",
    "7.1",
    "hexagonal",
    "octagonal",
    "ambisonic1",
    "ambisonic2",
    "ambisonic3",
  ] {
    let v: ChannelLayout = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug, "round-trip mismatch for `{slug}`");
  }
}

/// Every named variant's slug is the name FFmpeg's
/// `channel_layout_map[]` gives the constant that variant is named
/// after — transcribed from `libavutil/channel_layout.c` at the pinned
/// n9.0 tag.
///
/// This exists because the 5.x arms *look* wrong: FFmpeg gives the
/// unqualified `"5.0"` / `"5.1"` to the **back**-speaker layouts
/// (`AV_CH_LAYOUT_5POINT0_BACK` = `SURROUND|BACK_LEFT|BACK_RIGHT`) and
/// qualifies the side ones. mediaframe had them the other way round
/// until 0.4.0, so an FFmpeg-sourced `"5.1"` landed on the variant whose
/// documentation promised side speakers. Nothing but a transcribed table
/// catches that, so here it is; `ambisonic*` are excluded because FFmpeg
/// models ambisonics as a channel *order*, not a map entry.
#[test]
fn channel_layout_slugs_match_ffmpegs_map() {
  // (variant, `AV_CH_LAYOUT_` suffix it is named after, FFmpeg's name)
  const MAP: &[(ChannelLayout, &str, &str)] = &[
    (ChannelLayout::Mono, "MONO", "mono"),
    (ChannelLayout::Stereo, "STEREO", "stereo"),
    (ChannelLayout::N2Point1, "2POINT1", "2.1"),
    (ChannelLayout::N3Point0, "SURROUND", "3.0"),
    (ChannelLayout::N3Point0Back, "2_1", "3.0(back)"),
    (ChannelLayout::N3Point1, "3POINT1", "3.1"),
    (ChannelLayout::Quad, "QUAD", "quad"),
    (ChannelLayout::N5Point0, "5POINT0", "5.0(side)"),
    (ChannelLayout::N5Point0Back, "5POINT0_BACK", "5.0"),
    (ChannelLayout::N5Point1, "5POINT1", "5.1(side)"),
    (ChannelLayout::N5Point1Back, "5POINT1_BACK", "5.1"),
    (ChannelLayout::N6Point0, "6POINT0", "6.0"),
    (ChannelLayout::N6Point1, "6POINT1", "6.1"),
    (ChannelLayout::N7Point0, "7POINT0", "7.0"),
    (ChannelLayout::N7Point1, "7POINT1", "7.1"),
    (ChannelLayout::Hexagonal, "HEXAGONAL", "hexagonal"),
    (ChannelLayout::Octagonal, "OCTAGONAL", "octagonal"),
  ];
  for (layout, constant, ffmpeg_name) in MAP {
    assert_eq!(
      layout.as_str(),
      *ffmpeg_name,
      "AV_CH_LAYOUT_{constant} is named {ffmpeg_name:?} by FFmpeg, not {:?}",
      layout.as_str()
    );
    assert_eq!(
      ffmpeg_name.parse::<ChannelLayout>().unwrap(),
      *layout,
      "FFmpeg's {ffmpeg_name:?} must read back as the AV_CH_LAYOUT_{constant} variant"
    );
  }
}

/// The four 5.x slugs were swapped in 0.4.0. Pin both readings so a
/// well-meaning "fix" cannot quietly put them back.
#[test]
fn the_unqualified_five_point_slugs_are_the_back_layouts() {
  assert_eq!("5.0".parse(), Ok(ChannelLayout::N5Point0Back));
  assert_eq!("5.1".parse(), Ok(ChannelLayout::N5Point1Back));
  assert_eq!("5.0(side)".parse(), Ok(ChannelLayout::N5Point0));
  assert_eq!("5.1(side)".parse(), Ok(ChannelLayout::N5Point1));
  assert_eq!(ChannelLayout::N5Point0Back.as_str(), "5.0");
  assert_eq!(ChannelLayout::N5Point1Back.as_str(), "5.1");
  assert_eq!(ChannelLayout::N5Point0.as_str(), "5.0(side)");
  assert_eq!(ChannelLayout::N5Point1.as_str(), "5.1(side)");
}

#[test]
fn unknown_layout_lands_in_other() {
  let v: ChannelLayout = "22.2".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "22.2");
  assert_eq!(v.to_string(), "22.2");
}

#[test]
fn display_matches_as_str() {
  assert_eq!(ChannelLayout::Stereo.to_string(), "stereo");
  assert_eq!(ChannelLayout::N5Point1.to_string(), "5.1(side)");
  assert_eq!(
    ChannelLayout::Other(SmolStr::new("custom_layout")).to_string(),
    "custom_layout"
  );
}

#[test]
fn is_variant_predicates() {
  assert!(ChannelLayout::Mono.is_mono());
  assert!(ChannelLayout::Stereo.is_stereo());
  assert!(ChannelLayout::N5Point1.is_n_5_point_1());
  assert!(ChannelLayout::Other(SmolStr::new("x")).is_other());
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with the escape folding too, so one name is one
/// value under the derived `Eq` / `Hash`.
#[test]
fn channellayout_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["mono", "stereo", "5.1", "7.1", "quad"];
  for (i, slug) in SLUGS.iter().enumerate() {
    assert!(
      !slug.bytes().any(|b| b.is_ascii_uppercase()),
      "slug {slug:?} is not lowercase-canonical"
    );
    for prior in &SLUGS[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two variants fold onto {slug:?}"
      );
    }
    let v: ChannelLayout = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), *slug, "`{slug}` is not its own canonical form");
  }
  assert_eq!("mono", "MONO".parse::<ChannelLayout>().unwrap().as_str());

  // The escape folds on the way in.
  let escaped: ChannelLayout = "MONO_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "mono_x");
  assert_eq!(ChannelLayout::other("MONO_X"), escaped);
}
#[test]
fn channel_layout_unwrap_other_borrowed_view() {
  let v = ChannelLayout::other("22.2");
  assert_eq!(v.unwrap_other_ref().as_str(), "22.2");
  assert!(v.try_unwrap_other_ref().is_ok());
  assert!(ChannelLayout::Stereo.try_unwrap_other_ref().is_err());
}

/// The runtime half of the `ROSTER` contract for `ChannelLayout` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(
    ChannelLayout::ROSTER,
    "ChannelLayout",
    ChannelLayout::as_str,
  );
}
