//! The composition rules, row by row, and the four seats they fill.
//!
//! Each `#[test]` below is one clause of the ruling this type implements: the lossless tail, the
//! composition rules (`_`, `Suppress-Script`, grandfathered), the positional seats, and the
//! equality and ordering the four of them give.

use core::str::FromStr;
use std::{
  collections::BTreeSet,
  string::{String, ToString},
  vec::Vec,
};

use smol_bytes::Utf8Bytes;

use super::{LanguageId, ParseLanguageIdError};
use crate::lang::{Language, ParseLanguageError, ParseRegionError, Region, ScriptSubtag, registry};

/// One tag through the door, or the reason it was refused.
fn door(sent: &str) -> Result<LanguageId, ParseLanguageIdError> {
  LanguageId::new(sent)
}

/// One tag the door must take, unwrapped.
fn admitted(sent: &str) -> LanguageId {
  door(sent).unwrap_or_else(|refused| panic!("`{sent}` is a language tag: {refused}"))
}

/// The canonical rendering of one tag, which is what most rows below compare.
fn canonical(sent: &str) -> String {
  admitted(sent).to_string()
}

/// The corpus the two sweeps at the end of this file walk: the registry's own columns, every shape
/// the composition rules decide between, and the tags hand-picked for being adversarial about them.
///
/// The REGISTRY rather than a sample, for the reason the ordering walk in the household's own file
/// gives: what breaks a fold is a row where the fold MOVES a subtag, and those only turn up if you
/// walk the column that folds. The rows a column cannot produce — a tail in the shape of a seat, a
/// spelling arriving dirty — are added by hand after it.
///
/// A few rows are tags the door refuses (`i-default` and its two siblings), and the sweeps skip
/// them: the claim under test is about what is ADMITTED, and a refusal is the door already having
/// said no.
fn corpus() -> Vec<String> {
  let mut tags: Vec<String> = Vec::new();

  // Every registered language on its own — which is the `Preferred-Value` and alpha-3 folds, the
  // case fold, and `und`, over every row that has one.
  for (subtag, _) in registry::table::LANGUAGES {
    tags.push(String::from(subtag.as_str()));
  }

  // Every language the registry gives a `Suppress-Script`, in the shapes the fold has to decide
  // between: the bare pair, the pair with each later seat filled, and the tails that would climb
  // into the slot the fold vacates.
  for (subtag, script) in registry::table::LANGUAGE_SUPPRESS_SCRIPT {
    let subtag = subtag.as_str();
    tags.push(std::format!("{subtag}-{script}"));
    tags.push(std::format!("{subtag}-{script}-US"));
    tags.push(std::format!("{subtag}-{script}-419"));
    tags.push(std::format!("{subtag}-{script}-Cyrl"));
    tags.push(std::format!("{subtag}-{script}-{script}"));
    tags.push(std::format!("{subtag}-{script}-US-Cyrl"));
    tags.push(std::format!("{subtag}-{script}-1901"));
    tags.push(std::format!("{subtag}-{script}-x-a"));
    // The case fold is a canonicalisation too, so the same pair arrives dirty.
    tags.push(std::format!(
      "{}-{}",
      subtag.to_uppercase(),
      script.to_lowercase()
    ));
  }

  // Every registered script and region, beside a language that implies neither — so the seats are
  // filled from the registry rather than from a handful of familiar subtags.
  for (subtag, _) in registry::table::SCRIPTS {
    let subtag = subtag.as_str();
    tags.push(std::format!("zh-{subtag}"));
  }
  for (subtag, _) in registry::table::REGIONS {
    let subtag = subtag.as_str();
    tags.push(std::format!("zh-Hans-{subtag}"));
  }

  // Every grandfathered tag, replaced or kept.
  for (tag, _) in registry::table::GRANDFATHERED {
    tags.push(String::from(tag.as_str()));
  }
  for tag in registry::table::GRANDFATHERED_KEPT {
    tags.push(String::from(*tag));
  }

  tags.extend(interactions());

  for tag in [
    // The two the suppression's guard exists for, and the pair it must not stop folding.
    "en-Latn-Cyrl",
    "en-Latn-Latn",
    "en-Latn",
    "en-Latn-US",
    "en-Latn-US-Cyrl",
    "en-Latn-x-Cyrl",
    // The rest of the composition table, each row arriving as dirty as it can.
    "zh_Hans_CN",
    "GER-latn-de",
    "deu-Latn-DE",
    "EN-us",
    "iw-IL",
    "de-BU",
    "und",
    // The tail in each of its three grammars, and in the case the fold must not touch.
    "en-US-posix",
    "de-u-co-phonebk",
    "en-x-lorem",
    "en-US-x-Foo",
    "en-US-x-foo",
    "sl-rozaj-biske",
    "hy-Latn-IT-arevela",
    "en-a-bbb-x-a-ccc",
    "de-CH-1901",
    // The subtags no seat wanted, and the extlang that takes the later seats with it.
    "en-USA",
    "de-1a1",
    "zh-Hans-XYZ",
    "zh-yue",
    "zh-yue-Hant-HK",
    "xx-yy-zz-bogus",
    // Four letters offered to each seat in turn, the width that is a script's alone.
    "abcd",
    "abcd-Latn",
    "en-abcd",
    "en-Latn-abcd",
    "en-Latn-ABCD",
  ] {
    tags.push(String::from(tag));
  }

  tags
}

