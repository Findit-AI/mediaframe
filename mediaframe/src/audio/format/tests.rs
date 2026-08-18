use super::*;
use ::std::string::ToString;

#[test]
fn audio_format_u32_round_trips_named_variants() {
  for v in [
    SampleFormat::U8,
    SampleFormat::S16,
    SampleFormat::S32,
    SampleFormat::Flt,
    SampleFormat::Dbl,
    SampleFormat::U8p,
    SampleFormat::S16p,
    SampleFormat::S32p,
    SampleFormat::Fltp,
    SampleFormat::Dblp,
    SampleFormat::S64,
    SampleFormat::S64p,
  ] {
    let back = SampleFormat::from_u32(v.to_u32().expect("named format has a code"));
    assert_eq!(
      back,
      Some(v.clone()),
      "round-trip mismatch for `{}`",
      v.as_str()
    );
  }
}

#[test]
fn audio_format_unnamed_code_is_rejected_and_the_escape_keeps_its_name() {
  assert_eq!(SampleFormat::from_u32(12_345), None);
  let vendor = SampleFormat::other("VENDOR_S24");
  assert_eq!(vendor.as_str(), "vendor_s24");
  assert_eq!(vendor.to_u32(), None);
}

#[test]
fn audio_format_from_str_named() {
  for slug in [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ] {
    let v: SampleFormat = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn audio_format_unknown_slug_lands_in_other() {
  let v: SampleFormat = "weird_sample_fmt".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_sample_fmt");
}

#[test]
fn audio_format_is_planar_predicate() {
  assert!(SampleFormat::U8p.is_planar());
  assert!(SampleFormat::S16p.is_planar());
  assert!(SampleFormat::Fltp.is_planar());
  assert!(!SampleFormat::U8.is_planar());
  assert!(!SampleFormat::Flt.is_planar());
}

#[test]
fn audio_format_display_matches_as_str() {
  assert_eq!(SampleFormat::Flt.to_string(), "flt");
  assert_eq!(SampleFormat::Fltp.to_string(), "fltp");
}

#[test]
fn audio_container_round_trips_named_variants() {
  for slug in [
    "mp3", "aac", "flac", "ogg", "opus", "wav", "aiff", "alac", "wma", "ape", "wv", "mka", "m4a",
    "caf",
  ] {
    let v: ContainerFormat = slug.parse().unwrap();
    assert!(!v.is_other(), "`{slug}` should be a named variant");
    assert_eq!(v.as_str(), slug);
  }
}

#[test]
fn audio_container_unknown_lands_in_other() {
  let v: ContainerFormat = "weird_audio_container".parse().unwrap();
  assert!(v.is_other());
  assert_eq!(v.as_str(), "weird_audio_container");
}

#[test]
fn audio_container_display_matches_as_str() {
  assert_eq!(ContainerFormat::Mp3.to_string(), "mp3");
  assert_eq!(ContainerFormat::Flac.to_string(), "flac");
  assert_eq!(
    ContainerFormat::Other(SmolStr::new("snd")).to_string(),
    "snd"
  );
}

#[test]
fn audio_container_unwrap_other_borrowed_view() {
  // `Other(SmolStr)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants.
  let v = ContainerFormat::Other(SmolStr::new("custom_audio"));
  assert_eq!(v.unwrap_other_ref().as_str(), "custom_audio");
  assert!(v.try_unwrap_other_ref().is_ok());
  let named = ContainerFormat::Flac;
  assert!(named.try_unwrap_other_ref().is_err());
}

#[test]
fn audio_container_as_extension_matches_disk_form() {
  // Most variants: slug == extension.
  for (variant, ext) in [
    (ContainerFormat::Mp3, "mp3"),
    (ContainerFormat::Aac, "aac"),
    (ContainerFormat::Flac, "flac"),
    (ContainerFormat::Ogg, "ogg"),
    (ContainerFormat::Opus, "opus"),
    (ContainerFormat::Wav, "wav"),
    (ContainerFormat::Aiff, "aiff"),
    (ContainerFormat::Wma, "wma"),
    (ContainerFormat::Ape, "ape"),
    (ContainerFormat::Wv, "wv"),
    (ContainerFormat::Mka, "mka"),
    (ContainerFormat::M4a, "m4a"),
    (ContainerFormat::Caf, "caf"),
  ] {
    assert_eq!(variant.as_extension(), ext);
  }
  // ALAC has no standalone extension — rides in `.m4a`.
  assert_eq!(ContainerFormat::Alac.as_str(), "alac");
  assert_eq!(ContainerFormat::Alac.as_extension(), "m4a");
  // Other has no known extension.
  assert_eq!(
    ContainerFormat::Other(SmolStr::new("weird")).as_extension(),
    ""
  );
}
/// Both audio vocabularies are lowercase-canonical, collision-free once
/// folded, and read case-insensitively — escape included.
#[test]
fn audio_slugs_are_lowercase_canonical_and_fold() {
  const SAMPLE: &[&str] = &[
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p",
  ];
  const CONTAINER: &[&str] = &[
    "mp3", "aac", "flac", "ogg", "opus", "wav", "aiff", "alac", "wma", "ape", "wv", "mka", "m4a",
    "caf",
  ];

  for (i, slug) in SAMPLE.iter().enumerate() {
    assert!(!slug.bytes().any(|b| b.is_ascii_uppercase()));
    for prior in &SAMPLE[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two formats fold onto {slug:?}"
      );
    }
    let v: SampleFormat = slug.parse().unwrap();
    assert!(!v.is_other());
    assert_eq!(v.as_str(), *slug);
  }
  for (i, slug) in CONTAINER.iter().enumerate() {
    assert!(!slug.bytes().any(|b| b.is_ascii_uppercase()));
    for prior in &CONTAINER[..i] {
      assert!(
        !prior.eq_ignore_ascii_case(slug),
        "two containers fold onto {slug:?}"
      );
    }
    let v: ContainerFormat = slug.parse().unwrap();
    assert!(!v.is_other());
    assert_eq!(v.as_str(), *slug);
  }

  assert_eq!("S16".parse(), Ok(SampleFormat::S16));
  assert_eq!("FLTP".parse(), Ok(SampleFormat::Fltp));
  assert_eq!("FLAC".parse(), Ok(ContainerFormat::Flac));
  assert_eq!(SampleFormat::other("VENDOR_S24").as_str(), "vendor_s24");
  assert_eq!(ContainerFormat::other("SND").as_str(), "snd");
}
#[test]
fn sample_format_unwrap_other_borrowed_view() {
  // `Other(SmolStr)` carries data — golden-rule §2 mandates
  // unwrap/try_unwrap accessors for data-carrying variants, and this
  // type's 12 variants are far under the compile-time threshold that
  // exempts the 200-plus codec enums.
  let v = SampleFormat::other("vendor_s24");
  assert_eq!(v.unwrap_other_ref().as_str(), "vendor_s24");
  assert!(v.try_unwrap_other_ref().is_ok());
  assert!(SampleFormat::S16.try_unwrap_other_ref().is_err());
}
