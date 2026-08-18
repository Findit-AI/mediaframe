use super::*;

#[test]
fn try_new_happy() {
  let g = GeoLocation::try_new(48.8566, 2.3522, None).unwrap();
  assert!((g.lat() - 48.8566).abs() < 1e-9);
  assert!((g.lon() - 2.3522).abs() < 1e-9);
  assert!(g.altitude().is_none());
}

#[test]
fn try_new_with_altitude() {
  let g = GeoLocation::try_new(0.0, 0.0, Some(35.0)).unwrap();
  assert_eq!(g.altitude(), Some(35.0));
}

#[test]
fn try_new_rejects_lat_out_of_range() {
  let err = GeoLocation::try_new(91.0, 0.0, None).unwrap_err();
  assert!(matches!(err, GeoLocationError::LatOutOfRange(_)));
  let err = GeoLocation::try_new(-91.0, 0.0, None).unwrap_err();
  assert!(matches!(err, GeoLocationError::LatOutOfRange(_)));
}

#[test]
fn try_new_rejects_lon_out_of_range() {
  let err = GeoLocation::try_new(0.0, 181.0, None).unwrap_err();
  assert!(matches!(err, GeoLocationError::LonOutOfRange(_)));
  let err = GeoLocation::try_new(0.0, -181.0, None).unwrap_err();
  assert!(matches!(err, GeoLocationError::LonOutOfRange(_)));
}

#[test]
fn try_new_rejects_nan_and_inf() {
  assert!(matches!(
    GeoLocation::try_new(f64::NAN, 0.0, None),
    Err(GeoLocationError::LatOutOfRange(_))
  ));
  assert!(matches!(
    GeoLocation::try_new(0.0, f64::INFINITY, None),
    Err(GeoLocationError::LonOutOfRange(_))
  ));
}

#[test]
fn null_island_round_trips() {
  let g = GeoLocation::from_iso6709("+00.0000+000.0000/").unwrap();
  assert_eq!(g.lat(), 0.0);
  assert_eq!(g.lon(), 0.0);
  assert!(g.altitude().is_none());
  assert_eq!(g.to_iso6709(), "+00.0000+000.0000/");
}

#[test]
fn paris_round_trips() {
  let g = GeoLocation::from_iso6709("+48.8566+002.3522/").unwrap();
  assert!((g.lat() - 48.8566).abs() < 1e-6);
  assert!((g.lon() - 2.3522).abs() < 1e-6);
  assert_eq!(g.to_iso6709(), "+48.8566+002.3522/");
}

#[test]
fn paris_with_altitude_round_trips() {
  let g = GeoLocation::from_iso6709("+48.8566+002.3522+035/").unwrap();
  assert!((g.lat() - 48.8566).abs() < 1e-6);
  assert!((g.lon() - 2.3522).abs() < 1e-6);
  assert_eq!(g.altitude(), Some(35.0));
  assert_eq!(g.to_iso6709(), "+48.8566+002.3522+35/");
}

#[test]
fn sao_paulo_round_trips() {
  // São Paulo: negative lat, negative lon, +760 m altitude.
  let g = GeoLocation::from_iso6709("-23.5505-046.6333+760/").unwrap();
  assert!((g.lat() - -23.5505).abs() < 1e-6);
  assert!((g.lon() - -46.6333).abs() < 1e-6);
  assert_eq!(g.altitude(), Some(760.0));
  assert_eq!(g.to_iso6709(), "-23.5505-046.6333+760/");
}

#[test]
fn sydney_negative_lat_positive_lon() {
  // Sydney: -33.8688, +151.2093, no altitude.
  let g = GeoLocation::from_iso6709("-33.8688+151.2093/").unwrap();
  assert!((g.lat() - -33.8688).abs() < 1e-6);
  assert!((g.lon() - 151.2093).abs() < 1e-6);
  assert!(g.altitude().is_none());
  assert_eq!(g.to_iso6709(), "-33.8688+151.2093/");
}

#[test]
fn from_str_smoke() {
  let g: GeoLocation = "+48.8566+002.3522/".parse().unwrap();
  assert!((g.lat() - 48.8566).abs() < 1e-6);
}