/// **THE INTERACTION PREIMAGES** — the half of the corpus a walk over the tables cannot reach.
///
/// The corpus above walks each column's rows INDEPENDENTLY, and that blindness is exactly what let
/// `en-Latn-GB-oed` through: no row of any table is that tag, and every fold in it is an ordinary
/// one. What breaks a fixed point is never a row — it is a fold whose OUTPUT lands in another
/// fold's PREIMAGE, and the tag where the two meet is a composition no single column holds.
///
/// So this builds those meetings from the tables rather than naming them, which is what keeps them
/// growing with the registry instead of with a reviewer's imagination:
///
/// ```text
///   alpha-3, `Preferred-Value`  ──►  a grandfathered tag's LANGUAGE subtag
///   `Suppress-Script`           ──►  a grandfathered tag's SCRIPT slot, which the fold vacates
///   region `Preferred-Value`    ──►  a grandfathered tag's REGION slot
///   alpha-3, `Preferred-Value`  ──►  a language whose `Suppress-Script` then fires
///   region `Preferred-Value`    ──►  the seat after the one the suppression vacates
/// ```
fn interactions() -> Vec<String> {
  let mut tags: Vec<String> = Vec::new();

  // EVERY WAY IN to a grandfathered tag. Its language subtag is spelled every way another fold can
  // produce it, the script that language implies is offered at the slot the suppression will take
  // it back out of, and a region slot is spelled with every deprecated region that folds onto it —
  // all three at once, since the interaction is the point.
  for tag in registry::table::GRANDFATHERED
    .iter()
    .map(|(tag, _)| tag.as_str())
    .chain(registry::table::GRANDFATHERED_KEPT.iter().copied())
  {
    let (language, after) = tag.split_once('-').unwrap_or((tag, ""));
    let scripts: Vec<Option<&str>> = match registry::language_suppress_script(language) {
      Some(script) => std::vec![None, Some(script)],
      None => std::vec![None],
    };

    for spelling in language_preimages(language) {
      for script in scripts.iter().copied() {
        for after in region_preimages(after) {
          let mut built = spelling.clone();
          if let Some(script) = script {
            built.push('-');
            built.push_str(script);
          }
          if !after.is_empty() {
            built.push('-');
            built.push_str(&after);
          }
          tags.push(built.to_uppercase());
          tags.push(built);
        }
      }
    }
  }

  // THE SEAT FOLDS MEETING THE SUPPRESSION. A language reached through alpha-3 or a
  // `Preferred-Value` implies whatever its FOLDED spelling implies, so the suppression fires on a
  // subtag the sender never wrote — and the four shapes are the ones its guard decides between: the
  // bare pair, the pair with the seat after it filled, and the two tails that would climb into the
  // slot the fold vacates.
  let reached = registry::table::ALPHA3
    .iter()
    .map(|(code, shortest)| {
      (
        code.as_str(),
        registry::language_preferred(shortest).unwrap_or(shortest),
      )
    })
    .chain(
      registry::table::LANGUAGE_PREFERRED
        .iter()
        .map(|(subtag, preferred)| (subtag.as_str(), *preferred)),
    );

  for (sent, folded) in reached {
    let Some(script) = registry::language_suppress_script(folded) else {
      continue;
    };
    tags.push(std::format!("{sent}-{script}"));
    tags.push(std::format!("{sent}-{script}-US"));
    tags.push(std::format!("{sent}-{script}-Cyrl"));
    tags.push(std::format!("{sent}-{script}-{script}"));
  }

  // THE REGION FOLD BESIDE THE SUPPRESSION — a deprecated region in the seat that follows the one
  // the suppression vacates, on every language that implies a script.
  for (deprecated, _) in registry::table::REGION_PREFERRED {
    let deprecated = deprecated.as_str();
    for (subtag, script) in registry::table::LANGUAGE_SUPPRESS_SCRIPT {
      let subtag = subtag.as_str();
      tags.push(std::format!("{subtag}-{script}-{deprecated}"));
    }
  }

  tags
}

/// Every spelling a language subtag can ARRIVE as and still reach `subtag` — the preimage of the
/// two folds `Language::new` applies, in the sequence it applies them.
fn language_preimages(subtag: &str) -> Vec<String> {
  let mut spellings = std::vec![String::from(subtag)];

  spellings.extend(
    registry::table::LANGUAGE_PREFERRED
      .iter()
      .filter(|(_, preferred)| *preferred == subtag)
      .map(|(deprecated, _)| String::from(deprecated.as_str())),
  );

  // The alpha-3 fold runs FIRST, so a code reaches `subtag` either directly or through a spelling
  // the `Preferred-Value` fold then rewrites — which is the sequence the two are applied in, and
  // the reason they are applied in sequence rather than as alternatives.
  let through: Vec<String> = spellings.clone();
  spellings.extend(
    registry::table::ALPHA3
      .iter()
      .filter(|(_, shortest)| through.iter().any(|hop| hop == shortest))
      .map(|(code, _)| String::from(code.as_str())),
  );

  spellings
}

/// The tag's subtags past its language, with its region slot spelled every way the region fold can
/// produce it.
fn region_preimages(after: &str) -> Vec<String> {
  let mut spellings = std::vec![String::from(after)];

  let (head, rest) = after.split_once('-').unwrap_or((after, ""));
  if !super::region_shaped(head) {
    return spellings;
  }

  let canonical = head.to_uppercase();
  for (deprecated, preferred) in registry::table::REGION_PREFERRED {
    let deprecated = deprecated.as_str();
    if *preferred != canonical {
      continue;
    }
    spellings.push(match rest.is_empty() {
      true => String::from(deprecated),
      false => std::format!("{deprecated}-{rest}"),
    });
  }

  spellings
}

