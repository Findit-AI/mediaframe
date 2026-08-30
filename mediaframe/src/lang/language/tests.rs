//! The ruling's behaviour table, row by row, plus the properties those rows are instances of.
//!
//! Each `#[test]` below is one clause of the ruling this type implements, and the tables inside
//! them are transcribed from it: the canonical form, the four folds, `und` as a value, an
//! unregistered subtag admitted and named as such, and a structural violation refused.
//!
//! Where a clause is about the WHOLE registry rather than about a row — that a fold never collides
//! two languages into one, that every registered subtag survives its own door — the test walks all
//! 8,275 of them, for the reason the registry's own tests give: the subject is a table.

use core::str::FromStr;
use std::{
  collections::{BTreeMap, BTreeSet},
  string::{String, ToString},
  vec::Vec,
};

use smol_bytes::Utf8Bytes;

use super::{Language, ParseLanguageError};
use crate::lang::registry;

/// One subtag through the door, or the sentence it was refused with.
fn door(sent: &str) -> Result<Language, ParseLanguageError> {
  Language::new(sent)
}

/// One subtag the door must take, unwrapped.
fn admitted(sent: &str) -> Language {
  door(sent).unwrap_or_else(|refused| panic!("`{sent}` is a language subtag: {refused}"))
}

/// **THE CANONICAL FORM IS THE SHORTEST SPELLING**, which is the ruling's first clause.
///
/// Two letters where BCP 47 has two, and three where it has no two — so `de` and `zh` are
/// two-letter and `yue` and `fil` are three, and neither is a preference this house expressed.
#[test]
fn the_canonical_form_is_the_shortest_bcp47_spelling() {
  for (sent, canonical) in [
    ("de", "de"),
    ("zh", "zh"),
    ("yue", "yue"),
    ("fil", "fil"),
    ("und", "und"),
  ] {
    assert_eq!(admitted(sent).as_str(), canonical, "`{sent}`");
  }
}

/// **THE FOLD TABLE**, transcribed. Four columns of dirt, one canonical value out of each.
///
/// The rows are the ruling's own examples plus the general case each stands for, because a fold
/// that worked for `ger` and nothing else would pass a table of `ger`.
#[test]
fn the_door_folds_every_spelling_the_ruling_names() {
  for (sent, canonical, why) in [
    // ASCII case, on every width.
    ("DE", "de", "upper case"),
    ("De", "de", "mixed case"),
    ("YUE", "yue", "upper case, three letters"),
    ("UND", "und", "the sentinel is folded like anything else"),
    // ISO 639-2/B — what an mkv writes.
    ("ger", "de", "bibliographic"),
    ("GER", "de", "bibliographic, upper case"),
    ("fre", "fr", "bibliographic"),
    ("chi", "zh", "bibliographic"),
    // ISO 639-2/T — what an mp4 writes.
    ("deu", "de", "terminological"),
    ("fra", "fr", "terminological"),
    ("zho", "zh", "terminological"),
    // Deprecated, with a replacement the registry names.
    ("iw", "he", "deprecated"),
    ("IW", "he", "deprecated, upper case"),
    ("in", "id", "deprecated"),
    ("ji", "yi", "deprecated"),
    ("mo", "ro", "deprecated"),
  ] {
    assert_eq!(admitted(sent).as_str(), canonical, "`{sent}` — {why}");
  }
}

/// **The two folds compose, and a value that needs neither is untouched.**
///
/// The order is alphabet-then-preference and each fires at most once, which is what makes `GER` one
/// pass rather than a loop that could not say when to stop.
#[test]
fn a_subtag_that_needs_no_fold_is_held_as_it_arrived() {
  for already in ["de", "en", "fr", "zh", "yue", "haw", "und"] {
    assert_eq!(admitted(already).as_str(), already);
    assert!(admitted(already).is_registered(), "`{already}`");
  }
}

/// **AN MKV'S GERMAN AND AN MP4'S GERMAN ARE ONE VALUE** — the whole reason the fold is at the
/// door rather than at comparison time.
///
/// Equality is the canonical subtag, so the two containers' tags are one map key and one hash
/// bucket, and nothing downstream learns that ISO 639-2 has two alphabets.
#[test]
fn two_containers_spellings_of_one_language_are_equal() {
  let mkv = admitted("ger");
  let mp4 = admitted("deu");
  let bcp47 = admitted("de");

  assert_eq!(mkv, mp4);
  assert_eq!(mp4, bcp47);
  assert_eq!(mkv, bcp47);

  let mut distinct = BTreeSet::new();
  distinct.insert(mkv);
  distinct.insert(mp4);
  distinct.insert(bcp47);
  assert_eq!(distinct.len(), 1, "one language, one hash bucket");
}

