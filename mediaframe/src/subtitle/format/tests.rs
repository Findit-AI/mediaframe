use super::*;
use ::std::string::ToString;

/// Every named variant's slug round-trips through `as_str` →
/// `FromStr`. [`Format::HdmvPgs`] shares its slug with
/// [`Format::PgsSub`] so the round-trip canonicalises to
/// `PgsSub`; that pair is verified separately.
const NAMED_SLUGS: &[(&str, Format)] = &[
  ("srt", Format::Srt),
  ("webvtt", Format::WebVtt),
  ("ass", Format::Ass),
  ("ssa", Format::Ssa),
  ("microdvd", Format::Sub),
  ("mpl2", Format::Mpl2),
  ("lrc", Format::Lrc),
  ("sami", Format::Smi),
  ("stl", Format::Stl),
  ("subviewer", Format::Sbv),
  ("ttml", Format::Ttml),
  ("mov_text", Format::MovText),
  ("dvd_subtitle", Format::DvdSub),
  ("hdmv_pgs_subtitle", Format::PgsSub),
  ("dvb_subtitle", Format::DvbSub),
  ("xsub", Format::XSub),
];

#[test]
fn as_str_round_trips_for_every_named_variant() {
  for (slug, variant) in NAMED_SLUGS {
    assert_eq!(variant.as_str(), *slug, "as_str mismatch for {variant:?}");
    let parsed: Format = slug.parse().unwrap();
    assert_eq!(&parsed, variant, "FromStr mismatch for {slug:?}");
  }
}

#[test]
fn hdmv_pgs_slug_canonicalises_to_pgs_sub() {
  // `HdmvPgs` and `PgsSub` share the FFmpeg `"hdmv_pgs_subtitle"`
  // slug. Both render to it; parsing the slug picks the first
  // arm — `PgsSub`. `HdmvPgs` is kept as an alias for callers
  // that prefer the FFmpeg-canonical name.
  assert_eq!(Format::HdmvPgs.as_str(), "hdmv_pgs_subtitle");
  assert_eq!(Format::PgsSub.as_str(), "hdmv_pgs_subtitle");
  let parsed: Format = "hdmv_pgs_subtitle".parse().unwrap();
  assert_eq!(parsed, Format::PgsSub);
}

#[test]
fn from_str_is_total_for_unknown_slug() {
  let parsed: Format = "definitely_not_a_real_subtitle_format_xyz".parse().unwrap();
  assert!(matches!(parsed, Format::Other(_)));
  assert_eq!(parsed.as_str(), "definitely_not_a_real_subtitle_format_xyz");
}

#[test]
fn is_image_based_classifies_known_variants() {
  // Image-based.
  assert_eq!(Format::DvdSub.is_image_based(), Some(true));
  assert_eq!(Format::PgsSub.is_image_based(), Some(true));
  assert_eq!(Format::HdmvPgs.is_image_based(), Some(true));
  assert_eq!(Format::DvbSub.is_image_based(), Some(true));
  assert_eq!(Format::XSub.is_image_based(), Some(true));
  // Text-based.
  assert_eq!(Format::Srt.is_image_based(), Some(false));
  assert_eq!(Format::WebVtt.is_image_based(), Some(false));
  assert_eq!(Format::Ass.is_image_based(), Some(false));
  assert_eq!(Format::MovText.is_image_based(), Some(false));
  // Unknown.
  assert_eq!(Format::Other(SmolStr::new("weird")).is_image_based(), None,);
}

#[test]
fn display_matches_as_str() {
  for (_slug, variant) in NAMED_SLUGS {
    assert_eq!(variant.to_string(), variant.as_str());
  }
  assert_eq!(
    Format::Other(SmolStr::new("custom_fmt")).to_string(),
    "custom_fmt",
  );
}

#[test]
fn is_variant_predicates() {
  assert!(Format::Srt.is_srt());
  assert!(!Format::Srt.is_web_vtt());
  assert!(Format::Other(SmolStr::new("x")).is_other());
}

#[test]
fn as_extension_matches_disk_form() {
  // Text-based formats: extension is the canonical .ext (often differs
  // from the FFmpeg slug, e.g. WebVtt slug "webvtt" vs ext "vtt").
  for (variant, ext) in [
    (Format::Srt, "srt"),
    (Format::WebVtt, "vtt"),
    (Format::Ass, "ass"),
    (Format::Ssa, "ssa"),
    (Format::Sub, "sub"),
    (Format::Mpl2, "mpl"),
    (Format::Lrc, "lrc"),
    (Format::Smi, "smi"),
    (Format::Stl, "stl"),
    (Format::Sbv, "sbv"),
    (Format::Ttml, "ttml"),
  ] {
    assert_eq!(variant.as_extension(), ext, "{variant:?}");
  }
  // Image-based + container-embedded: no standalone extension.
  for variant in [
    Format::MovText,
    Format::DvdSub,
    Format::PgsSub,
    Format::HdmvPgs,
    Format::DvbSub,
    Format::XSub,
  ] {
    assert_eq!(variant.as_extension(), "", "{variant:?}");
  }
  // Other: unknown.
  assert_eq!(Format::Other(SmolStr::new("custom")).as_extension(), "");
}

/// Lowercase-canonical, collision-free once folded, and read
/// case-insensitively — with the escape folding too, so one name is one
/// value under the derived `Eq` / `Hash`.
#[test]
fn format_slugs_are_lowercase_canonical_and_fold() {
  const SLUGS: &[&str] = &["srt", "webvtt", "ass", "ssa", "ttml", "mov_text"];
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
  assert_eq!("srt", "SRT".parse::<Format>().unwrap().as_str());

  // The escape folds on the way in.
  let escaped: Format = "SRT_X".parse().unwrap();
  assert!(escaped.is_other());
  assert_eq!(escaped.as_str(), "srt_x");
  assert_eq!(Format::other("SRT_X"), escaped);
}