/// **THE COMPOSITION TABLE**, transcribed from the ruling.
#[test]
fn the_door_applies_every_composition_rule_the_ruling_names() {
  for (sent, held, why) in [
    // `_` is read as `-`, which is what a filename-safe tag uses.
    ("zh_Hans_CN", "zh-Hans-CN", "underscores"),
    ("en_US", "en-US", "underscores"),
    // The SUPPRESSION: a declared script the language already implies carries nothing.
    ("en-Latn", "en", "`en` implies `Latn`"),
    ("en-Latn-US", "en-US", "and the region survives it"),
    ("fr-latn", "fr", "any case"),
    ("he-Hebr", "he", "not only Latin"),
    // …and a language that implies NO script keeps the one it was given.
    ("zh-Hans", "zh-Hans", "`zh` implies nothing"),
    ("zh-Hant", "zh-Hant", "and the two stay apart"),
    ("zh-Hant-TW", "zh-Hant-TW", "with a region beside it"),
    // GRANDFATHERED tags fold by their `Preferred-Value`, before the tag is taken apart.
    ("i-klingon", "tlh", "grandfathered"),
    ("I-KLINGON", "tlh", "grandfathered, any case"),
    ("zh-guoyu", "cmn", "grandfathered"),
    ("art-lojban", "jbo", "grandfathered"),
    ("no-bok", "nb", "grandfathered"),
    (
      "en-GB-oed",
      "en-GB-oxendict",
      "a multi-subtag preferred value",
    ),
    // Each SEAT's own door, applied where it sits.
    // Three folds in one tag: `ger` is an mkv's German, `latn` is the script German already
    // implies, and the trailing `de` is the REGION Germany — which is why the answer keeps a seat
    // the first two lost.
    (
      "GER-latn-de",
      "de-DE",
      "alpha-3, suppression and region case, at once",
    ),
    ("deu-DE", "de-DE", "an mp4's language subtag"),
    ("iw-IL", "he-IL", "a deprecated language"),
    ("de-BU", "de-MM", "a deprecated region"),
    ("EN-us", "en-US", "case on two seats"),
  ] {
    assert_eq!(canonical(sent), held, "`{sent}` — {why}");
  }
}

/// **THE SEATS ARE POSITIONAL**, and each is filled only by a subtag of its own shape.
#[test]
fn each_seat_is_filled_by_the_shape_that_belongs_in_it() {
  let full = admitted("sr-Cyrl-RS");
  assert_eq!(full.language().as_str(), "sr");
  assert_eq!(
    full.script().as_ref().map(ScriptSubtag::as_str),
    Some("Cyrl")
  );
  assert_eq!(full.region().as_ref().map(Region::as_str), Some("RS"));
  assert_eq!(full.rest(), None);

  let bare = admitted("de");
  assert_eq!(bare.language().as_str(), "de");
  assert_eq!(bare.script(), None);
  assert_eq!(bare.region(), None);

  // A script is FOUR letters and a region is two letters or three DIGITS, so a two-letter subtag
  // after the language is a region even though a script slot is still open.
  let regional = admitted("de-AT");
  assert_eq!(regional.script(), None);
  assert_eq!(regional.region().as_ref().map(Region::as_str), Some("AT"));

  // …and an M.49 area code reaches the same seat.
  let area = admitted("es-419");
  assert_eq!(area.region().as_ref().map(Region::as_str), Some("419"));
  assert!(area.region().expect("a region").is_area());
}

/// **THE TAIL IS LOSSLESS AND VERBATIM** — the ruling's first clause.
///
/// Variants, extensions and the private-use sequence are held exactly as they arrived, rendered
/// back exactly, and counted in equality.
#[test]
fn the_tail_is_held_verbatim_and_rendered_back() {
  for tag in [
    "en-US-posix",
    "de-u-co-phonebk",
    "en-x-lorem",
    "sl-rozaj-biske",
    "en-US-x-Foo",
    "hy-Latn-IT-arevela",
  ] {
    assert_eq!(canonical(tag), tag, "`{tag}` did not round-trip");
  }

  let held = admitted("en-US-x-Foo");
  assert_eq!(held.language().as_str(), "en");
  assert_eq!(held.region().as_ref().map(Region::as_str), Some("US"));
  assert_eq!(held.rest().map(Utf8Bytes::as_str), Some("x-Foo"));
}

/// **THE TAIL'S CASE IS PART OF IT**, which is the price of the seat being lossless.
///
/// RFC 5646's own canonical form lower-cases a variant and an extension; this seat does not, because
/// a private-use sequence's meaning belongs to the private party that wrote it and this type cannot
/// know its case is insignificant. Two spellings of one tail are two values, and the rest of the tag
/// is unaffected.
#[test]
fn two_spellings_of_one_tail_are_two_values() {
  let upper = admitted("en-US-x-Foo");
  let lower = admitted("en-US-x-foo");

  assert_ne!(upper, lower);
  assert_eq!(upper.language(), lower.language());
  assert_eq!(upper.region(), lower.region());

  // The seats BEFORE the tail are folded, so the difference is the tail's alone.
  assert_eq!(admitted("EN-us-x-Foo"), upper);
}

