//! Walker spec for the `Grayf16` source format (FFmpeg `grayf16{le,be}`).
//!
//! Single `half::f16` luma plane. Nominal range `[0.0, 1.0]`; HDR > 1.0 is
//! permitted. Stride is in f16 elements. No chroma planes exist. The
//! half-float twin of [`super::Grayf32`].
//!
//! The marker carries `<const BE: bool = false>`: `Grayf16` (= `Grayf16<false>`)
//! is the LE source; `Grayf16<true>` is the BE source. Two walker entry points
//! are emitted: [`grayf16_to`] is an LE-only compatibility wrapper preserving
//! the single-generic signature `grayf16_to::<S>`;
//! [`grayf16_to_endian::<S, BE>`] is the const-generic entry point for
//! BE-aware callers, propagating `BE` from [`Grayf16Frame<'_, BE>`] into the
//! sinker dispatch. The kernel reinterprets each `f16` via byte-swapped `u16`
//! bits when `BE = true`.

use crate::frame::Grayf16Frame;

walker! {
  planar1_be {
    /// Marker type for the `Grayf16` source format (16-bit half-float luma).
    ///
    /// Nominal luma range `[0.0, 1.0]`; HDR values > 1.0 are permitted.
    /// Out-of-range values are clamped during output conversion, not at frame
    /// construction time. `<const BE: bool>` defaults to `false` (LE).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Grayf16,
    frame: Grayf16Frame,
    row: Grayf16Row,
    sink: Grayf16Sink,
    walker: grayf16_to,
    walker_endian: grayf16_to_endian,
    elem_type: half::f16,
    row_doc: "A single row from a [`Grayf16Frame`](crate::frame::Grayf16Frame) — `width` f16 luma samples.",
    walker_doc: "Walks a [`Grayf16Frame<'_, BE>`] row by row, dispatching each \
                 row to the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Grayf16Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
