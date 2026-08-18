use super::*;
use ::std::vec;

#[test]
fn try_new_happy_path() {
  let art = CoverArt::try_new("image/jpeg", vec![0xFFu8, 0xD8, 0xFF]).unwrap();
  assert_eq!(art.mime(), "image/jpeg");
  assert_eq!(art.data(), &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn try_new_rejects_empty_mime() {
  let err = CoverArt::try_new("", vec![1u8, 2, 3]).unwrap_err();
  assert_eq!(err, CoverArtError::EmptyMime);
}

#[test]
fn try_new_rejects_empty_data() {
  let err = CoverArt::try_new("image/png", vec![]).unwrap_err();
  assert_eq!(err, CoverArtError::EmptyData);
}