/// **AN EXTLANG HAS NO SEAT, so it rides the tail — and takes the rest of the tag with it.**
///
/// The consequence the type docs state rather than leave to be discovered: the tail is a TAIL, so
/// once a subtag falls into it every later one does too. The tag round-trips exactly and the
/// language is still readable off its seat, which is the question the ruling names; reading the
/// script of such a tag off a seat is what is lost.
#[test]
fn an_extlang_rides_the_tail_and_takes_the_later_seats_with_it() {
  let cantonese = admitted("zh-yue");
  assert_eq!(cantonese.language().as_str(), "zh");
  assert_eq!(cantonese.rest().map(Utf8Bytes::as_str), Some("yue"));
  assert_eq!(cantonese.to_string(), "zh-yue");

  let swallowed = admitted("zh-yue-Hant-HK");
  assert_eq!(swallowed.language().as_str(), "zh");
  assert_eq!(swallowed.script(), None, "the script is IN the tail");
  assert_eq!(swallowed.region(), None, "and so is the region");
  assert_eq!(swallowed.rest().map(Utf8Bytes::as_str), Some("yue-Hant-HK"));
  assert_eq!(swallowed.to_string(), "zh-yue-Hant-HK", "lossless anyway");

  // The well-formed spelling of the same content has both seats filled and both readable.
  let plain = admitted("zh-Hant-HK");
  assert_eq!(
    plain.script().as_ref().map(ScriptSubtag::as_str),
    Some("Hant")
  );
  assert_eq!(plain.region().as_ref().map(Region::as_str), Some("HK"));
}

/// **THE TAIL'S ENVELOPE IS DELIBERATELY LOOSE** — one to eight ASCII alphanumeric characters per
/// subtag, which is the outer shape of every variant, extension and private-use subtag together.
///
/// `en-USA` is admitted though `USA` is not a legal variant, for the reason an unregistered language
/// subtag is: a container writes what its muxer believed.
#[test]
fn the_tail_admits_what_is_alphanumeric_and_refuses_what_is_not() {
  assert_eq!(
    canonical("en-USA"),
    "en-USA",
    "not a legal variant, and carried"
  );
  assert_eq!(canonical("de-1901"), "de-1901");
  assert_eq!(canonical("en-a-bbb-x-a-ccc"), "en-a-bbb-x-a-ccc");

  assert_eq!(door("en-US-!!"), Err(ParseLanguageIdError::Tail('!')));
  assert_eq!(door("en-US-po six"), Err(ParseLanguageIdError::Tail(' ')));
  assert_eq!(
    door("en-US-abcdefghi"),
    Err(ParseLanguageIdError::TailWidth)
  );
  assert!(door("en-US-abcdefgh").is_ok(), "eight is the ceiling");
}

/// **THE FIVE GRANDFATHERED TAGS WITH NO REPLACEMENT** fall through to the ordinary parse, and two
/// of them turn out to be ordinary compositions.
///
/// The honest handling of a table row that names nothing to fold onto: `cel-gaulish` is a registered
/// collection language with a variant-shaped tail, `zh-min` is Chinese with a tail, and the three
/// beginning `i-` are refused because a one-letter primary subtag is outside the grammar — which is
/// a sentence the language door can state.
#[test]
fn a_grandfathered_tag_with_no_replacement_falls_through_to_the_ordinary_parse() {
  assert_eq!(canonical("cel-gaulish"), "cel-gaulish");
  assert_eq!(admitted("cel-gaulish").language().as_str(), "cel");
  assert_eq!(
    admitted("cel-gaulish").rest().map(Utf8Bytes::as_str),
    Some("gaulish")
  );

  assert_eq!(canonical("zh-min"), "zh-min");
  assert_eq!(
    admitted("zh-min").rest().map(Utf8Bytes::as_str),
    Some("min")
  );

  for refused in ["i-default", "i-mingo", "i-enochian"] {
    assert_eq!(
      door(refused),
      Err(ParseLanguageIdError::Language(ParseLanguageError::TooShort)),
      "`{refused}`"
    );
  }
}

/// **THE GRANDFATHERED LOOKUP'S STACK BUFFER FITS EVERY TAG THE TABLE HOLDS.**
///
/// The lookup lower-cases into a fixed buffer and skips a tag too wide for it, which is what keeps
/// the fold from allocating for every ordinary tag that comes through the door. That shortcut is
/// only sound while no grandfathered tag is wider than the buffer, and this is what says so — a
/// registry that added one would fail here rather than silently stop folding it.
#[test]
fn the_grandfathered_table_fits_the_lookup_buffer() {
  let widest = registry::table::GRANDFATHERED
    .iter()
    .map(|(tag, _)| tag.as_str().len())
    .chain(
      registry::table::GRANDFATHERED_KEPT
        .iter()
        .map(|tag| tag.len()),
    )
    .max()
    .expect("twenty-six of them");

  assert!(
    widest <= super::GRANDFATHERED_MAX,
    "the widest grandfathered tag is {widest} bytes and the buffer is {}",
    super::GRANDFATHERED_MAX
  );
}

