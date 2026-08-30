//! The ruling's behaviour table for a script, row by row.
//!
//! The rows are shorter than [`Language`](super::super::Language)'s because the type is: one width,
//! one fold, no alias table. What that leaves is the case fold, the structural gate, the two
//! sentinels, the two postures towards an unregistered subtag — and the ONE pin that is this type's
//! whole reason for existing separately, which is that simplified and traditional Han are two
//! values.

use core::str::FromStr;
use std::{collections::BTreeSet, string::ToString, vec::Vec};

use super::{ParseScriptSubtagError, ScriptSubtag};
use crate::lang::registry;

/// One subtag through the door, or the sentence it was refused with.
fn door(sent: &str) -> Result<ScriptSubtag, ParseScriptSubtagError> {
  ScriptSubtag::new(sent)
}

/// One subtag the door must take, unwrapped.
fn admitted(sent: &str) -> ScriptSubtag {
  door(sent).unwrap_or_else(|refused| panic!("`{sent}` is a script subtag: {refused}"))
}

/// **THE CANONICAL FORM IS ISO 15924's TITLECASE**, and every case reaches it.
#[test]
fn the_canonical_form_is_titlecase_and_the_door_folds_every_case() {
  for (sent, canonical) in [
    ("Latn", "Latn"),
    ("latn", "Latn"),
    ("LATN", "Latn"),
    ("lAtN", "Latn"),
    ("hans", "Hans"),
    ("HANT", "Hant"),
    ("cyrl", "Cyrl"),
    ("zxxx", "Zxxx"),
    ("ZZZZ", "Zzzz"),
  ] {
    assert_eq!(admitted(sent).as_str(), canonical, "`{sent}`");
  }
}

/// **SIMPLIFIED AND TRADITIONAL ARE TWO VALUES.** The ruling's pin, and this type's whole reason for
/// being separate from a language.
///
/// They compare unequal, they hash apart, they store as different bytes and no method here relates
/// them. A fold would make the two indistinguishable in EVERY direction, including the one somebody
/// wants; what a search across them should do belongs to a retrieval layer that has a query and a
/// corpus to weigh, and a primitive has neither.
#[test]
fn simplified_and_traditional_han_are_two_scripts() {
  let simplified = admitted("Hans");
  let traditional = admitted("Hant");

  assert_ne!(simplified, traditional);
  assert_ne!(simplified.as_str(), traditional.as_str());
  assert!(simplified.is_registered() && traditional.is_registered());

  let mut distinct = BTreeSet::new();
  distinct.insert(simplified);
  distinct.insert(traditional);
  assert_eq!(distinct.len(), 2, "two scripts, two hash buckets");

  // Their registry names are the registry's own words, and they say the same thing.
  assert_eq!(simplified.name(), Some("Han (Simplified variant)"));
  assert_eq!(traditional.name(), Some("Han (Traditional variant)"));

  // …and `Hani` is the unified one, which is a THIRD value rather than a meeting point: nothing
  // here folds either variant onto it.
  let unified = admitted("Hani");
  assert_ne!(unified, simplified);
  assert_ne!(unified, traditional);
}

/// **A STRUCTURAL VIOLATION IS REFUSED**, and a script has ONE width, so there is no *too short* to
/// tell apart from *too long*.
#[test]
fn a_structural_violation_is_refused_by_name() {
  for (sent, expected) in [
    ("", ParseScriptSubtagError::Empty),
    ("Lat", ParseScriptSubtagError::WrongWidth),
    ("Latin", ParseScriptSubtagError::WrongWidth),
    ("L", ParseScriptSubtagError::WrongWidth),
    ("Lat_", ParseScriptSubtagError::NotAlphabetic('_')),
    ("Lat1", ParseScriptSubtagError::NotAlphabetic('1')),
    ("Latn-", ParseScriptSubtagError::NotAlphabetic('-')),
    ("漢字", ParseScriptSubtagError::NotAlphabetic('漢')),
  ] {
    assert_eq!(door(sent), Err(expected), "`{sent}`");
  }
}

/// **The ALPHABET is tested before the WIDTH**, so a non-ASCII input is refused for its characters
/// rather than for the width of its encoding.
///
/// `漢字` is SIX bytes and two characters. A width-first door would report that it is not four —
/// true of its encoding, and about something the client did not do.
#[test]
fn a_non_ascii_input_is_refused_for_its_characters() {
  assert_eq!("漢字".len(), 6, "and not four");
  assert_eq!(
    door("漢字"),
    Err(ParseScriptSubtagError::NotAlphabetic('漢'))
  );
}

