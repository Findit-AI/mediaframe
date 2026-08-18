//! Walker spec for the `Gray32` source format (FFmpeg `gray32{le,be}`).
//!
//! Single `u32` luma plane, all 32 bits active. No chroma. The full-bit
//! integer twin of [`super::Gray16`]; widened `u16` → `u32`.
//!
//! The marker carries `<const BE: bool = false>`: `Gray32` (= `Gray32<false>`)
//! is the LE source; `Gray32<true>` is the BE source. Two walker entry points
//! are emitted: [`gray32_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `gray32_to::<S>`; [`gray32_to_endian::<S, BE>`]
//! is the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Gray32Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Gray32Frame;

walker! {
  planar1_be {
    /// Marker type for the `Gray32` source format (32-bit integer u32).
    /// `<const BE: bool>` defaults to `false` (LE).
    marker: Gray32,
    frame: Gray32Frame,
    row: Gray32Row,
    sink: Gray32Sink,
    walker: gray32_to,
    walker_endian: gray32_to_endian,
    elem_type: u32,
    row_doc: "A single row from a [`Gray32Frame`](crate::frame::Gray32Frame).",
    walker_doc: "Walks a [`Gray32Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Gray32Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
