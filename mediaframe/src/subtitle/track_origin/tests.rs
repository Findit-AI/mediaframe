use super::*;
use ::std::string::ToString;

const ALL: &[TrackOrigin] = &[
  TrackOrigin::Embedded,
  TrackOrigin::Sidecar,
  TrackOrigin::External,
  TrackOrigin::Derived,
];

#[test]
fn round_trip_via_u32_for_every_named_variant() {
  for o in ALL {
    let code = o.to_u32().expect("named variants all carry an id");
    assert_eq!(&TrackOrigin::from_u32(code), o);
  }
}

#[test]
fn escape_has_no_numeric_id() {
  assert_eq!(TrackOrigin::other("broadcast").to_u32(), None);
}

#[test]
fn from_u32_unknown_falls_back_to_default() {
  assert_eq!(TrackOrigin::from_u32(999), TrackOrigin::Embedded,);
}

#[test]
fn as_str_matches_spec() {
  assert_eq!(TrackOrigin::Embedded.as_str(), "embedded");
  assert_eq!(TrackOrigin::Sidecar.as_str(), "sidecar");
  assert_eq!(TrackOrigin::External.as_str(), "external");
  assert_eq!(TrackOrigin::Derived.as_str(), "derived");
}

#[test]
fn display_matches_as_str() {
  for o in ALL {
    assert_eq!(o.to_string(), o.as_str());
  }
}

#[test]
fn default_is_embedded() {
  assert_eq!(TrackOrigin::default(), TrackOrigin::Embedded);
}

#[test]
fn is_variant_predicates() {
  assert!(TrackOrigin::Embedded.is_embedded());
  assert!(!TrackOrigin::Embedded.is_sidecar());
  assert!(TrackOrigin::Sidecar.is_sidecar());
  assert!(TrackOrigin::External.is_external());
  assert!(TrackOrigin::Derived.is_derived());
}

/// The slug round-trips for every named variant, and an unnamed slug
/// rides the escape instead of being refused.
#[test]
fn every_origin_round_trips_through_its_slug() {
  for origin in ALL {
    assert_eq!(origin.as_str().parse::<TrackOrigin>().as_ref(), Ok(origin));
  }

  // The lookup folds — a named meaning is one value whatever case it
  // arrives in — but a genuine stranger keeps its own spelling verbatim
  // through the escape.
  assert_eq!("Embedded".parse(), Ok(TrackOrigin::Embedded));
  assert_eq!(TrackOrigin::other("Embedded"), TrackOrigin::Embedded);
  let o: TrackOrigin = "BroadCast".parse().unwrap();
  assert_eq!(o, TrackOrigin::other("BroadCast"));
  assert_eq!(o.as_str(), "BroadCast");
  assert_ne!("broadcast".parse::<TrackOrigin>().unwrap(), o);
}

/// **Type-level** totality, the same proof the nine all-tier
/// vocabularies carry: each `let Ok(..) = ..` is an *irrefutable*
/// pattern, which it can only be because `FromStr::Err` is uninhabited.
/// Give `TrackOrigin` a refusing `Err` again and this stops compiling
/// (`E0005`) rather than merely going untested.
///
/// `TrackOrigin` needs no `cfg` on the proof: its module exists only at
/// the `alloc` tier, so unlike the nine there is no lean build in which
/// the vocabulary closes.
#[test]
fn parse_is_total_at_every_tier_this_type_exists_at() {
  // The empty slug is a value, not a refusal.
  let Ok(empty) = "".parse::<TrackOrigin>();
  assert_eq!(empty, TrackOrigin::other(""));

  // So is a word from nobody's vocabulary.
  let Ok(unknown) = "not-a-track-origin".parse::<TrackOrigin>();
  assert_eq!(unknown, TrackOrigin::other("not-a-track-origin"));

  // And a named one still lands on its variant.
  let Ok(named) = "derived".parse::<TrackOrigin>();
  assert_eq!(named, TrackOrigin::Derived);
}

/// The `Other` arm is reachable through the borrowed unwrap views, the
/// same surface `subtitle::Format` carries.
#[test]
fn unwrap_other_borrowed_view() {
  let o = TrackOrigin::other("broadcast");
  assert_eq!(o.unwrap_other_ref().as_str(), "broadcast");
  assert!(o.try_unwrap_other_ref().is_ok());
  assert!(TrackOrigin::Embedded.try_unwrap_other_ref().is_err());
}

/// The runtime half of the `ROSTER` contract for `TrackOrigin` — no duplicate
/// entry, no two entries sharing a slug, and `as_str` → `FromStr` the
/// identity on every named variant. Completeness is the compile-time
/// half: the witness beside each declaration is `E0004` the moment a
/// variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(TrackOrigin::ROSTER, "TrackOrigin", TrackOrigin::as_str);
}