/// **`und` IS A VALUE**, and the ruling's distinction is between it and the ABSENCE of one.
///
/// A `Language` holding `und` says a muxer looked and could not tell. A track with no language at
/// all is `Option::None`, which is a different fact and a different observable — and the pair below
/// is where the two are visibly not the same thing.
#[test]
fn undetermined_is_a_value_and_not_an_absence() {
  let undetermined = Language::UND;

  assert_eq!(undetermined.as_str(), "und");
  assert!(undetermined.is_undetermined());
  assert!(undetermined.is_registered(), "the registry registers `und`");
  assert_eq!(undetermined.name(), Some("Undetermined"));
  assert_eq!(admitted("und"), undetermined);

  assert!(!admitted("de").is_undetermined());

  let absent: Option<Language> = None;
  let determined: Option<Language> = Some(undetermined);
  assert_ne!(absent, determined, "no language is not `und`");
}

/// **A STRUCTURALLY VALID SUBTAG THE REGISTRY HAS NEVER HEARD OF IS ADMITTED**, and the predicate
/// is what names it.
#[test]
fn an_unregistered_subtag_is_admitted_and_named_as_such() {
  for unknown in ["xyz", "qqq", "abcdefgh"] {
    let held = admitted(unknown);

    assert_eq!(held.as_str(), unknown, "held as it arrived");
    assert!(!held.is_registered(), "`{unknown}`");
    assert_eq!(held.name(), None, "`{unknown}`");
  }

  // …and the private-use range is the same posture with a name on it, which is what tells a
  // deliberate private tag apart from a typo. Both are unregistered; only one is intentional.
  let private = admitted("qaa");
  assert!(private.is_private_use());
  assert!(!private.is_registered());

  let typo = admitted("xyz");
  assert!(!typo.is_private_use());
  assert!(!typo.is_registered());
}

/// **A STRUCTURAL VIOLATION IS REFUSED**, and the refusal names the rule rather than reporting that
/// the input was malformed.
#[test]
fn a_structural_violation_is_refused_by_name() {
  for (sent, expected) in [
    ("", ParseLanguageError::Empty),
    ("e", ParseLanguageError::TooShort),
    ("abcdefghi", ParseLanguageError::TooLong),
    ("zh_CN", ParseLanguageError::NotAlphabetic('_')),
    ("zh-CN", ParseLanguageError::NotAlphabetic('-')),
    ("d3", ParseLanguageError::NotAlphabetic('3')),
    ("419", ParseLanguageError::NotAlphabetic('4')),
    ("dé", ParseLanguageError::NotAlphabetic('é')),
    ("日本語", ParseLanguageError::NotAlphabetic('日')),
    (" de", ParseLanguageError::NotAlphabetic(' ')),
  ] {
    assert_eq!(door(sent), Err(expected), "`{sent}`");
  }
}

/// **The ALPHABET is tested before the LENGTH**, which is what gives `日本語` the sentence about
/// characters rather than the one about length.
///
/// Nine bytes and three characters: a length-first door would refuse it as too long, which is true
/// of its encoding and says nothing a client can act on.
#[test]
fn a_non_ascii_input_is_refused_for_its_characters_and_not_its_width() {
  assert_eq!("日本語".len(), 9, "wider than the eight-byte ceiling");
  assert_eq!(door("日本語"), Err(ParseLanguageError::NotAlphabetic('日')));

  // The length rule still fires where the input IS letters.
  assert_eq!(door("abcdefghi"), Err(ParseLanguageError::TooLong));
  assert!(door("abcdefgh").is_ok(), "eight is the ceiling, inclusive");
}

/// **EVERY REGISTERED SUBTAG SURVIVES ITS OWN DOOR**, walked whole — and a fold collides no two
/// languages into one.
///
/// The property a spot check cannot hold. A canonical subtag has to be a FIXPOINT of the door
/// (otherwise a stored value would change meaning on the way back in), and the fold has to be
/// INJECTIVE over the registry's own subtags (otherwise two languages would share one stored value
/// and nothing could ever separate them again).
///
/// The exceptions are the 112 subtags the registry itself supersedes, and they are exceptions to
/// the fixpoint rather than to the injectivity: `iw` folds to `he`, so `iw` is not a fixpoint, and
/// `he` still reaches only itself.
#[test]
fn a_canonical_subtag_is_a_fixpoint_and_the_fold_collides_nothing() {
  let mut reached: BTreeMap<String, &str> = BTreeMap::new();

  for (subtag, _) in registry::table::LANGUAGES {
    let subtag = subtag.as_str();
    let held = admitted(subtag);

    match registry::language_preferred(subtag) {
      // A superseded subtag is the one kind that is not its own fixpoint, and where it lands is
      // exactly what the registry says.
      Some(preferred) => assert_eq!(held.as_str(), preferred, "`{subtag}`"),
      None => {
        assert_eq!(held.as_str(), subtag, "`{subtag}` is not a fixpoint");
        assert_eq!(
          reached.insert(String::from(held.as_str()), subtag),
          None,
          "two languages folded onto `{}`",
          held.as_str()
        );
      }
    }

    // Whatever it folded to, folding again changes nothing — the door is idempotent, which is what
    // makes a stored value safe to read back through it.
    assert_eq!(
      admitted(held.as_str()),
      held,
      "`{subtag}` is not idempotent"
    );
  }

  assert_eq!(reached.len(), registry::LANGUAGE_COUNT - 112);
}

