use super::*;

#[test]
fn new_holds_supplied_fields() {
  let g = ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99));
  assert_eq!(g.track_gain_db(), -6.4);
  assert_eq!(g.track_peak(), 0.97);
  assert_eq!(g.album_gain_db(), Some(-7.1));
  assert_eq!(g.album_peak(), Some(0.99));
}

#[test]
fn default_is_zero_track_none_album() {
  let g = ReplayGain::default();
  assert_eq!(g.track_gain_db(), 0.0);
  assert_eq!(g.track_peak(), 0.0);
  assert_eq!(g.album_gain_db(), None);
  assert_eq!(g.album_peak(), None);
}

#[test]
fn with_chain_builds_full_value() {
  let g = ReplayGain::default()
    .with_track_gain_db(-6.4)
    .with_track_peak(0.97)
    .with_album_gain_db(Some(-7.1))
    .with_album_peak(Some(0.99));
  assert_eq!(g, ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99)));
}

#[test]
fn setters_mutate_in_place() {
  let mut g = ReplayGain::default();
  g.set_track_gain_db(-6.4)
    .set_track_peak(0.97)
    .set_album_gain_db(Some(-7.1))
    .set_album_peak(Some(0.99));
  assert_eq!(g, ReplayGain::new(-6.4, 0.97, Some(-7.1), Some(0.99)));
}

#[test]
fn album_fields_are_independent() {
  // Track present, album missing — the common case for
  // single-track distribution.
  let g = ReplayGain::default()
    .with_track_gain_db(-6.4)
    .with_track_peak(0.97);
  assert_eq!(g.album_gain_db(), None);
  assert_eq!(g.album_peak(), None);
}
