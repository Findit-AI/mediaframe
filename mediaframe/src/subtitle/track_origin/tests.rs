use super::*;
use ::std::string::ToString;

const ALL: &[TrackOrigin] = &[
  TrackOrigin::Embedded,
  TrackOrigin::Sidecar,
  TrackOrigin::External,
];

#[test]
fn round_trip_via_u32_for_every_variant() {
  for &o in ALL {
    assert_eq!(TrackOrigin::from_u32(o.to_u32()), o);
  }
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
}

#[test]
fn display_matches_as_str() {
  for &o in ALL {
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
}

/// `TrackOrigin` is a closed unit vocabulary — the slug round-trips for
/// every variant and nothing else parses. The `match` makes the list
/// exhaustive by construction.
#[test]
fn every_origin_round_trips_through_its_slug() {
  const fn _is_exhaustive(o: TrackOrigin) {
    match o {
      TrackOrigin::Embedded | TrackOrigin::Sidecar | TrackOrigin::External => (),
    }
  }

  for origin in ALL {
    assert_eq!(origin.as_str().parse(), Ok(*origin));
  }

  let err: ParseTrackOriginError = "broadcast".parse::<TrackOrigin>().unwrap_err();
  let _ = err;
  // Case folds.
  assert_eq!("Embedded".parse(), Ok(TrackOrigin::Embedded));
  assert!("".parse::<TrackOrigin>().is_err());
}
