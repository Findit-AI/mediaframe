//! What the generated table has to be TRUE of for the lookups above to be lookups at all, and the
//! handful of rows the rulings this house implements name by hand.
//!
//! The first group is structural — sortedness, agreement between the two vendored files, one-hop
//! folds — and every one of it walks the WHOLE table rather than a sample, because the subject is a
//! table and one hand-picked row proves nothing about eight thousand. The generator asserts the
//! same properties before it emits, so these are a second opinion held by a different program: a
//! table hand-edited past `cargo xtask check` still fails here.
//!
//! The second group is the ROWS. Every one of them is a fold a ruling spelled out — `ger` and `deu`
//! onto `de`, `iw` onto `he`, `BU` onto `MM`, `en` implying `Latn` and `zh` implying nothing — and
//! each is here so that the sentence and the data can be compared by running something.

use std::vec::Vec;

use super::{
  GRANDFATHERED_COUNT, LANGUAGE_COUNT, LANGUAGE_PRIVATE_USE, REGION_COUNT, REGION_PRIVATE_USE,
  SCRIPT_COUNT, SCRIPT_PRIVATE_USE, alpha3, grandfathered_preferred, is_grandfathered,
  language_is_deprecated, language_is_private_use, language_name, language_preferred,
  language_suppress_script, region_is_deprecated, region_is_private_use, region_name,
  region_preferred, script_is_private_use, script_name, table,
};

/// Every pair table this module binary-searches is SORTED on its first element.
///
/// The property the lookups are correct under, and the one a generator bug would break silently:
/// `binary_search` over an unsorted slice does not fail, it answers `Err` for rows that are there.
#[test]
fn every_pair_table_is_sorted() {
  for (name, table) in [
    ("LANGUAGES", table::LANGUAGES),
    ("LANGUAGE_PREFERRED", table::LANGUAGE_PREFERRED),
    ("LANGUAGE_SUPPRESS_SCRIPT", table::LANGUAGE_SUPPRESS_SCRIPT),
    ("ALPHA3", table::ALPHA3),
    ("SCRIPTS", table::SCRIPTS),
    ("REGIONS", table::REGIONS),
    ("REGION_PREFERRED", table::REGION_PREFERRED),
    ("GRANDFATHERED", table::GRANDFATHERED),
  ] {
    assert!(
      table.windows(2).all(|pair| pair[0].0 < pair[1].0),
      "{name} is not sorted, so its lookups are not lookups"
    );
  }

  for (name, table) in [
    ("LANGUAGE_DEPRECATED", table::LANGUAGE_DEPRECATED),
    ("REGION_DEPRECATED", table::REGION_DEPRECATED),
    ("GRANDFATHERED_KEPT", table::GRANDFATHERED_KEPT),
  ] {
    assert!(
      table.windows(2).all(|pair| pair[0] < pair[1]),
      "{name} is not sorted"
    );
  }
}

/// **Every registered subtag answers to its own name**, walked whole.
///
/// A lookup that missed rows would show as a `None` somewhere in the middle of eight thousand
/// binary searches, which is exactly the failure a spot check does not find.
#[test]
fn every_registered_subtag_is_found_by_its_own_subtag() {
  for (subtag, name) in table::LANGUAGES {
    assert_eq!(language_name(subtag), Some(*name), "language `{subtag}`");
  }
  for (subtag, name) in table::SCRIPTS {
    assert_eq!(script_name(subtag), Some(*name), "script `{subtag}`");
  }
  for (subtag, name) in table::REGIONS {
    assert_eq!(region_name(subtag), Some(*name), "region `{subtag}`");
  }

  assert_eq!(LANGUAGE_COUNT, table::LANGUAGES.len());
  assert_eq!(SCRIPT_COUNT, table::SCRIPTS.len());
  assert_eq!(REGION_COUNT, table::REGIONS.len());
  assert_eq!(
    GRANDFATHERED_COUNT, 26,
    "BCP 47 grandfathered twenty-six tags"
  );
}

