//! Walker spec for `Gray9` (FFmpeg `gray9{le,be}`).
//!
//! The marker carries `<const BE: bool = false>`: `Gray9` (= `Gray9<false>`)
//! is the LE source; `Gray9<true>` is the BE source. Two walker entry points
//! are emitted: [`gray9_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `gray9_to::<S>`; [`gray9_to_endian::<S, BE>`]
//! is the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Gray9Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::{Gray9Frame, GrayNFrame};

walker! {
  planar1_bits_be {
    /// Marker type for the `Gray9` source format (9-bit low-packed u16).
    /// `<const BE: bool>` defaults to `false` (LE).
    marker: Gray9,
    frame: Gray9Frame,
    generic_frame: GrayNFrame,
    bits: 9,
    row: Gray9Row,
    sink: Gray9Sink,
    walker: gray9_to,
    walker_endian: gray9_to_endian,
    walker_inner: gray9_to_inner,
    elem_type: u16,
    row_doc: "A single row from a [`Gray9Frame`](crate::frame::Gray9Frame).",
    walker_doc: "Walks a [`Gray9Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Gray9Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
