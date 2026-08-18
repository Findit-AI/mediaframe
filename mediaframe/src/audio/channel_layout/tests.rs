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
  assert_eq!(ChannelLayout::N5Point1.to_string(), "5.1");
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
