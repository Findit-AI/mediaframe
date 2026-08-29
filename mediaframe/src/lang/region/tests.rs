//! The ruling's behaviour table for a region, row by row.
//!
//! Two grammars is what makes this table longer than a script's: the width rules are separate, the
//! case fold reaches only one arm, and there is a way to be alphanumeric and belong to neither. On
//! top of that sit the two rows the ruling names by hand — `BU` folding onto `MM`, and the
//! deprecated subtags that name no successor and stay themselves.

use core::str::FromStr;
use std::{
  collections::BTreeMap,
  string::{String, ToString},
  vec::Vec,
};

use super::{ParseRegionError, Region};
use crate::lang::registry;

/// One subtag through the door, or the sentence it was refused with.
fn door(sent: &str) -> Result<Region, ParseRegionError> {
  Region::new(sent)
}

/// One subtag the door must take, unwrapped.
fn admitted(sent: &str) -> Region {
  door(sent).unwrap_or_else(|refused| panic!("`{sent}` is a region subtag: {refused}"))
}

/// **TWO GRAMMARS, ONE TYPE** — a country code case-folded to upper, and an area code held exactly
/// as it arrived.
#[test]
fn both_grammars_reach_their_own_canonical_form() {
  for (sent, canonical) in [
    ("DE", "DE"),
    ("de", "DE"),
    ("dE", "DE"),
    ("tw", "TW"),
    ("419", "419"),
    ("001", "001"),
    ("150", "150"),
    ("zz", "ZZ"),
  ] {
    assert_eq!(admitted(sent).as_str(), canonical, "`{sent}`");
  }
}

/// **A LEADING ZERO IS PART OF THE CODE**, not formatting — so `001` is *World* and `1` is nothing.
///
/// The pin for the digit arm's whole posture: nothing is stripped and nothing is restored, which is
/// also why the wire door refuses a JSON number.
#[test]
fn an_area_codes_leading_zeros_are_part_of_it() {
  assert_eq!(admitted("001").as_str(), "001");
  assert_eq!(admitted("001").name(), Some("World"));

  assert_eq!(door("1"), Err(ParseRegionError::WrongDigitWidth));
  assert_eq!(door("01"), Err(ParseRegionError::WrongDigitWidth));
  assert_eq!(door("0001"), Err(ParseRegionError::WrongDigitWidth));
}

/// **`BU` IS `MM`** — the ruling's own example, and the five rows beside it.
#[test]
fn a_deprecated_region_folds_onto_the_one_that_replaced_it() {
  for (sent, canonical) in [
    ("BU", "MM"),
    ("bu", "MM"),
    ("ZR", "CD"),
    ("TP", "TL"),
    ("DD", "DE"),
    ("FX", "FR"),
    ("YD", "YE"),
  ] {
    let held = admitted(sent);
    assert_eq!(held.as_str(), canonical, "`{sent}`");
    assert!(!held.is_deprecated(), "`{sent}` became a current region");
  }

  assert_eq!(admitted("BU"), admitted("MM"), "one value, one stored row");
  assert_eq!(admitted("MM").name(), Some("Myanmar"));
}

/// **A DEPRECATED REGION WITH NO SUCCESSOR STAYS ITSELF**, which is the only honest answer.
///
/// Five of the eleven name none, and each is a state that dissolved into SEVERAL: inventing a
/// successor for `YU` would be picking one of Yugoslavia's six.
#[test]
fn a_deprecated_region_without_a_successor_stays_itself() {
  for orphaned in ["AN", "CS", "NT", "SU", "YU"] {
    let held = admitted(orphaned);

    assert_eq!(held.as_str(), orphaned, "`{orphaned}`");
    assert!(held.is_deprecated(), "`{orphaned}`");
    assert!(held.is_registered(), "`{orphaned}`");
    assert_eq!(registry::region_preferred(orphaned), None, "`{orphaned}`");
  }
}

/// **A STRUCTURAL VIOLATION IS REFUSED**, and the refusal names which of the two grammars was
/// nearly satisfied.
///
/// `DEU` is the row that earns the separate variants: it is a LANGUAGE's shape, and telling a
/// client that a region written in letters is exactly two is what stops it wondering whether the
/// problem was the letters.
#[test]
fn a_structural_violation_is_refused_by_the_arm_it_was_nearest() {
  for (sent, expected) in [
    ("", ParseRegionError::Empty),
    ("D", ParseRegionError::WrongLetterWidth),
    ("DEU", ParseRegionError::WrongLetterWidth),
    ("GERM", ParseRegionError::WrongLetterWidth),
    ("1", ParseRegionError::WrongDigitWidth),
    ("41", ParseRegionError::WrongDigitWidth),
    ("4190", ParseRegionError::WrongDigitWidth),
    ("D1", ParseRegionError::Mixed),
    ("4A", ParseRegionError::Mixed),
    ("419A", ParseRegionError::Mixed),
    ("D_", ParseRegionError::NotAlphanumeric('_')),
    ("de-DE", ParseRegionError::NotAlphanumeric('-')),
    ("日本", ParseRegionError::NotAlphanumeric('日')),
    (" DE", ParseRegionError::NotAlphanumeric(' ')),
  ] {
    assert_eq!(door(sent), Err(expected), "`{sent}`");
  }
}