#[test]
fn display_smoke() {
  let g = GeoLocation::try_new(0.0, 0.0, None).unwrap();
  let rendered = std::format!("{}", g);
  assert_eq!(rendered, "+00.0000+000.0000/");
}

#[test]
fn iso6709_rejects_missing_slash() {
  assert!(matches!(
    GeoLocation::from_iso6709("+48.8566+002.3522"),
    Err(GeoLocationError::Iso6709Malformed(_))
  ));
}

#[test]
fn iso6709_rejects_missing_sign() {
  assert!(matches!(
    GeoLocation::from_iso6709("48.8566+002.3522/"),
    Err(GeoLocationError::Iso6709Malformed(_))
  ));
}

#[test]
fn iso6709_rejects_garbage() {
  assert!(matches!(
    GeoLocation::from_iso6709("not a location"),
    Err(GeoLocationError::Iso6709Malformed(_))
  ));
  assert!(matches!(
    GeoLocation::from_iso6709("+99.0000+000.0000/"),
    Err(GeoLocationError::LatOutOfRange(_))
  ));
  assert!(matches!(
    GeoLocation::from_iso6709("+00.0000+999.0000/"),
    Err(GeoLocationError::LonOutOfRange(_))
  ));
}

#[test]
fn iso6709_rejects_wrong_int_digit_count() {
  // Lat must be 2 integer digits.
  assert!(matches!(
    GeoLocation::from_iso6709("+8.8566+002.3522/"),
    Err(GeoLocationError::Iso6709Malformed(_))
  ));
  // Lon must be 3 integer digits.
  assert!(matches!(
    GeoLocation::from_iso6709("+48.8566+02.3522/"),
    Err(GeoLocationError::Iso6709Malformed(_))
  ));
}

#[test]
fn with_altitude_builder() {
  let g = GeoLocation::try_new(0.0, 0.0, None)
    .unwrap()
    .with_altitude(120.0);
  assert_eq!(g.altitude(), Some(120.0));
}

#[test]
fn maybe_altitude_assigns_raw_wrapper() {
  let g = GeoLocation::try_new(0.0, 0.0, None)
    .unwrap()
    .maybe_altitude(Some(80.0));
  assert_eq!(g.altitude(), Some(80.0));
  let g = g.maybe_altitude(None);
  assert!(g.altitude().is_none());
}

#[test]
fn set_altitude_mutates_in_place() {
  let mut g = GeoLocation::try_new(0.0, 0.0, None).unwrap();
  g.set_altitude(50.5);
  assert_eq!(g.altitude(), Some(50.5));
  g.update_altitude(Some(60.0));
  assert_eq!(g.altitude(), Some(60.0));
  g.clear_altitude();
  assert!(g.altitude().is_none());
}

#[test]
fn non_finite_altitude_normalises_to_none() {
  // Every altitude entry point collapses NaN / ±inf to `None` so the
  // field invariant ("None or finite") holds and `to_iso6709` never
  // emits a NaN->0 cast artifact.
  for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
    assert!(
      GeoLocation::try_new(48.0, 2.0, Some(bad))
        .unwrap()
        .altitude()
        .is_none(),
      "try_new should normalise non-finite altitude to None"
    );
    let mut g = GeoLocation::try_new(48.0, 2.0, Some(10.0)).unwrap();
    g.set_altitude(bad);
    assert!(
      g.altitude().is_none(),
      "set_altitude should normalise non-finite to None"
    );
    g.update_altitude(Some(bad));
    assert!(
      g.altitude().is_none(),
      "update_altitude should normalise non-finite to None"
    );
    assert!(
      GeoLocation::try_new(48.0, 2.0, None)
        .unwrap()
        .with_altitude(bad)
        .altitude()
        .is_none(),
      "with_altitude should normalise non-finite to None"
    );
    assert!(
      GeoLocation::try_new(48.0, 2.0, None)
        .unwrap()
        .maybe_altitude(Some(bad))
        .altitude()
        .is_none(),
      "maybe_altitude should normalise non-finite to None"
    );
  }
  // A non-finite altitude must not survive into ISO-6709 output.
  let g = GeoLocation::try_new(48.8566, 2.3522, Some(f32::NAN)).unwrap();
  assert_eq!(g.to_iso6709(), "+48.8566+002.3522/");
}
