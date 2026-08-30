//! The rows [`subtag_common`](super::subtag_common) writes for all three subtag types at once,
//! asserted once for all three.
//!
//! Every other row the macro emits is about ONE type's grammar and is asserted in that type's own
//! file — the door, the fold, the refusals, the rendering. What is here is the SHAPE the three
//! share, and a shape asserted three times is three places for one claim to drift.

use std::{
  string::{String, ToString},
  vec::Vec,
};

use smol_bytes::Utf8Bytes;

use super::{
  Language, LanguageId, ParseLanguageError, ParseRegionError, ParseScriptSubtagError, Region,
  ScriptSubtag, registry,
};

/// **THE TEXT OUT, ALL THREE WAYS, FOR ALL THREE TYPES** — the borrow, the seat the type holds, and
/// the owned `String` a text boundary crosses on.
///
/// Each is asked of a value that ARRIVED DIRTY, because the claim worth pinning is not that a
/// conversion copies bytes but that it copies the FOLDED bytes: a conversion reading anything but
/// the canonical text would put an mkv's `GER` back on a boundary the rest of this house has
/// already agreed is `de`.
#[test]
fn the_text_out_is_the_canonical_text_for_all_three() {
  let language = Language::new("GER").expect("a language subtag");
  assert_eq!(language.as_ref() as &str, "de");
  assert_eq!(String::from(language), "de");
  assert_eq!(Utf8Bytes::from(language), Utf8Bytes::from("de"));

  let script = ScriptSubtag::new("hans").expect("a script subtag");
  assert_eq!(script.as_ref() as &str, "Hans");
  assert_eq!(String::from(script), "Hans");
  assert_eq!(Utf8Bytes::from(script), Utf8Bytes::from("Hans"));

  let region = Region::new("bu").expect("a region subtag");
  assert_eq!(region.as_ref() as &str, "MM");
  assert_eq!(String::from(region), "MM");
  assert_eq!(Utf8Bytes::from(region), Utf8Bytes::from("MM"));
}

/// **THE THREE BOUNDED SEATS ARE `Copy`**, which is the whole content of the inline repr.
///
/// A compile-time witness rather than a runtime assertion, and it stands where the retired triple's
/// own `Copy` check stood: if a seat ever grows a heap-backed field, the build fails here rather
/// than at a distant call site that had been passing it by value.
const _: () = {
  const fn is_copy<T: Copy>() {}

  is_copy::<Language>();
  is_copy::<ScriptSubtag>();
  is_copy::<Region>();
};

/// **THE DERIVED ORDER IS THE TEXT'S ORDER**, walked over the WHOLE registry rather than sampled.
///
/// The one claim the inline repr rests on that is not obvious. [`Ord`] is derived over a fixed byte
/// buffer, so it compares the PADDING too — and the padding is what makes it agree with `str`:
/// every byte a subtag can hold is an ASCII letter or digit (`0x30` at the lowest), so a shorter
/// subtag's first unused byte (`0x00`) sorts below anything a longer one could have there, which is
/// exactly what `str` does when one operand runs out.
///
/// The comparison is against each value's OWN canonical text rather than the registry spelling it
/// was built from, because the door folds: `aam` is held as the subtag the registry prefers, and
/// what this asserts is that the seat and its own rendering order alike.
///
/// A sample would not hold this. The pairs that could break it are the ones where one spelling is a
/// PREFIX of the other, and they only turn up if you walk the table — so this compares every
/// registered subtag against every other of the same kind that shares its first byte, which is
/// where every prefix pair lives.
#[test]
fn the_derived_order_is_the_texts_order_across_the_registry() {
  fn agrees<T: Ord + Copy>(held: &[T], text: impl Fn(&T) -> &str) {
    for (at, left) in held.iter().enumerate() {
      for right in &held[at..] {
        assert_eq!(
          left.cmp(right),
          text(left).cmp(text(right)),
          "`{}` vs `{}`: the derived order and the text's part",
          text(left),
          text(right),
        );
      }
    }
  }

  // Grouped by first byte: two subtags that differ in it are decided by that byte on both
  // readings, so the pairs worth comparing are the ones sharing it — which is also what keeps this
  // out of 8275².
  let mut languages: std::collections::BTreeMap<u8, Vec<Language>> =
    std::collections::BTreeMap::new();
  for (subtag, _) in registry::table::LANGUAGES {
    let held = Language::new(subtag.as_str()).expect("a registered subtag is a subtag");
    languages
      .entry(held.as_str().as_bytes()[0])
      .or_default()
      .push(held);
  }
  for group in languages.values() {
    agrees(group, |held| held.as_str());
  }

  let scripts: Vec<ScriptSubtag> = registry::table::SCRIPTS
    .iter()
    .map(|(subtag, _)| ScriptSubtag::new(subtag.as_str()).expect("a script"))
    .collect();
  agrees(&scripts, |held| held.as_str());

  // Both region grammars in one walk, which is the pair that matters here: `419` and `DE` are three
  // digits and two letters, and digits sort below letters on both readings.
  let regions: Vec<Region> = registry::table::REGIONS
    .iter()
    .map(|(subtag, _)| Region::new(subtag.as_str()).expect("a region"))
    .collect();
  agrees(&regions, |held| held.as_str());
}