/// **`Zxxx` and `Zzzz` ARE VALUES**, and neither is the absence of a script.
#[test]
fn the_two_sentinels_are_values_and_not_an_absence() {
  let unwritten = ScriptSubtag::ZXXX;
  let uncoded = ScriptSubtag::ZZZZ;

  assert_eq!(unwritten.as_str(), "Zxxx");
  assert_eq!(uncoded.as_str(), "Zzzz");
  assert!(unwritten.is_unwritten() && !unwritten.is_uncoded());
  assert!(uncoded.is_uncoded() && !uncoded.is_unwritten());
  assert!(unwritten.is_registered() && uncoded.is_registered());
  assert_ne!(unwritten, uncoded, "unwritten is not uncoded");

  assert_eq!(
    admitted("zxxx"),
    unwritten,
    "the door folds them like any other"
  );
  assert_eq!(admitted("ZZZZ"), uncoded);

  let absent: Option<ScriptSubtag> = None;
  assert_ne!(absent, Some(unwritten), "no script is not `Zxxx`");
}

/// **An UNREGISTERED subtag is admitted**, and the private-use range is what names a deliberate one.
#[test]
fn an_unregistered_subtag_is_admitted_and_the_private_range_is_named() {
  let unknown = admitted("Abcd");
  assert_eq!(unknown.as_str(), "Abcd");
  assert!(!unknown.is_registered());
  assert!(!unknown.is_private_use());
  assert_eq!(unknown.name(), None);

  for private in ["Qaaa", "qaaa", "QABX", "Qaam"] {
    let held = admitted(private);
    assert!(held.is_private_use(), "`{private}`");
    assert!(!held.is_registered(), "`{private}`");
  }

  // The bound is exclusive above the range's top, which is what a lexicographic test without a
  // width check would get wrong in the other direction.
  assert!(!admitted("Qaby").is_private_use());
}

/// **EVERY REGISTERED SUBTAG SURVIVES ITS OWN DOOR**, walked whole — and the fold is injective, so
/// no two scripts share a stored value.
///
/// The registry publishes no `Preferred-Value` on a script, so unlike a language EVERY registered
/// script is a fixpoint: there is nothing to fold onto and no exception to state.
#[test]
fn every_registered_script_is_a_fixpoint_and_the_fold_collides_nothing() {
  let mut reached = BTreeSet::new();

  for (subtag, _) in registry::table::SCRIPTS {
    let subtag = subtag.as_str();
    let held = admitted(subtag);

    assert_eq!(held.as_str(), subtag, "`{subtag}` is not a fixpoint");
    assert!(held.is_registered(), "`{subtag}`");
    assert_eq!(
      admitted(&subtag.to_ascii_uppercase()),
      held,
      "`{subtag}` upper-cased"
    );
    assert_eq!(
      admitted(&subtag.to_ascii_lowercase()),
      held,
      "`{subtag}` lower-cased"
    );
    assert!(reached.insert(held), "two scripts folded onto `{subtag}`");
  }

  assert_eq!(reached.len(), registry::SCRIPT_COUNT);
}

/// The rendering and the parse are inverse, and the [`Debug`](core::fmt::Debug) face prints the
/// subtag.
#[test]
fn the_rendering_and_the_parse_are_inverse() {
  for sent in ["Latn", "hans", "ZZZZ", "Qaaa", "Abcd"] {
    let held = admitted(sent);
    let rendered = held.to_string();

    assert_eq!(
      ScriptSubtag::from_str(&rendered).expect("canonical text"),
      held
    );
    assert_eq!(rendered, held.as_str());
  }

  assert_eq!(
    std::format!("{:?}", admitted("latn")),
    r#"ScriptSubtag("Latn")"#
  );
}

/// Ordering is alphabetical over the canonical spelling — a sort key, and nothing more, exactly as
/// on a language.
#[test]
fn ordering_is_alphabetical_over_the_canonical_spelling() {
  let mut sorted = [
    admitted("Zzzz"),
    admitted("hans"),
    admitted("LATN"),
    admitted("Hant"),
  ];
  sorted.sort();

  let spellings: Vec<&str> = sorted.iter().map(ScriptSubtag::as_str).collect();
  assert_eq!(spellings, ["Hans", "Hant", "Latn", "Zzzz"]);
}

/// **The `Suppress-Script` knowledge is NOT this type's**, which is the ruling's second pin.
///
/// It is a column of the LANGUAGE record — English implies Latin — so it hangs off
/// [`Language`](super::super::Language) and is spent by the COMPOSITION, where `en-Latn` becomes
/// `en`. A script cannot answer it: `Latn` is implied by 90 languages and by none of them in
/// particular.
///
/// Pinned from the language's side because that is where the method is, and from the table's side
/// because that is where the knowledge is. What SPENDS it is `LanguageId`.
#[test]
fn a_script_does_not_know_which_languages_imply_it() {
  let latin = admitted("Latn");
  assert!(latin.is_registered());

  let implying = registry::table::LANGUAGE_SUPPRESS_SCRIPT
    .iter()
    .filter(|(_, script)| *script == latin.as_str())
    .count();

  assert!(implying > 1, "`Latn` is implied by {implying} languages");
  assert_eq!(
    registry::language_suppress_script("Latn"),
    None,
    "not a language"
  );
}