/// **A STRUCTURAL VIOLATION IS REFUSED, and the refusal carries the SEAT's own words.**
#[test]
fn a_refusal_names_the_seat_and_carries_its_error() {
  assert_eq!(door(""), Err(ParseLanguageIdError::Empty));
  assert_eq!(door("en-"), Err(ParseLanguageIdError::EmptySubtag));
  assert_eq!(door("en--US"), Err(ParseLanguageIdError::EmptySubtag));
  assert_eq!(door("-en"), Err(ParseLanguageIdError::EmptySubtag));

  assert_eq!(
    door("e-US"),
    Err(ParseLanguageIdError::Language(ParseLanguageError::TooShort))
  );
  assert_eq!(
    door("日本語-JP"),
    Err(ParseLanguageIdError::Language(
      ParseLanguageError::NotAlphabetic('日')
    ))
  );

  // A subtag in the region POSITION that is not region-SHAPED is never handed to the region door
  // at all — it falls to the tail, where the looser envelope admits it. So the shape test and the
  // seat's own grammar cannot disagree about one subtag, and a tag like `zh-Hans-XYZ` is carried
  // rather than refused.
  assert_eq!(canonical("zh-Hans-XYZ"), "zh-Hans-XYZ", "the tail took it");
  assert_eq!(
    canonical("de-1a1"),
    "de-1a1",
    "neither two letters nor three digits"
  );
  assert_eq!(
    admitted("de-1a1").rest().map(Utf8Bytes::as_str),
    Some("1a1")
  );
  assert_eq!(admitted("de-1a1").region(), None);
}

/// **EQUALITY IS THE FOUR SEATS**, so two spellings of one identity are one value.
#[test]
fn two_spellings_of_one_identity_are_one_value() {
  let mkv = admitted("GER-latn-de");
  let mp4 = admitted("deu-Latn-DE");
  let bcp47 = admitted("de-DE");

  assert_eq!(mkv, mp4);
  assert_eq!(mp4, bcp47);

  let mut distinct = BTreeSet::new();
  distinct.insert(mkv);
  distinct.insert(mp4);
  distinct.insert(bcp47);
  assert_eq!(distinct.len(), 1, "one identity, one hash bucket");

  // …and a tag with one fewer seat filled is a DIFFERENT identity, which is the other half of the
  // same claim: the fold makes spellings meet, it does not make tags equal.
  assert_ne!(admitted("de"), admitted("de-DE"));
}

/// The rendering and the parse are inverse over canonical text, which is what makes the tag the one
/// text form serde and the wire codec both read and write.
#[test]
fn the_rendering_and_the_parse_are_inverse() {
  for sent in [
    "de",
    "zh-Hans",
    "en-US",
    "sr-Cyrl-RS",
    "en-US-x-Foo",
    "GER-latn-de",
    "i-klingon",
    "zh_Hans_CN",
  ] {
    let held = admitted(sent);
    let rendered = held.to_string();

    assert_eq!(LanguageId::from_str(&rendered).expect("canonical"), held);
    assert_eq!(admitted(&rendered), held, "the door is idempotent");
  }

  assert_eq!(
    std::format!("{:?}", admitted("GER-latn-de")),
    r#"LanguageId("de-DE")"#
  );
}

/// **`compose` and the door agree**, which is what keeps the crate's own by-parts road from
/// producing a value the parser could not have.
///
/// The suppression fires on both routes because it LIVES on one: the tag parser ends by calling
/// `compose`, so there is a single copy of the rule and nothing for the two to disagree about. A
/// `LanguageId` built from `en` and `Latn` is `en`.
#[test]
fn building_an_identity_by_parts_applies_the_same_rules() {
  let english = Language::new("en").expect("a language");
  let latin = ScriptSubtag::new("Latn").expect("a script");

  let composed = LanguageId::compose(english, Some(latin), None, None);
  assert_eq!(composed, admitted("en"));
  assert_eq!(composed.script(), None, "the suppression fired");

  let chinese = Language::new("zh").expect("a language");
  let simplified = ScriptSubtag::new("Hans").expect("a script");
  let composed = LanguageId::compose(chinese, Some(simplified), None, None);
  assert_eq!(composed, admitted("zh-Hans"));

  // A bare language composes to a tag with three empty seats, which is what `From` gives.
  assert_eq!(LanguageId::from(english), admitted("en"));
}

/// **THE PUBLIC ROAD FROM PARTS IS RENDER-THEN-REPARSE**, and it reaches the same value.
///
/// `compose` is `pub(crate)` because it TRUSTS its tail — a bare text seat carries nothing that says
/// it went through the envelope — so the road a caller outside this crate takes is to spell the
/// seats and walk the standard door, which validates all four in one pass. This is the road a
/// storage layer holding an identity in separate columns takes to read one back, and it is asserted
/// here so that it stays a road.
#[test]
fn the_public_road_from_parts_is_the_standard_door() {
  let seats = [
    ("zh", Some("Hans"), Some("CN"), None),
    ("en", Some("Latn"), Some("US"), None),
    ("de", None, None, Some("1901")),
    ("und", None, None, None),
  ];

  for (language, script, region, rest) in seats {
    let mut spelled = String::from(Language::new(language).expect("a language").as_str());
    for part in [script, region, rest].into_iter().flatten() {
      spelled.push('-');
      spelled.push_str(part);
    }

    let composed = LanguageId::compose(
      Language::new(language).expect("a language"),
      script.map(|s| ScriptSubtag::new(s).expect("a script")),
      region.map(|r| Region::new(r).expect("a region")),
      rest.map(Utf8Bytes::from),
    );

    assert_eq!(
      LanguageId::try_from(spelled.as_str()).expect("the spelled seats parse"),
      composed,
      "`{spelled}` — the two roads part"
    );
  }
}