/// **A fold is ONE hop**, on both columns that have one.
///
/// `Language`'s canonicalisation applies `Preferred-Value` exactly once, so a preferred value that
/// itself preferred something else would leave a value this house calls canonical and the registry
/// calls superseded — and it would do it silently, the second hop being one nobody takes.
#[test]
fn a_preferred_value_prefers_nothing_itself() {
  for (subtag, preferred) in table::LANGUAGE_PREFERRED {
    assert!(
      language_name(preferred).is_some(),
      "`{subtag}` prefers `{preferred}`, which is not registered"
    );
    assert_eq!(
      language_preferred(preferred),
      None,
      "`{subtag}` prefers `{preferred}`, which prefers something else"
    );
  }

  for (subtag, preferred) in table::REGION_PREFERRED {
    assert!(
      region_name(preferred).is_some(),
      "`{subtag}` prefers `{preferred}`, which is not registered"
    );
    assert_eq!(
      region_preferred(preferred),
      None,
      "`{subtag}` prefers `{preferred}`, which prefers something else"
    );
  }
}

/// **The two vendored files AGREE**, which is the premise the second one is vendored under.
///
/// It supplies spellings BCP 47 leaves out. So every key of the alpha-3 fold must be a word the
/// registry does NOT carry — otherwise two tables would answer one subtag — and every value must be
/// one it does, or the fold would produce a language that is not registered.
#[test]
fn the_alpha3_fold_supplies_only_what_bcp47_omits() {
  for (code, shortest) in table::ALPHA3 {
    assert_eq!(
      language_name(code),
      None,
      "ISO 639-2 folds `{code}` onto `{shortest}` and BCP 47 registers `{code}` itself"
    );
    assert!(
      language_name(shortest).is_some(),
      "ISO 639-2 folds `{code}` onto `{shortest}`, which BCP 47 does not register"
    );
  }
}

/// A `Suppress-Script` names a script the composition then compares against, so one naming no
/// registered script would be a suppression that never fires and never says why.
#[test]
fn every_suppressed_script_is_a_registered_script() {
  for (subtag, script) in table::LANGUAGE_SUPPRESS_SCRIPT {
    assert!(
      script_name(script).is_some(),
      "`{subtag}` suppresses `{script}`, which is not a registered script"
    );
  }
}

/// **THE RULING'S OWN EXAMPLE.** An mkv's `ger` and an mp4's `deu` are one language, and it is `de`.
///
/// The row the second vendored file exists for. Neither word is in the BCP 47 registry — asserted
/// here rather than assumed, because it is the whole reason there are two files — and both reach
/// the two-letter code in ONE hop rather than through each other.
#[test]
fn the_bibliographic_and_terminological_codes_both_reach_the_shortest_one() {
  assert_eq!(alpha3("ger"), Some("de"));
  assert_eq!(alpha3("deu"), Some("de"));

  assert_eq!(language_name("ger"), None, "BCP 47 has no `ger`");
  assert_eq!(language_name("deu"), None, "BCP 47 has no `deu`");
  assert_eq!(language_name("de"), Some("German"));

  // The general case, not only German's: an alpha-3 code with a two-letter spelling folds onto it.
  assert_eq!(alpha3("fre"), Some("fr"));
  assert_eq!(alpha3("fra"), Some("fr"));
  assert_eq!(alpha3("spa"), Some("es"));

  // …and a language with NO two-letter code is already the shortest spelling, so it folds nowhere
  // and the registry is where it lives.
  assert_eq!(alpha3("haw"), None);
  assert_eq!(language_name("haw"), Some("Hawaiian"));
  assert_eq!(alpha3("yue"), None);
  assert_eq!(language_name("yue"), Some("Yue Chinese"));
  assert_eq!(alpha3("fil"), None);
  assert_eq!(language_name("fil"), Some("Filipino"));
}

/// **THE RULING'S OTHER EXAMPLE.** A muxer's forty-year-old `iw` is `he`, and the registry says so.
#[test]
fn a_deprecated_subtag_reaches_the_one_that_replaced_it() {
  assert_eq!(language_preferred("iw"), Some("he"));
  assert!(language_is_deprecated("iw"));
  assert_eq!(language_name("he"), Some("Hebrew"));
  assert!(!language_is_deprecated("he"));

  assert_eq!(language_preferred("in"), Some("id"));
  assert_eq!(language_preferred("ji"), Some("yi"));
  assert_eq!(language_preferred("mo"), Some("ro"));
}

