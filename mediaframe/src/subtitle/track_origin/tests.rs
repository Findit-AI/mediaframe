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

  // Case folds, on both the lookup and the escape.
  assert_eq!("Embedded".parse(), Ok(TrackOrigin::Embedded));
  let o: TrackOrigin = "BroadCast".parse().unwrap();
  assert_eq!(o, TrackOrigin::other("broadcast"));
  assert_eq!(o.as_str(), "broadcast");
  assert_eq!("broadcast".parse::<TrackOrigin>().unwrap(), o);
}

/// The parse is total at this tier: every slug — including the empty
/// one — lands on a value rather than a refusal.
#[test]
fn parse_is_total_and_empty_slug_is_the_escape() {
  assert_eq!("".parse::<TrackOrigin>().unwrap(), TrackOrigin::other(""));
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
