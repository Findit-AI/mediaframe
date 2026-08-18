//! Walker spec for the `Ya8` source format (FFmpeg `ya8` / `AV_PIX_FMT_YA8`).
//!
//! Single `u8` plane packed as `[Y0, A0, Y1, A1, ...]`. Each pixel occupies
//! 2 bytes; stride covers `width × 2` bytes. Alpha is real source α at slot 1
//! of every pixel pair.

use crate::frame::Ya8Frame;

walker! {
  packed {
    /// Marker type for the `Ya8` source format (8-bit gray + alpha, 2 bytes/pixel).
    ///
    /// Packed layout per pixel: `[Y(8), A(8)]`. Alpha is real source transparency
    /// and is passed through to RGBA outputs.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Ya8,
    frame: Ya8Frame<'_>,
    row: Ya8Row,
    sink: Ya8Sink,
    walker: ya8_to,
    buf_field: packed,
    elem_type: u8,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Ya8`] source — `width × 2` bytes (2 bytes per pixel:\n",
      "Y then A).\n",
      "\n",
      "Byte layout per pixel:\n",
      "\n",
      "| Byte offset | Field |\n",
      "|-------------|-------|\n",
      "| 0           | Y (luma)      |\n",
      "| 1           | A (real α)    |\n",
      "\n",
      "The walker does not interpret the bytes — it passes the raw packed\n",
      "slice to the sink. Byte-level channel extraction happens in the\n",
      "row-kernel layer.",
    ),
    walker_doc: "Walks a [`Ya8Frame`](crate::frame::Ya8Frame) row by row into the sink.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
