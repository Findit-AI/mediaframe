//! The `#[doc(hidden)]` row door, kept honest.
//!
//! `Xyz12Row::new` is `pub(crate)`, so out-of-tree kernel-parity suites
//! reach a row only through `for_tests`. Nothing in this crate needs the
//! door — every in-tree row comes off the walker — so without this file
//! it would ship unproven.
//!
//! Its own file rather than `tests.rs` for the reason `color` keeps
//! `kernel_tests` apart: the door is a surface with its own contract
//! (what it takes, what it promises, what it does *not* promise), not
//! another fact about the walk.

use super::*;
use crate::{PixelSink, frame::Xyz12Frame};
use core::convert::Infallible;

/// Everything a row will say about itself, read back through the public
/// accessors — the whole of what the door has to get right.
#[derive(Debug, PartialEq, Eq)]
struct Seen {
  row: usize,
  gamut: KernelGamut,
  luma_q15: (i32, i32, i32),
  big_endian: bool,
  xyz_len: usize,
}

/// Captures the one row it is handed. Generic over `BE` so the same
/// sink can receive a row from either endianness of the format.
struct OneRowSink<const BE: bool> {
  seen: Option<Seen>,
}

impl<const BE: bool> OneRowSink<BE> {
  const fn empty() -> Self {
    Self { seen: None }
  }
}

impl<const BE: bool> PixelSink for OneRowSink<BE> {
  type Input<'r> = Xyz12Row<'r, BE>;
  type Error = Infallible;

  fn process(&mut self, row: Xyz12Row<'_, BE>) -> Result<(), Infallible> {
    self.seen = Some(Seen {
      row: row.row(),
      gamut: row.target_gamut(),
      luma_q15: row.luma_q15(),
      big_endian: row.big_endian(),
      xyz_len: row.xyz().len(),
    });
    Ok(())
  }
}

impl<const BE: bool> Xyz12Sink<BE> for OneRowSink<BE> {}

/// The door builds a row with no frame and no walker in sight, which is
/// the one thing `pub(crate) new` took away and the only reason the door
/// exists.
#[test]
fn the_hidden_door_builds_a_row_without_a_frame() {
  let xyz = [1u16, 2, 3, 4, 5, 6];
  let luma = luma_weights_q15_for_gamut(KernelGamut::Rec2020);

  let mut sink = OneRowSink::<false>::empty();
  sink
    .process(Xyz12Row::for_tests(&xyz, 7, KernelGamut::Rec2020, luma))
    .unwrap();

  assert_eq!(
    sink.seen,
    Some(Seen {
      row: 7,
      gamut: KernelGamut::Rec2020,
      luma_q15: luma,
      big_endian: false,
      xyz_len: 6,
    })
  );
}

/// The row type is the crate's only one with a const generic, and the
/// door has to carry it: a `Xyz12Row::<true>` built through the door
/// must still report big-endian, or a parity suite would drive the LE
/// kernel against BE samples and see the byte-swap branch never taken.
#[test]
fn the_door_carries_the_endianness_of_its_type() {
  let xyz = [0u16; 3];
  let luma = luma_weights_q15_for_gamut(KernelGamut::DciP3);

  let mut sink = OneRowSink::<true>::empty();
  sink
    .process(Xyz12Row::<true>::for_tests(
      &xyz,
      0,
      KernelGamut::DciP3,
      luma,
    ))
    .unwrap();

  assert!(
    sink.seen.expect("one row").big_endian,
    "the BE row marker must survive the door"
  );
}

/// The door takes exactly what `new` takes, in the same order — the
/// property that lets a parity suite trust it. Proven against the
/// walker rather than against `new` directly: forwarding to `new` is
/// visible in the source, but that the four seats mean what the walk
/// puts in them is not.
#[test]
fn the_door_reproduces_the_row_the_walker_would_have_handed_over() {
  // One row of one pixel: `width * 3` samples, stride in elements.
  let plane = [11u16, 22, 33];
  let frame = Xyz12Frame::new(&plane, 1, 1, 3);

  let mut walked = OneRowSink::<false>::empty();
  xyz12_to(&frame, &mut walked).unwrap();

  // `OneRowSink` leaves `target_gamut` at its default, so the walk above
  // asked, got the stated DCI-P3, and stamped it with the luma weights
  // derived from it. The door is handed that same pair by hand.
  let mut through_the_door = OneRowSink::<false>::empty();
  through_the_door
    .process(Xyz12Row::for_tests(
      &plane,
      0,
      KernelGamut::DciP3,
      luma_weights_q15_for_gamut(KernelGamut::DciP3),
    ))
    .unwrap();

  assert_eq!(through_the_door.seen, walked.seen);
}
