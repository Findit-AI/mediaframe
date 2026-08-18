//! Walker spec for the `Yaf32` source format
//! (FFmpeg `yaf32{le,be}` / `AV_PIX_FMT_YAF32{LE,BE}`).
//!
//! Single `f32` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel occupies
//! 2 f32 elements; stride is in f32 elements and must be `≥ width × 2` (may
//! include row padding). Alpha is real source α at element slot 1 of every
//! pixel pair. The single-precision twin of [`super::Yaf16`].
//!
//! The marker carries `<const BE: bool = false>`: `Yaf32` (= `Yaf32<false>`) is
//! the LE source; `Yaf32<true>` is the BE source. Two walker entry points are
//! emitted: [`yaf32_to`] is an LE-only compatibility wrapper preserving the
//! single-generic signature `yaf32_to::<S>`; [`yaf32_to_endian::<S, BE>`] is
//! the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Yaf32Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Yaf32Frame;

walker! {
  packed_be {
    /// Marker type for the `Yaf32` source format (32-bit float gray + alpha,
    /// 2 f32/pixel). `<const BE: bool>` defaults to `false` (LE).
    ///
    /// Packed layout per pixel: `[Y(f32), A(f32)]`. Alpha is real source
    /// transparency and is passed through to RGBA outputs. Nominal range
    /// `[0.0, 1.0]`; HDR > 1.0 is permitted and clamped at output.
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    marker: Yaf32,
    frame: Yaf32Frame,
    row: Yaf32Row,
    sink: Yaf32Sink,
    walker: yaf32_to,
    walker_endian: yaf32_to_endian,
    buf_field: packed,
    elem_type: f32,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Yaf32`] source — `width × 2` f32 elements (2 f32 per pixel:\n",
      "Y then A).\n",
      "\n",
      "f32 slot layout per pixel:\n",
      "\n",
      "| f32 slot | Field |\n",
      "|----------|-------|\n",
      "| 0        | Y (luma, 32-bit float)   |\n",
      "| 1        | A (real α, 32-bit float) |\n",
      "\n",
      "The walker does not interpret the f32 elements — it passes the raw packed\n",
      "slice to the sink. Endianness is recorded on the parent\n",
      "[`Yaf32Frame<'_, BE>`] / sinker, not on the Row itself — the kernel\n",
      "monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Yaf32Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Yaf32Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