/// **Deprecation and replacement are two columns**, and 120 language subtags carry the first
/// without the second.
///
/// Those stay themselves, which is the only honest answer available: the registry deprecated the
/// subtag and named nothing to use instead. A fold that quietly dropped them, or one that refused
/// them, would each be inventing an answer the registry declined to give.
#[test]
fn a_deprecated_subtag_without_a_replacement_stays_itself() {
  let orphaned: Vec<&str> = table::LANGUAGE_DEPRECATED
    .iter()
    .copied()
    .filter(|subtag| language_preferred(subtag).is_none())
    .collect();

  assert_eq!(orphaned.len(), 120);

  for subtag in orphaned {
    assert!(
      language_name(subtag).is_some(),
      "`{subtag}` is deprecated and still registered"
    );
  }

  // The regions do it too, and there the reason is legible: a state that dissolved into several has
  // no single successor to name.
  for subtag in ["AN", "CS", "NT", "SU", "YU"] {
    assert!(region_is_deprecated(subtag), "`{subtag}`");
    assert_eq!(region_preferred(subtag), None, "`{subtag}`");
    assert!(region_name(subtag).is_some(), "`{subtag}`");
  }
}

/// **`BU` is `MM`** — the region column's own ruling, and the five other rows beside it.
#[test]
fn a_deprecated_region_reaches_the_one_that_replaced_it() {
  assert_eq!(region_preferred("BU"), Some("MM"));
  assert!(region_is_deprecated("BU"));
  assert_eq!(region_name("MM"), Some("Myanmar"));

  assert_eq!(region_preferred("ZR"), Some("CD"));
  assert_eq!(region_preferred("TP"), Some("TL"));
  assert_eq!(region_preferred("DD"), Some("DE"));
  assert_eq!(region_preferred("FX"), Some("FR"));
  assert_eq!(region_preferred("YD"), Some("YE"));
}

/// **`en` implies `Latn` and `zh` implies NOTHING**, which is the whole of why `en-Latn` composes as
/// `en` while `zh-Hans` composes as itself.
///
/// The pin for the ruling that metadata does not fold the simplified/traditional distinction: it
/// survives because the registry declines to suppress a script for Chinese, and this is the row
/// that says so.
#[test]
fn a_suppressed_script_is_implied_and_chinese_suppresses_none() {
  assert_eq!(language_suppress_script("en"), Some("Latn"));
  assert_eq!(language_suppress_script("fr"), Some("Latn"));
  assert_eq!(language_suppress_script("he"), Some("Hebr"));

  assert_eq!(
    language_suppress_script("zh"),
    None,
    "Chinese implies no script"
  );
  assert!(script_name("Hans").is_some());
  assert!(script_name("Hant").is_some());
}

/// The grandfathered tags: twenty-one fold, five do not, and both halves are one question here.
#[test]
fn a_grandfathered_tag_folds_where_the_registry_names_a_successor() {
  assert_eq!(grandfathered_preferred("i-klingon"), Some("tlh"));
  assert_eq!(grandfathered_preferred("zh-guoyu"), Some("cmn"));
  assert_eq!(grandfathered_preferred("art-lojban"), Some("jbo"));
  assert_eq!(grandfathered_preferred("no-bok"), Some("nb"));

  for tag in [
    "cel-gaulish",
    "i-default",
    "i-enochian",
    "i-mingo",
    "zh-min",
  ] {
    assert!(is_grandfathered(tag), "`{tag}`");
    assert_eq!(grandfathered_preferred(tag), None, "`{tag}`");
  }

  assert!(!is_grandfathered("en-GB"), "an ordinary composition");
  assert_eq!(table::GRANDFATHERED_KEPT.len(), 5);
}

