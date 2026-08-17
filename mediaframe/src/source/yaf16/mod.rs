//! Walker spec for the `Yaf16` source format
//! (FFmpeg `yaf16{le,be}` / `AV_PIX_FMT_YAF16{LE,BE}`).
//!
//! Single `half::f16` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel
//! occupies 2 f16 elements; stride is in f16 elements and must be `≥ width × 2`
//! (may include row padding). Alpha is real source α at element slot 1 of every
//! pixel pair. The half-float twin of [`super::Ya16`].
//!
//! The marker carries `<const BE: bool = false>`: `Yaf16` (= `Yaf16<false>`) is
//! the LE source; `Yaf16<true>` is the BE source. Two walker entry points are
//! emitted: [`yaf16_to`] is an LE-only compatibility wrapper preserving the
//! single-generic signature `yaf16_to::<S>`; [`yaf16_to_endian::<S, BE>`] is
//! the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Yaf16Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Yaf16Frame;

walker! {
  packed_be {
    /// Marker type for the `Yaf16` source format (16-bit half-float gray +
    /// alpha, 2 f16/pixel). `<const BE: bool>` defaults to `false` (LE).
    ///
    /// Packed layout per pixel: `[Y(f16), A(f16)]`. Alpha is real source
    /// transparency and is passed through to RGBA outputs. Nominal range
    /// `[0.0, 1.0]`; HDR > 1.0 is permitted and clamped at output.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Yaf16,
    frame: Yaf16Frame,
    row: Yaf16Row,
    sink: Yaf16Sink,
    walker: yaf16_to,
    walker_endian: yaf16_to_endian,
    buf_field: packed,
    elem_type: half::f16,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Yaf16`] source — `width × 2` f16 elements (2 f16 per pixel:\n",
      "Y then A).\n",
      "\n",
      "f16 slot layout per pixel:\n",
      "\n",
      "| f16 slot | Field |\n",
      "|----------|-------|\n",
      "| 0        | Y (luma, 16-bit half-float)   |\n",
      "| 1        | A (real α, 16-bit half-float) |\n",
      "\n",
      "The walker does not interpret the f16 elements — it passes the raw packed\n",
      "slice to the sink. Endianness is recorded on the parent\n",
      "[`Yaf16Frame<'_, BE>`] / sinker, not on the Row itself — the kernel\n",
      "monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Yaf16Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Yaf16Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
