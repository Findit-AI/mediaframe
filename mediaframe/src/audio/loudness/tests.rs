use super::*;

#[test]
fn new_holds_supplied_fields() {
  let l = Loudness::new(-23.0, 7.5, -1.2, -3.4);
  assert_eq!(l.integrated_lufs(), -23.0);
  assert_eq!(l.range_lu(), 7.5);
  assert_eq!(l.true_peak_dbtp(), -1.2);
  assert_eq!(l.sample_peak_dbfs(), -3.4);
}

#[test]
fn default_is_all_zero() {
  let l = Loudness::default();
  assert_eq!(l.integrated_lufs(), 0.0);
  assert_eq!(l.range_lu(), 0.0);
  assert_eq!(l.true_peak_dbtp(), 0.0);
  assert_eq!(l.sample_peak_dbfs(), 0.0);
}

#[test]
fn with_chain_builds_full_value() {
  let l = Loudness::default()
    .with_integrated_lufs(-23.0)
    .with_range_lu(7.5)
    .with_true_peak_dbtp(-1.2)
    .with_sample_peak_dbfs(-3.4);
  assert_eq!(l, Loudness::new(-23.0, 7.5, -1.2, -3.4));
}

#[test]
fn setters_mutate_in_place() {
  let mut l = Loudness::default();
  l.set_integrated_lufs(-16.0)
    .set_range_lu(5.0)
    .set_true_peak_dbtp(-0.5)
    .set_sample_peak_dbfs(-1.0);
  assert_eq!(l, Loudness::new(-16.0, 5.0, -0.5, -1.0));
}
