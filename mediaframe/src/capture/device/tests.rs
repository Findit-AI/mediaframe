use super::*;

#[test]
fn new_is_all_empty() {
  let d = Device::new();
  assert_eq!(d.make(), "");
  assert_eq!(d.model(), "");
  assert!(d.is_empty());
}

#[test]
fn default_matches_new() {
  assert_eq!(Device::default(), Device::new());
}

#[test]
fn builder_chain_populates() {
  let d = Device::new().with_make("Apple").with_model("iPhone 15 Pro");
  assert_eq!(d.make(), "Apple");
  assert_eq!(d.model(), "iPhone 15 Pro");
  assert!(!d.is_empty());
}

#[test]
fn setters_mutate_in_place() {
  let mut d = Device::new();
  d.set_make("Sony");
  d.set_model("ILCE-7M4");
  assert_eq!(d.make(), "Sony");
  assert_eq!(d.model(), "ILCE-7M4");
  assert!(!d.is_empty());
}

#[test]
fn is_empty_partial() {
  let m = Device::new().with_make("Apple");
  assert!(!m.is_empty());
  let n = Device::new().with_model("ILCE-7M4");
  assert!(!n.is_empty());
}