/// **EVERY ALPHA-3 CODE REACHES A REGISTERED LANGUAGE**, walked whole — the second vendored file's
/// whole contribution, asserted at this type's door rather than at the table.
#[test]
fn every_iso_639_2_code_reaches_a_registered_language() {
  for (code, shortest) in registry::table::ALPHA3 {
    let code = code.as_str();
    let held = admitted(code);

    assert_eq!(held.as_str(), *shortest, "`{code}`");
    assert!(held.is_registered(), "`{code}` reached `{shortest}`");
    assert_eq!(
      admitted(&code.to_ascii_uppercase()),
      held,
      "`{code}` upper-cased"
    );
  }
}

/// The two spellings of one rendering: [`Display`](core::fmt::Display) writes the canonical subtag
/// and [`FromStr`] reads it back, so the pair is a round trip and not merely a pretty printer.
#[test]
fn the_rendering_and_the_parse_are_inverse() {
  for sent in ["de", "GER", "iw", "yue", "und", "qaa", "xyz"] {
    let held = admitted(sent);
    let rendered = held.to_string();

    assert_eq!(Language::from_str(&rendered).expect("canonical text"), held);
    assert_eq!(rendered, held.as_str());
  }
}

/// The [`Debug`](core::fmt::Debug) face prints the subtag rather than the text seat's own shape,
/// which is what keeps an assertion message readable.
#[test]
fn the_debug_face_prints_the_subtag() {
  assert_eq!(std::format!("{:?}", admitted("ger")), r#"Language("de")"#);
}

/// **Deprecation survives the fold where the registry names no successor**, which is the honest
/// answer and not a gap.
///
/// A canonical value can be deprecated: 120 language subtags carry `Deprecated` and no
/// `Preferred-Value`, so there is nowhere to fold them to. `iw` is never one of these — it prefers
/// `he`, so no `Language` ever holds it and `is_deprecated` is `false` for the value it became.
#[test]
fn a_canonical_value_can_still_be_deprecated() {
  let orphaned = registry::table::LANGUAGE_DEPRECATED
    .iter()
    .copied()
    .find(|subtag| registry::language_preferred(subtag).is_none())
    .expect("120 of them");

  let held = admitted(orphaned);
  assert_eq!(held.as_str(), orphaned);
  assert!(held.is_deprecated());
  assert!(held.is_registered());

  let replaced = admitted("iw");
  assert_eq!(replaced.as_str(), "he");
  assert!(!replaced.is_deprecated(), "the value it became is current");
}

/// The `Suppress-Script` reading, published here because it is a fact about the LANGUAGE — and
/// pinned here because it is the one that keeps `zh-Hans` standing.
#[test]
fn a_language_publishes_the_script_it_implies() {
  assert_eq!(admitted("en").suppressed_script(), Some("Latn"));
  assert_eq!(admitted("he").suppressed_script(), Some("Hebr"));
  assert_eq!(
    admitted("iw").suppressed_script(),
    Some("Hebr"),
    "after the fold"
  );
  assert_eq!(admitted("zh").suppressed_script(), None);
  assert_eq!(admitted("xyz").suppressed_script(), None, "unregistered");
}

/// The text seat is [`Utf8Bytes`], and the conversions out are the three a caller needs — the
/// borrow, the seat itself, and the owned `String` a text boundary crosses on.
#[test]
fn the_text_seat_converts_both_ways() {
  let held = admitted("ger");

  assert_eq!(held.as_ref() as &str, "de");
  assert_eq!(String::from(held), "de");
  assert_eq!(Utf8Bytes::from(held), Utf8Bytes::from("de"));
}

/// Ordering is alphabetical over the CANONICAL spelling, which is what makes it a usable sort key
/// and nothing more than that.
#[test]
fn ordering_is_alphabetical_over_the_canonical_spelling() {
  let mut sorted = [
    admitted("zh"),
    admitted("ger"),
    admitted("en"),
    admitted("fr"),
  ];
  sorted.sort();

  let spellings: Vec<&str> = sorted.iter().map(Language::as_str).collect();
  assert_eq!(spellings, ["de", "en", "fr", "zh"]);
}