/// **THE STANDARD CONVERSION TRAITS ARE THE DOOR**, off both carriers, with the same refusals.
#[test]
fn the_try_from_rows_are_the_door() {
  assert_eq!(
    LanguageId::try_from("GER-latn-de").expect("a tag"),
    admitted("de-DE"),
    "the folds apply — it is the same door"
  );
  assert_eq!(
    LanguageId::try_from(Utf8Bytes::from("zh_Hans_CN")).expect("a tag"),
    admitted("zh-Hans-CN"),
  );

  assert_eq!(
    LanguageId::try_from("en-US-!!"),
    Err(ParseLanguageIdError::Tail('!')),
    "and so do the refusals"
  );
  assert_eq!(
    LanguageId::try_from(Utf8Bytes::from("")),
    Err(ParseLanguageIdError::Empty)
  );
}

/// **A REGION THE DOOR REFUSES IS THE REGION SEAT'S ERROR**, carried whole.
///
/// The one place a seat past the language can refuse: a two-letter subtag reaches the region door,
/// and there is a shape it turns down — none, as it happens, since two ASCII letters is exactly what
/// that door takes. So this asserts the ROUTE rather than a refusal, by showing that the error type
/// can carry one.
#[test]
fn the_region_seats_error_has_a_route_to_the_surface() {
  let refused = ParseLanguageIdError::Region(ParseRegionError::Mixed);
  assert!(refused.to_string().contains("region subtag"));
  assert!(core::error::Error::source(&refused).is_some());

  let language = ParseLanguageIdError::Language(ParseLanguageError::Empty);
  assert!(core::error::Error::source(&language).is_some());
  assert!(core::error::Error::source(&ParseLanguageIdError::Empty).is_none());
}

/// **A SUPPRESSION THAT WOULD NOT REPARSE DOES NOT FIRE**, because the fold rewrites the TEXT and
/// the text is this type's stored identity.
///
/// `en-Latn-Cyrl` is the case, and the loose tail envelope is what makes it one: `Latn` sits on the
/// script seat and `Cyrl` on the tail. Dropping `Latn` would render `en-Cyrl` — and the door reads
/// that as the SCRIPT `Cyrl` with an empty tail, so serde and the wire codec, which both write the
/// rendering and read it back through the door, would hand back a different identity than they were
/// given. Three answers move at once: `script`, `rest`, and equality.
///
/// So the script is RETAINED, which costs the fold nothing it was owed: a four-letter tail head is
/// outside BCP 47's tail grammar to begin with, and every tag that keeps a script here is one this
/// crate admits only because a metadata layer carries what its muxer believed.
#[test]
fn a_suppression_that_would_not_reparse_does_not_fire() {
  for (sent, tail, why) in [
    (
      "en-Latn-Cyrl",
      "Cyrl",
      "the tail's head would be read as the script",
    ),
    (
      "en-Latn-Latn",
      "Latn",
      "and the same where the head IS the suppressed script",
    ),
  ] {
    let held = admitted(sent);

    assert_eq!(held.to_string(), sent, "`{sent}` — {why}");
    assert_eq!(
      held.script().as_ref().map(ScriptSubtag::as_str),
      Some("Latn"),
      "`{sent}` — the script is retained"
    );
    assert_eq!(held.region(), None, "`{sent}`");
    assert_eq!(held.rest().map(Utf8Bytes::as_str), Some(tail), "`{sent}`");

    // The whole claim, on the surface serde and the wire codec use.
    let rendered = held.to_string();
    assert_eq!(
      LanguageId::from_str(&rendered).expect("the rendering is a tag"),
      held,
      "`{sent}` — the rendering read back as another value"
    );
  }

  // The value the unguarded fold would have produced IS a different one, which is what makes the
  // guard load-bearing rather than cosmetic.
  let suppressed_text = admitted("en-Cyrl");
  assert_ne!(admitted("en-Latn-Cyrl"), suppressed_text);
  assert_eq!(
    suppressed_text.script().as_ref().map(ScriptSubtag::as_str),
    Some("Cyrl"),
    "the reparser's script slot takes what the fold would have vacated"
  );
  assert_eq!(suppressed_text.rest(), None, "and the tail is gone with it");
  assert_ne!(admitted("en-Latn-Latn"), admitted("en-Latn"));

  // …and THE FOLD STILL FIRES wherever its text reads back as the same value, which is every tag
  // the suppression was written for.
  assert_eq!(canonical("en-Latn"), "en", "the tail is empty — safe");
  assert_eq!(
    canonical("en-Latn-US"),
    "en-US",
    "the region is a seat — safe"
  );
  assert_eq!(
    canonical("en-Latn-1901"),
    "en-1901",
    "a variant-shaped head is not a script — safe"
  );
  assert_eq!(
    canonical("en-Latn-x-Cyrl"),
    "en-x-Cyrl",
    "the tail is a contiguous suffix, so only its HEAD can be reclassified"
  );

  // The guard does not consult the region seat, so a tag whose region would have shielded its tail
  // keeps its script anyway. Retaining is always identity-safe, and this pins the price.
  assert_eq!(canonical("en-Latn-US-Cyrl"), "en-Latn-US-Cyrl");
}

