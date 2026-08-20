use super::*;
use ::std::string::ToString;

#[test]
fn every_named_variant_round_trips() {
  for slug in [
    "mov", "mp4", "mkv", "webm", "avi", "flv", "mpegts", "ogg", "asf", "rm", "wmv", "mxf", "gxf",
    "3gp",
  ] {
    let v: Format = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn unknown_slug_lands_in_other() {
  let v: Format = "weird_container".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_container");
  assert_eq!(v.to_string(), "weird_container");
}

#[test]
fn display_matches_as_str() {
  assert_eq!(Format::Mp4.to_string(), "mp4");
  assert_eq!(Format::MpegTs.to_string(), "mpegts");
  assert_eq!(Format::Threegp.to_string(), "3gp");
  assert_eq!(Format::Other(SmolStr::new("custom")).to_string(), "custom");
}

#[test]
fn is_variant_predicates() {
  // Hand-written `is_mp4` (vs the auto-derived `is_mp_4` that the
  // `IsVariant` derive would otherwise produce) — see the
  // `#[is_variant(ignore)]` attribute on `Format::Mp4`.
  assert!(Format::Mp4.is_mp4());
  assert!(!Format::Mkv.is_mp4());
  assert!(Format::Threegp.is_threegp());
  assert!(Format::Other(SmolStr::new("x")).is_other());
}

#[test]
fn unwrap_other_borrowed_view() {
  // `Other(SmolStr)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants.
  let v = Format::Other(SmolStr::new("custom"));
  assert_eq!(v.unwrap_other_ref().as_str(), "custom");
  assert!(v.try_unwrap_other_ref().is_ok());
  let named = Format::Mp4;
  assert!(named.try_unwrap_other_ref().is_err());
}

#[test]
fn as_extension_matches_disk_form() {
  // Most variants: slug == extension.
  assert_eq!(Format::Mov.as_extension(), "mov");
  assert_eq!(Format::Mp4.as_extension(), "mp4");
  assert_eq!(Format::Mkv.as_extension(), "mkv");
  assert_eq!(Format::Webm.as_extension(), "webm");
  assert_eq!(Format::Avi.as_extension(), "avi");
  assert_eq!(Format::Flv.as_extension(), "flv");
  assert_eq!(Format::Threegp.as_extension(), "3gp");
  // Variants where extension differs from FFmpeg slug.
  assert_eq!(Format::MpegTs.as_str(), "mpegts");
  assert_eq!(Format::MpegTs.as_extension(), "ts");
  assert_eq!(Format::Ogg.as_str(), "ogg");
  assert_eq!(Format::Ogg.as_extension(), "ogv");
  // Other has no known extension.
  assert_eq!(Format::Other(SmolStr::new("weird")).as_extension(), "");
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with the escape folding too, so one name is one
/// value under the derived `Eq` / `Hash`.
#[test]
fn format_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["mp4", "mkv", "webm", "mov", "avi", "mpegts", "flv", "ogg"];
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
    let v: Format = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), *slug, "`{slug}` is not its own canonical form");
  }
  assert_eq!("mp4", "MP4".parse::<Format>().unwrap().as_str());

  // The escape folds on the way in.
  let escaped: Format = "MP4_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "mp4_x");
  assert_eq!(Format::other("MP4_X"), escaped);
}

/// The runtime half of the `ROSTER` contract for `Format` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(Format::ROSTER, "Format", Format::as_str);
}
