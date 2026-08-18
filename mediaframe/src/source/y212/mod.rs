//! Packed YUV 4:2:2 12-bit `Y212` source — high-bit-depth packed
//! capture format (Microsoft Media Foundation / DXVA HEVC 12-bit
//! 4:2:2 hardware decode). Each row is a sequence of YUYV-shaped
//! u16 quadruples (`Y₀, U, Y₁, V`); active 12 bits are MSB-aligned
//! in each u16 (low 4 bits = 0). See [`Y212Frame`](crate::frame::Y212Frame)
//! for layout details.
//!
//! The marker carries `<const BE: bool = false>`: `Y212` (= `Y212<false>`)
//! is the LE source; `Y212<true>` is the BE source. The walker
//! [`y212_to::<BE>`] propagates `BE` from
//! [`Y2xxFrame<'_, 12, BE>`](crate::frame::Y2xxFrame) into the
//! sinker dispatch.
//!
//! Outputs are produced via:
//! - `with_rgb` / `with_rgba` — packed YUV → RGB Q15 pipeline at
//!   BITS=12, downshifted to u8.
//! - `with_rgb_u16` / `with_rgba_u16` — same pipeline at native
//!   12-bit depth, low-bit-packed in `u16`.
//! - `with_luma` — extracts the Y values from each Y212 quadruple
//!   and downshifts via `>> 8` (12-bit MSB-aligned → u8).
//! - `with_luma_u16` — extracts the 12-bit Y values into u16
//!   (low-bit-packed).
//! - `with_hsv` — stages an internal RGB scratch and runs the
//!   existing `rgb_to_hsv_row` kernel.

// `Y212Frame` is referenced through `$crate::frame::Y2xxFrame<'_, 12, BE>` by
// the `packed_be_y2xx` walker arm; no outer import needed.

walker! {
  packed_be_y2xx {
    /// Zero-sized marker for the packed **Y212** source format
    /// (`AV_PIX_FMT_Y212{LE,BE}`). `<const BE: bool>` defaults to `false`
    /// (LE); `Y212` resolves to `Y212<false>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Y212,
    frame_inner: Y2xxFrame,
    bits: 12,
    row: Y212Row,
    sink: Y212Sink,
    walker: y212_to,
    walker_endian: y212_to_endian,
    buf_field: packed,
    elem_type: u16,
    row_elems: |w| w * 2,
    row_doc: concat!(
      "One row of a [`Y212`] source — `width × 2` u16 elements\n",
      "(`Y₀, U, Y₁, V` quadruples per 2-pixel block).\n",
      "\n",
      "Each u16 sample carries an active 12-bit value MSB-aligned (the\n",
      "low 4 bits are zero). Per 2-pixel block layout (4 u16 elements):\n",
      "\n",
      "| u16 slot | Field | Active bits           |\n",
      "|----------|-------|-----------------------|\n",
      "| 0        | Y₀    | bits `15:4` (12-bit) |\n",
      "| 1        | U     | bits `15:4` (12-bit) |\n",
      "| 2        | Y₁    | bits `15:4` (12-bit) |\n",
      "| 3        | V     | bits `15:4` (12-bit) |\n",
      "\n",
      "Full range Y: `[0, 4095]` (12-bit MSB-aligned in u16). Limited\n",
      "range Y: `[256, 3760]`, limited range chroma: `[256, 3840]`.\n",
      "\n",
      "Endianness is recorded on the parent \
       [`Y2xxFrame<'_, 12, BE>`](crate::frame::Y2xxFrame) / sinker,\n",
      "not on the Row itself — the kernel receives `BE` as the runtime\n",
      "`big_endian` argument from the sinker dispatch.",
    ),
    walker_doc: "Walks a [`Y2xxFrame<'_, 12, BE>`](crate::frame::Y2xxFrame) row \
                 by row into the sink. Propagates `<const BE: bool>` from the \
                 frame into [`Y212Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
