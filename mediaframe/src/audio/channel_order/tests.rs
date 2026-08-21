use super::*;
use ::std::string::ToString;

#[test]
fn default_is_unspecified() {
  assert_eq!(ChannelOrder::default(), ChannelOrder::Unspecified);
}

#[test]
fn slugs() {
  // `FromStr` reads the very table `as_str` writes, so a typo there
  // round-trips happily. This is the independent copy that catches it.
  let table = [
    (ChannelOrder::Unspecified, "unspecified"),
    (ChannelOrder::Native, "native"),
    (ChannelOrder::Custom, "custom"),
    (ChannelOrder::Ambisonic, "ambisonic"),
  ];
  assert_eq!(table.len(), ChannelOrder::ROSTER.len());
  for (order, slug) in table {
    assert_eq!(order.as_str(), slug, "slug mismatch for {order:?}");
  }
}

#[test]
fn display_matches_as_str() {
  assert_eq!(ChannelOrder::Ambisonic.to_string(), "ambisonic");
  for &order in ChannelOrder::ROSTER {
    assert_eq!(
      order.to_string(),
      order.as_str(),
      "display drifted from as_str"
    );
  }
}

#[test]
fn is_variant_predicates() {
  assert!(ChannelOrder::Unspecified.is_unspecified());
  assert!(ChannelOrder::Native.is_native());
  assert!(ChannelOrder::Custom.is_custom());
  assert!(ChannelOrder::Ambisonic.is_ambisonic());
  assert!(!ChannelOrder::Native.is_custom());
}

#[test]
fn repr_matches_to_u32() {
  // The `repr(u32)` discriminants are the wire ids, so the two spellings
  // must agree — `to_u32` is a cast, and this is what pins the cast to
  // the documented numbers.
  assert_eq!(ChannelOrder::Unspecified as u32, 0);
  assert_eq!(ChannelOrder::Native as u32, 1);
  assert_eq!(ChannelOrder::Custom as u32, 2);
  assert_eq!(ChannelOrder::Ambisonic as u32, 3);
  assert_eq!(ChannelOrder::Native.to_u32(), 1);
}

#[test]
fn u32_round_trip_named_variants() {
  for &order in ChannelOrder::ROSTER {
    assert_eq!(ChannelOrder::from_u32(order.to_u32()), order);
  }
}

#[test]
fn from_u32_absorbs_an_unknown_code_and_try_from_u32_refuses_it() {
  // The lenient door canonicalises a corrupt discriminant into the
  // "we were not told" value; the strict door is the one the wire uses,
  // and it refuses rather than inventing.
  assert_eq!(ChannelOrder::from_u32(42), ChannelOrder::Unspecified);
  assert_eq!(ChannelOrder::from_u32(0), ChannelOrder::Unspecified);
  assert_eq!(
    ChannelOrder::try_from_u32(0),
    Some(ChannelOrder::Unspecified)
  );
  assert_eq!(ChannelOrder::try_from_u32(3), Some(ChannelOrder::Ambisonic));
  assert_eq!(ChannelOrder::try_from_u32(4), None);
  assert_eq!(ChannelOrder::try_from_u32(u32::MAX), None);
}

#[test]
fn every_order_round_trips_through_its_slug() {
  const fn _is_exhaustive(o: ChannelOrder) {
    match o {
      ChannelOrder::Unspecified
      | ChannelOrder::Native
      | ChannelOrder::Custom
      | ChannelOrder::Ambisonic => (),
    }
  }

  for &order in ChannelOrder::ROSTER {
    assert_eq!(
      order.as_str().parse(),
      Ok(order),
      "slug round-trip failed for {order:?}"
    );
  }
}

#[test]
fn folds_ascii_case() {
  // Case is the whole of the folding — one name per value, not one
  // spelling.
  assert_eq!("Native".parse(), Ok(ChannelOrder::Native));
  assert_eq!("AMBISONIC".parse(), Ok(ChannelOrder::Ambisonic));
  assert_eq!("UnSpEcIfIeD".parse(), Ok(ChannelOrder::Unspecified));
}

#[test]
fn rejects_what_it_cannot_name() {
  // The numeric door absorbs an unknown code into `Unspecified`; the
  // text door refuses an unknown name rather than inventing one.
  assert!("".parse::<ChannelOrder>().is_err());
  assert!("interleaved".parse::<ChannelOrder>().is_err());
  // Neither whitespace nor a second spelling is an alias.
  assert!("native ".parse::<ChannelOrder>().is_err());
  assert!("channel-order".parse::<ChannelOrder>().is_err());
  let err: ParseChannelOrderError = "interleaved".parse::<ChannelOrder>().unwrap_err();
  assert_eq!(err.to_string(), "not a channel-order name");
}

/// The runtime half of the `ROSTER` contract for `ChannelOrder` — no
/// duplicate entry, no two entries sharing a slug, and `as_str` →
/// `FromStr` the identity on every named variant. Completeness is the
/// compile-time half: the witness the `roster!` macro emits is `E0004`
/// the moment a variant is added without being rostered.
#[test]
fn rosters_are_well_formed() {
  crate::roster_tests::check(ChannelOrder::ROSTER, "ChannelOrder", |o| o.as_str());
}

/// Every live wire code reaches the roster. `ROSTER` is what the text
/// door walks and the generators sample, so a code that decodes to a
/// variant missing from it would be writable and unreadable.
#[test]
fn roster_covers_every_live_wire_code() {
  let mut live = 0;
  for code in 0..=16u32 {
    let Some(order) = ChannelOrder::try_from_u32(code) else {
      continue;
    };
    assert_eq!(
      order.to_u32(),
      code,
      "code {code} decodes to a different id"
    );
    live += 1;
    assert!(
      ChannelOrder::ROSTER.contains(&order),
      "{order:?} (code {code}) is missing from the roster"
    );
  }
  assert_eq!(
    ChannelOrder::ROSTER.len(),
    live,
    "the roster holds an entry no wire code decodes to"
  );
}
