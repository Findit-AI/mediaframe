//! Walker spec for the `Ya16` source format
//! (FFmpeg `ya16{le,be}` / `AV_PIX_FMT_YA16{LE,BE}`).
//!
//! Single `u16` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel occupies
//! 2 u16 elements; stride is in u16 elements and must be `≥ width × 2` (may
//! include row padding). Alpha is real source α at element slot 1 of every
//! pixel pair.
//!
//! The marker carries `<const BE: bool = false>`: `Ya16` (= `Ya16<false>`) is
//! the LE source; `Ya16<true>` is the BE source. Two walker entry points are
//! emitted: [`ya16_to`] is an LE-only compatibility wrapper preserving the
//! single-generic signature `ya16_to::<S>`; [`ya16_to_endian::<S, BE>`] is
//! the const-generic entry point for BE-aware callers, propagating `BE`
//! from [`Ya16Frame<'_, BE>`] into the sinker dispatch.

use crate::frame::Ya16Frame;

walker! {
  packed_be {
    /// Marker type for the `Ya16` source format (16-bit gray + alpha,
    /// 2 u16/pixel). `<const BE: bool>` defaults to `false` (LE).
    ///
    /// Packed layout per pixel: `[Y(16), A(16)]`. Alpha is real source
    /// transparency and is passed through to RGBA outputs (depth-converted
    /// to u8 via `>> 8` for 8-bit RGBA output).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Ya16,
    frame: Ya16Frame,
    row: Ya16Row,
    sink: Ya16Sink,
    walker: ya16_to,
    walker_endian: ya16_to_endian,
    buf_field: packed,
    elem_type: u16,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Ya16`] source — `width × 2` u16 elements (2 u16 per pixel:\n",
      "Y then A).\n",
      "\n",
      "u16 slot layout per pixel:\n",
      "\n",
      "| u16 slot | Field |\n",
      "|----------|-------|\n",
      "| 0        | Y (luma, 16-bit native)   |\n",
      "| 1        | A (real α, 16-bit native) |\n",
      "\n",
      "The walker does not interpret the u16 elements — it passes the raw packed\n",
      "slice to the sink. Endianness is recorded on the parent\n",
      "[`Ya16Frame<'_, BE>`] / sinker, not on the Row itself — the kernel\n",
      "monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Ya16Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Ya16Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