/// **The sentinels every ruling calls a first-class value are REGISTERED**, so nothing about them is
/// a special case this house invented.
#[test]
fn the_undetermined_sentinels_are_ordinary_registered_subtags() {
  assert_eq!(language_name("und"), Some("Undetermined"));
  assert_eq!(script_name("Zxxx"), Some("Code for unwritten documents"));
  assert_eq!(script_name("Zzzz"), Some("Code for uncoded script"));
  assert_eq!(region_name("ZZ"), Some("Private use"));

  for sentinel in ["und", "Zxxx", "Zzzz", "ZZ"] {
    assert!(!sentinel.is_empty());
  }
  assert!(!language_is_deprecated("und"));
}

/// **A private-use subtag is structurally fine, unnamed and unregistered — three answers, not one.**
///
/// The range is a single registry record naming a block, so nothing inside it carries a
/// `Description`, and the predicate is what tells such a subtag apart from one nobody has heard of.
#[test]
fn a_private_use_subtag_is_in_range_and_out_of_the_roster() {
  assert_eq!(LANGUAGE_PRIVATE_USE, ("qaa", "qtz"));

  for subtag in ["qaa", "qtz", "qmm"] {
    assert!(language_is_private_use(subtag), "`{subtag}`");
    assert_eq!(language_name(subtag), None, "`{subtag}` carries no name");
  }
  assert!(!language_is_private_use("qua"), "past the range's top");
  assert!(!language_is_private_use("de"));

  assert_eq!(SCRIPT_PRIVATE_USE, ("Qaaa", "Qabx"));
  assert!(script_is_private_use("Qaaa"));
  assert!(script_is_private_use("Qabx"));
  assert!(!script_is_private_use("Qaby"));
  assert!(!script_is_private_use("Latn"));

  assert_eq!(REGION_PRIVATE_USE, [("QM", "QZ"), ("XA", "XZ")]);
  for subtag in ["QM", "QZ", "QR", "XA", "XZ", "XK"] {
    assert!(region_is_private_use(subtag), "`{subtag}`");
  }
  assert!(!region_is_private_use("QL"), "below the first range");
  assert!(!region_is_private_use("DE"));
}

/// **The WIDTH test is what makes a range test a range test**, rather than a lexicographic accident.
///
/// `qq` sorts between `qaa` and `qtz` as text and names nothing inside the block, the range being
/// three letters wide. Every registry range is one width, so requiring it costs nothing — and
/// without it a two-letter subtag would be reported as privately used.
#[test]
fn a_range_admits_only_subtags_of_its_own_width() {
  assert!(
    !language_is_private_use("qq"),
    "two letters, inside the bounds as text"
  );
  assert!(!language_is_private_use("qaaa"), "four letters");
  assert!(!region_is_private_use("Q"), "one letter");
  assert!(!script_is_private_use("Qaa"), "three letters");
}

/// **The lookups take the registry's own case and fold nothing**, which is the floor this module
/// documents rather than a defect.
///
/// A value that reaches one of them has been folded once already, by the constructor of the type
/// that holds it. Asserting the floor is what keeps a later reader from "fixing" it and paying for
/// a fold twice on every lookup.
#[test]
fn a_lookup_is_case_sensitive_and_says_so() {
  assert_eq!(language_name("DE"), None);
  assert_eq!(language_name("de"), Some("German"));

  assert_eq!(script_name("latn"), None);
  assert_eq!(script_name("LATN"), None);
  assert_eq!(script_name("Latn"), Some("Latin"));

  assert_eq!(region_name("de"), None);
  assert_eq!(region_name("DE"), Some("Germany"));
}

/// The three-digit half of the region roster — the UN M.49 area codes, which share one table with
/// the two-letter country codes because the registry keeps them in one list.
#[test]
fn the_region_roster_holds_both_grammars() {
  let digits = table::REGIONS
    .iter()
    .filter(|(subtag, _)| subtag.bytes().all(|byte| byte.is_ascii_digit()))
    .count();

  assert_eq!(digits, 31, "the M.49 areas");
  assert_eq!(region_name("419"), Some("Latin America and the Caribbean"));
  assert_eq!(region_name("001"), Some("World"));
  assert_eq!(REGION_COUNT - digits, table::REGIONS.len() - 31);
}