/// **THE REGION SLOT TAKES ITS SUBTAG BEFORE A TAIL CAN OPEN WITH ONE** — the door's greed, which is
/// what makes the region-shaped sibling of the suppression's guard unreachable rather than unhandled.
///
/// The guard above turns away a tail whose head the reparser's SCRIPT slot would claim. The next
/// slot down asks the same question: with the script dropped, would the REGION slot claim a head of
/// two letters or three digits? It would — and no value can be in that state, because the region
/// door is offered that subtag FIRST and only what it turns down becomes the head of a tail. This is
/// that claim, walked over the corpus rather than argued, and it is what the guard's own comment
/// rests on.
#[test]
fn the_region_slot_takes_its_subtag_before_a_tail_can_open_with_one() {
  // Named first: the seat takes its subtag whether or not a script was declared ahead of it, and on
  // both region grammars.
  for (sent, region, rest) in [
    ("en-Latn-US", Some("US"), None),
    ("en-Latn-419", Some("419"), None),
    ("en-Latn-US-Cyrl", Some("US"), Some("Cyrl")),
    ("en-US-posix", Some("US"), Some("posix")),
    ("es-419-x-a", Some("419"), Some("x-a")),
    ("zh-Hans-CN-x-a", Some("CN"), Some("x-a")),
  ] {
    let held = admitted(sent);
    assert_eq!(
      held.region().as_ref().map(Region::as_str),
      region,
      "`{sent}` — the region seat"
    );
    assert_eq!(
      held.rest().map(Utf8Bytes::as_str),
      rest,
      "`{sent}` — the tail begins after it"
    );
  }

  // …and the state the region-shaped sibling would need — an EMPTY region seat beside a tail that
  // opens region-shaped — is nowhere in the corpus, because the door cannot build one.
  let mut walked = 0usize;
  for tag in corpus() {
    let Ok(held) = door(&tag) else { continue };
    walked += 1;

    if held.region().is_some() {
      continue;
    }
    let Some(rest) = held.rest() else { continue };
    let head = rest
      .as_str()
      .split('-')
      .next()
      .expect("a tail has at least one subtag");

    assert!(
      !super::region_shaped(head),
      "`{tag}` holds an empty region beside a region-shaped tail head `{head}` — the sibling case \
       is REACHABLE and the suppression's guard must grow an arm for it"
    );
  }
  assert!(walked > 11_000, "the corpus walked only {walked} tags");
}

/// **EVERY CANONICALISATION IS REPARSE-STABLE**, which is the fixed point the household rests on and
/// the class the suppression's guard belongs to.
///
/// The rendering and the door are inverse over every value the door admits, and this sweeps the
/// class rather than the instance: the case folds on all three seats, the `Preferred-Value` and
/// alpha-3 folds, the grandfathered table, the `und` default, the separator fold and the
/// `Suppress-Script` fold are each a rewrite of the text, and every one of them has to read back as
/// the same four seats. Downstream this is not a tidiness claim — serde and the wire codec store the
/// rendering, so a fold that failed here would mutate stored identities on their next read.
///
/// Both halves are asserted, because they are different claims: the VALUE is a fixed point (the
/// rendering parses back equal) and so is the TEXT (rendering it again is byte-identical). A fold
/// that moved a subtag between two seats of equal rendering would pass the second alone.
///
/// A THIRD claim is asserted beside them and it is the strong one: no admitted value renders text
/// the whole-tag table would fold AGAIN. The two halves say the door is idempotent; this says the
/// door leaves nothing on the table, which is what the iteration buys and what a single pass could
/// not give at any fold order.
#[test]
fn every_canonicalisation_is_reparse_stable() {
  let mut walked = 0usize;

  for tag in corpus() {
    let Ok(held) = door(&tag) else { continue };
    let rendered = held.to_string();

    let reread = admitted(&rendered);
    assert_eq!(
      reread, held,
      "`{tag}` rendered `{rendered}`, which reads back as a different identity"
    );
    assert_eq!(
      reread.to_string(),
      rendered,
      "`{tag}` — the rendering is not a fixed point"
    );
    assert_eq!(
      registry::grandfathered_preferred(&rendered.to_ascii_lowercase()),
      None,
      "`{tag}` rendered `{rendered}`, which the whole-tag table folds again — the \
       canonicalisation stopped short of its fixed point"
    );
    walked += 1;
  }

  assert!(
    walked > 11_000,
    "the sweep is the registry's corpus and its interaction preimages, and it walked only \
     {walked} tags"
  );
}

