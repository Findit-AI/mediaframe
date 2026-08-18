use super::*;
use ::std::vec;

#[test]
fn try_new_happy_path() {
  let fp = Fingerprint::try_new("chromaprint", vec![1u8, 2, 3, 4]).unwrap();
  assert_eq!(fp.algorithm(), "chromaprint");
  assert_eq!(fp.value(), &[1, 2, 3, 4]);
}

#[test]
fn try_new_rejects_empty_algorithm() {
  let err = Fingerprint::try_new("", vec![1u8]).unwrap_err();
  assert_eq!(err, FingerprintError::EmptyAlgorithm);
}

#[test]
fn try_new_accepts_empty_value() {
  let fp = Fingerprint::try_new("acoustid", vec![]).unwrap();
  assert_eq!(fp.algorithm(), "acoustid");
  assert!(fp.value().is_empty());
}