/// **`ZZ` AND `AA` ARE REGISTERED AND PRIVATE AT ONCE**, which no other subtag in this family
/// manages.
///
/// The registry gives a region four private-use spellings and only two are ranges. So the predicate
/// pair that is mutually exclusive on a language and a script is not on a region, and a range test
/// alone would answer `false` to the two a container actually writes.
#[test]
fn the_two_individually_registered_private_regions_are_both_at_once() {
  for private in ["AA", "ZZ"] {
    let held = admitted(private);

    assert!(held.is_private_use(), "`{private}`");
    assert!(held.is_registered(), "`{private}` has a record of its own");
    assert_eq!(held.name(), Some("Private use"), "`{private}`");
  }

  assert_eq!(registry::REGION_PRIVATE_USE_SUBTAGS, ["AA", "ZZ"]);

  // The RANGES are the other two, and their members are private WITHOUT being registered — which is
  // the shape a language and a script have for all of theirs.
  for ranged in ["QM", "QZ", "XA", "XZ", "XK"] {
    let held = admitted(ranged);
    assert!(held.is_private_use(), "`{ranged}`");
    assert!(!held.is_registered(), "`{ranged}`");
  }
}

/// **`ZZ` IS A VALUE**, and it is not the absence of a region.
#[test]
fn zz_is_a_value_and_not_an_absence() {
  let unknown = Region::ZZ;

  assert_eq!(unknown.as_str(), "ZZ");
  assert!(unknown.is_zz());
  assert_eq!(admitted("zz"), unknown);
  assert!(!admitted("DE").is_zz());

  let absent: Option<Region> = None;
  assert_ne!(absent, Some(unknown), "no region is not `ZZ`");
}

/// The AREA reading tells the two grammars apart, which a caller aggregating by country has to be
/// able to do.
#[test]
fn an_area_code_says_that_it_is_one() {
  assert!(admitted("419").is_area());
  assert!(admitted("001").is_area());
  assert!(!admitted("DE").is_area());
  assert!(!admitted("ZZ").is_area());

  assert_eq!(admitted("150").name(), Some("Europe"));
  assert_eq!(
    admitted("419").name(),
    Some("Latin America and the Caribbean")
  );
}

/// **An UNREGISTERED subtag is admitted**, on both arms.
#[test]
fn an_unregistered_subtag_is_admitted_on_either_grammar() {
  let letters = admitted("ZY");
  assert_eq!(letters.as_str(), "ZY");
  assert!(!letters.is_registered());
  assert!(!letters.is_private_use());

  let digits = admitted("999");
  assert_eq!(digits.as_str(), "999");
  assert!(!digits.is_registered());
  assert!(digits.is_area());
}

/// **EVERY REGISTERED SUBTAG SURVIVES ITS OWN DOOR**, walked whole, and the fold collides nothing.
///
/// The six the registry supersedes are the exceptions to the fixpoint, and they are exactly the
/// `Preferred-Value` rows.
#[test]
fn a_canonical_region_is_a_fixpoint_and_the_fold_collides_nothing() {
  let mut reached: BTreeMap<String, &str> = BTreeMap::new();

  for (subtag, _) in registry::table::REGIONS {
    let held = admitted(subtag);

    match registry::region_preferred(subtag) {
      Some(preferred) => assert_eq!(held.as_str(), preferred, "`{subtag}`"),
      None => {
        assert_eq!(held.as_str(), *subtag, "`{subtag}` is not a fixpoint");
        assert_eq!(
          reached.insert(String::from(held.as_str()), subtag),
          None,
          "two regions folded onto `{}`",
          held.as_str()
        );
      }
    }

    assert_eq!(
      admitted(held.as_str()),
      held,
      "`{subtag}` is not idempotent"
    );
  }

  assert_eq!(reached.len(), registry::REGION_COUNT - 6);
}

/// The rendering and the parse are inverse, and the [`Debug`](core::fmt::Debug) face prints the
/// subtag.
#[test]
fn the_rendering_and_the_parse_are_inverse() {
  for sent in ["DE", "bu", "419", "001", "ZZ", "ZY"] {
    let held = admitted(sent);
    let rendered = held.to_string();

    assert_eq!(Region::from_str(&rendered).expect("canonical text"), held);
    assert_eq!(rendered, held.as_str());
  }

  assert_eq!(std::format!("{:?}", admitted("bu")), r#"Region("MM")"#);
}

/// Ordering is alphabetical over the canonical spelling, so the two grammars interleave by their
/// BYTES — digits before letters. A sort key, and nothing more, exactly as on the siblings.
#[test]
fn ordering_is_bytewise_over_the_canonical_spelling() {
  let mut sorted = [
    admitted("ZZ"),
    admitted("de"),
    admitted("419"),
    admitted("001"),
  ];
  sorted.sort();

  let spellings: Vec<&str> = sorted.iter().map(Region::as_str).collect();
  assert_eq!(spellings, ["001", "419", "DE", "ZZ"], "digits sort first");
}