/// **THE WHOLE-TAG FOLD IS APPLIED UNTIL THE RENDERING STOPS MOVING**, which is the rule that makes
/// the order the other folds are written in stop mattering.
///
/// Every rule in the door rewrites the TAG, and the whole-tag table is keyed by a tag — so a rule
/// whose output lands in that table's preimage moves the tag again. None of the four below is
/// grandfathered as it arrives; each BECOMES one, through a different fold:
///
/// ```text
///   en-Latn-GB-oed  ──Suppress-Script──►  en-GB-oed  ──whole tag──►  en-GB-oxendict
///   eng-GB-oed      ──alpha-3─────────►   en-GB-oed  ──whole tag──►  en-GB-oxendict
///   nor-bok         ──alpha-3─────────►   no-bok     ──whole tag──►  nb
///   zho-guoyu       ──alpha-3─────────►   zh-guoyu   ──whole tag──►  cmn
/// ```
///
/// A single pass would have stored the middle column, and the middle column reads back as the right
/// one — so serde and the wire codec would have handed back a different identity on the next read
/// than the one they were given. That is the same defect the suppression's guard closed for one
/// pair; iterating closes the CLASS, and `MAX_GRANDFATHERED_HOPS` is why iterating terminates.
#[test]
fn the_whole_tag_fold_is_applied_until_the_rendering_stops_moving() {
  for (sent, held, through) in [
    (
      "en-Latn-GB-oed",
      "en-GB-oxendict",
      "the suppression drops `Latn` and leaves a grandfathered tag",
    ),
    (
      "eng-GB-oed",
      "en-GB-oxendict",
      "an mp4's alpha-3 reaches the same text",
    ),
    (
      "ENG-GB-OED",
      "en-GB-oxendict",
      "and the case fold is part of the road there",
    ),
    ("nor-bok", "nb", "alpha-3, then the whole tag"),
    (
      "NOR-BOK",
      "nb",
      "the tail keeps its case, and the lookup folds it anyway",
    ),
    ("zho-guoyu", "cmn", "the terminological Chinese"),
    ("chi-guoyu", "cmn", "and the bibliographic one"),
    ("zho-hakka", "hak", "another row of the same table"),
    ("nor-nyn", "nn", "and the sibling of the first"),
  ] {
    assert_eq!(canonical(sent), held, "`{sent}` — {through}");

    // EQUALITY: the dirty spelling and the canonical one are ONE identity, which is what a stored
    // row and a later query meeting depends on — and one hash bucket, since a fold that moved a
    // value's text without moving its seats would pass `Eq` and fail `Hash`.
    let value = admitted(sent);
    assert_eq!(value, admitted(held), "`{sent}` — two identities, not one");
    let mut distinct = BTreeSet::new();
    distinct.insert(value.clone());
    distinct.insert(admitted(held));
    assert_eq!(distinct.len(), 1, "`{sent}`");

    // Both halves of the fixed point, on the surface serde and the wire codec use.
    let rendered = value.to_string();
    assert_eq!(admitted(&rendered), value, "`{sent}` — the value moved");
    assert_eq!(
      admitted(&rendered).to_string(),
      rendered,
      "`{sent}` — the text moved"
    );
  }

  // The pair the R2 guard exists for stays green, because the iteration does not replace the
  // guard — it is a leg of the same fixed point, and a fold that would not reparse must still not
  // fire however many times it is offered.
  assert_eq!(canonical("en-Latn-Cyrl"), "en-Latn-Cyrl");
  assert_eq!(canonical("en-Latn-Latn"), "en-Latn-Latn");

  // …and the tags that reach a grandfathered row through NO fold are untouched by the loop.
  assert_eq!(canonical("en-GB-oed"), "en-GB-oxendict");
  assert_eq!(canonical("i-klingon"), "tlh");
  assert_eq!(canonical("zh-min"), "zh-min", "kept, so nothing folds");
  assert_eq!(
    canonical("zho-min"),
    "zh-min",
    "…and a fold that LANDS on a kept row stops there too"
  );
}

/// **THE GENERATED HOP BOUND IS THE CHAIN THIS REGISTRY ACTUALLY HAS**, walked through the crate's
/// own composition rather than through the generator's model of it.
///
/// `MAX_GRANDFATHERED_HOPS` is proven at generation, over the tables the generator is about to
/// emit — which means it is proven by a SECOND implementation of the composition rules, living in
/// `xtask`. This is the pin between the two: it re-walks the same graph through `composed` and
/// `Display`, the real ones, and asserts it reaches the number that was emitted. A model that
/// drifted from the fold would fail here rather than certify a bound the door does not have.
///
/// The walk is exhaustive for the reason the generator's is: the loop's first hop may start from
/// any tag a caller sends, but it always LANDS on a `Preferred-Value` of this table, so every hop
/// after the first is an edge of a graph whose nodes are the table's own rows.
#[test]
fn the_generated_hop_bound_is_the_chain_this_registry_has() {
  let mut deepest = 0usize;

  for (start, preferred) in registry::table::GRANDFATHERED {
    let start = start.as_str();
    let mut chain: Vec<String> = std::vec![String::from(start)];
    let mut held = LanguageId::composed(preferred).unwrap_or_else(|refused| {
      panic!("`{start}` prefers `{preferred}`, which the door refuses: {refused}")
    });

    while let Some(next) = held.folds_onto() {
      // `folds_onto` answered, so this value's own rendering IS a row of the table.
      let key = held.to_string().to_ascii_lowercase();
      assert!(
        !chain.contains(&key),
        "the whole-tag fold CYCLES: {} → {key} — canonicalisation has no fixed point to reach",
        chain.join(" → ")
      );

      chain.push(key);
      held = LanguageId::composed(next).unwrap_or_else(|refused| {
        panic!("`{next}` is a `Preferred-Value` the door refuses: {refused}")
      });
    }

    deepest = deepest.max(chain.len());
  }

  assert_eq!(
    deepest,
    registry::MAX_GRANDFATHERED_HOPS,
    "the generator proved a bound of {} and the fold's own chain is {deepest} — regenerate with \
     `cargo xtask gen-lang`",
    registry::MAX_GRANDFATHERED_HOPS
  );
}

/// Ordering is the four seats in order, which is total and stable.
#[test]
fn ordering_walks_the_seats_in_order() {
  let mut sorted = [
    admitted("de-DE"),
    admitted("de"),
    admitted("de-AT"),
    admitted("zh-Hans"),
    admitted("zh"),
  ];
  sorted.sort();

  let tags: Vec<String> = sorted.iter().map(LanguageId::to_string).collect();
  assert_eq!(tags, ["de", "de-AT", "de-DE", "zh", "zh-Hans"]);
}