/// **A PREFIX PAIR IS THE CASE THE PADDING EXISTS FOR**, named rather than left to the walk.
///
/// The pair is deliberately UNREGISTERED. Every registered prefix pair the table holds is one the
/// door folds away — `eng` is ISO 639-2 for English and reaches `en`, so the two are one value and
/// prove nothing about ordering — and what is under test here is the seat, not the registry.
#[test]
fn a_shorter_subtag_sorts_before_the_longer_one_it_prefixes() {
  let short = Language::new("xy").expect("structurally a language subtag");
  let long = Language::new("xyz").expect("structurally a language subtag");

  assert_eq!(short.as_str(), "xy", "unregistered, so nothing folded it");
  assert_eq!(long.as_str(), "xyz");
  assert!(short < long, "the padding must sort below any real byte");
  assert_eq!(short.cmp(&long), "xy".cmp("xyz"));

  // …and the same on the region's digit arm, where the bytes are lower still.
  assert!(
    Region::new("419").expect("a region") < Region::new("AA").expect("a region"),
    "digits sort before letters, on both readings",
  );
}

/// **THE TEXT IN, BOTH STANDARD SPELLINGS, FOR ALL THREE TYPES** — `TryFrom<&str>` and
/// `TryFrom<Utf8Bytes>`, each forwarding to the type's own `new`.
///
/// They are the door and not a second grammar, which is the whole claim: a dirty spelling folds
/// through them exactly as it folds through `new`, and a refusal is the same refusal. The
/// `Utf8Bytes` row is the inverse of the `From` row above, fallible where that one is not.
#[test]
fn the_text_in_is_the_door_for_all_three() {
  assert_eq!(
    Language::try_from("GER").expect("a language subtag"),
    Language::new("de").expect("a language subtag"),
  );
  assert_eq!(
    Language::try_from(Utf8Bytes::from("iw")).expect("a language subtag"),
    Language::new("he").expect("a language subtag"),
  );
  assert_eq!(
    ScriptSubtag::try_from("hans").expect("a script subtag"),
    ScriptSubtag::new("Hans").expect("a script subtag"),
  );
  assert_eq!(
    ScriptSubtag::try_from(Utf8Bytes::from("LATN")).expect("a script subtag"),
    ScriptSubtag::new("Latn").expect("a script subtag"),
  );
  assert_eq!(
    Region::try_from("bu").expect("a region subtag"),
    Region::new("MM").expect("a region subtag"),
  );
  assert_eq!(
    Region::try_from(Utf8Bytes::from("419")).expect("a region subtag"),
    Region::new("419").expect("a region subtag"),
  );

  // The refusals are the doors' own, carried whole.
  assert_eq!(Language::try_from("e"), Err(ParseLanguageError::TooShort));
  assert_eq!(
    ScriptSubtag::try_from(Utf8Bytes::from("Lat")),
    Err(ParseScriptSubtagError::WrongWidth)
  );
  assert_eq!(
    Region::try_from("DEU"),
    Err(ParseRegionError::WrongLetterWidth)
  );
}

/// **`und` IS THE DEFAULT IDENTITY, AND THE DEFAULT IS NOT AN ABSENCE.**
///
/// The row [`Default`] adds to the family beyond what arrived, and the one most at risk of being
/// misread: a track nobody tagged is an `Option::None`, and this is the value a muxer writes when it
/// looked and could not tell. The wire codec needs it to seed a value before reading one, which is
/// the whole of why it exists.
#[test]
fn the_default_identity_is_the_undetermined_tag() {
  let default = LanguageId::default();

  assert_eq!(default.to_string(), "und");
  assert!(default.language().is_undetermined());
  assert_eq!(default.script(), None);
  assert_eq!(default.region(), None);
  assert_eq!(default.rest(), None);

  assert_eq!(LanguageId::new("und").expect("a tag"), default);
  assert_eq!(LanguageId::from(Language::UND), default);

  let absent: Option<LanguageId> = None;
  assert_ne!(absent, Some(default), "no tag is not the `und` tag");
}
