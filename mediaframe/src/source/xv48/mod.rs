//! Packed YUV 4:4:4 16-bit `XV48` source — full-depth packed capture
//! format (FFmpeg `AV_PIX_FMT_XV48{LE,BE}`). Each pixel is a u16
//! quadruple `U(16) ‖ Y(16) ‖ V(16) ‖ X(16)` with every channel using
//! the full 16 bits (no MSB shift — the full-depth sibling of `XV36`,
//! which is 12-bit MSB-aligned). The channel order is fixed by FFmpeg
//! `libavutil/pixdesc.c` (`AV_PIX_FMT_XV48LE` `.comp` byte offsets
//! `U` @ 0, `Y` @ 2, `V` @ 4, `X` @ 6) — identical ordering to `XV36`.
//! The `X` slot is padding ("variant of Y416 where alpha channel is
//! left undefined") — read but discarded; RGBA outputs force α = max.
//! See [`Xv48Frame`](crate::frame::Xv48Frame) for layout details.
//!
//! The marker carries `<const BE: bool = false>`: `Xv48` (=
//! `Xv48<false>`) is the LE source; `Xv48<true>` is the BE source.
//! The walker [`xv48_to::<BE>`] propagates `BE` from
//! [`Xv48Frame<'_, BE>`] into the sinker dispatch.
//!
//! Outputs are produced via:
//! - `with_rgb` / `with_rgba` — packed YUV → RGB Q15 pipeline at
//!   BITS=16, downshifted to u8; RGBA α = `0xFF` (XV48 has no alpha
//!   channel — X slot is padding).
//! - `with_rgb_u16` / `with_rgba_u16` — same pipeline at native
//!   16-bit depth; RGBA α = `0xFFFF` (16-bit max).
//! - `with_luma` — extracts Y values from each XV48 quadruple and
//!   downshifts via `>> 8` (16-bit → u8).
//! - `with_luma_u16` — passes the 16-bit Y values straight through
//!   into u16 (full-depth, no shift).
//! - `with_hsv` — stages an internal RGB scratch and runs the
//!   existing `rgb_to_hsv_row` kernel.

use crate::frame::Xv48Frame;

walker! {
  packed_be {
    /// Zero-sized marker for the packed **XV48** source format
    /// (`AV_PIX_FMT_XV48{LE,BE}`). `<const BE: bool>` defaults to
    /// `false` (LE); the alias `Xv48` resolves to `Xv48<false>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Xv48,
    frame: Xv48Frame,
    row: Xv48Row,
    sink: Xv48Sink,
    walker: xv48_to,
    walker_endian: xv48_to_endian,
    buf_field: packed,
    elem_type: u16,
    row_elems: |w| w * 4,
    row_doc: concat!(
      "One row of an [`Xv48`] source — `width × 4` u16 elements (4\n",
      "channels per pixel: U, Y, V, X; the X slot is padding).\n",
      "\n",
      "Each u16 channel holds a full 16-bit sample (all bits active).\n",
      "Channel layout per pixel:\n",
      "\n",
      "| u16 slot | Field | Active bits           |\n",
      "|----------|-------|-----------------------|\n",
      "| 0        | U     | bits `15:0` (16-bit) |\n",
      "| 1        | Y     | bits `15:0` (16-bit) |\n",
      "| 2        | V     | bits `15:0` (16-bit) |\n",
      "| 3        | X     | bits `15:0` (padding)|\n",
      "\n",
      "Full range: `[0, 65535]` (16-bit). Endianness is recorded on\n",
      "the parent [`Xv48Frame<'_, BE>`] / sinker, not on the Row itself —\n",
      "the kernel monomorphizes on `BE` at the sinker dispatch.",
    ),
    walker_doc: "Walks an [`Xv48Frame<'_, BE>`] row by row into the sink. \
                 Propagates `<const BE: bool>` from the frame into \
                 [`Xv48Sink<BE>`].",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
