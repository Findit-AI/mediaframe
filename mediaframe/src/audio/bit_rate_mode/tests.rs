use super::*;
use ::std::string::ToString;

#[test]
fn default_is_cbr() {
  assert_eq!(BitRateMode::default(), BitRateMode::Cbr);
}

#[test]
fn as_str_round_trips() {
  assert_eq!(BitRateMode::Cbr.as_str(), "cbr");
  assert_eq!(BitRateMode::Vbr.as_str(), "vbr");
  assert_eq!(BitRateMode::Abr.as_str(), "abr");
}

#[test]
fn u32_round_trip_named_variants() {
  for v in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
    assert_eq!(BitRateMode::from_u32(v.to_u32()), v);
  }
}

#[test]
fn display_matches_as_str() {
  assert_eq!(BitRateMode::Cbr.to_string(), "cbr");
  assert_eq!(BitRateMode::Vbr.to_string(), "vbr");
  assert_eq!(BitRateMode::Abr.to_string(), "abr");
}

#[test]
fn is_variant_predicates() {
  assert!(BitRateMode::Cbr.is_cbr());
  assert!(BitRateMode::Vbr.is_vbr());
  assert!(BitRateMode::Abr.is_abr());
}

/// `BitRateMode` is a closed unit vocabulary — the slug round-trips for
/// every variant and nothing else parses.
#[test]
fn every_mode_round_trips_through_its_slug() {
  const fn _is_exhaustive(m: BitRateMode) {
    match m {
      BitRateMode::Cbr | BitRateMode::Vbr | BitRateMode::Abr => (),
    }
  }

  for mode in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
    assert_eq!(mode.as_str().parse(), Ok(mode));
  }

  let err: ParseBitRateModeError = "constant".parse::<BitRateMode>().unwrap_err();
  let _ = err;
  // Case folds — the vocabulary is one name per value, not one spelling.
  assert_eq!("CBR".parse(), Ok(BitRateMode::Cbr));
  assert_eq!("Vbr".parse(), Ok(BitRateMode::Vbr));
  assert!("".parse::<BitRateMode>().is_err());
}
