//! Walker spec for the `Grayf32` source format (FFmpeg `grayf32{le,be}`).
//!
//! Single `f32` luma plane. Nominal range `[0.0, 1.0]`; HDR > 1.0 is permitted.
//! Stride is in f32 elements. No chroma planes exist.
//!
//! The marker carries `<const BE: bool = false>`: `Grayf32` (= `Grayf32<false>`)
//! is the LE source; `Grayf32<true>` is the BE source. Two walker entry points
//! are emitted: [`grayf32_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `grayf32_to::<S>`;
//! [`grayf32_to_endian::<S, BE>`] is the const-generic entry point for
//! BE-aware callers, propagating `BE` from [`Grayf32Frame<'_, BE>`] into the
//! sinker dispatch. The kernel reinterprets each `f32` via byte-swapped `u32`
//! bits when `BE = true`.

use crate::frame::Grayf32Frame;

walker! {
  planar1_be {
    /// Marker type for the `Grayf32` source format (32-bit float luma).
    ///
    /// Nominal luma range `[0.0, 1.0]`; HDR values > 1.0 are permitted.
    /// Out-of-range values are clamped during output conversion, not at frame
    /// construction time. `<const BE: bool>` defaults to `false` (LE).
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    marker: Grayf32,
    frame: Grayf32Frame,
    row: Grayf32Row,
    sink: Grayf32Sink,
    walker: grayf32_to,
    walker_endian: grayf32_to_endian,
    elem_type: f32,
    row_doc: "A single row from a [`Grayf32Frame`](crate::frame::Grayf32Frame) — `width` f32 luma samples.",
    walker_doc: "Walks a [`Grayf32Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Grayf32Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
