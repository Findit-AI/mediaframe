//! Audio-stream descriptor vocabulary — channel layout (the name) and
//! channel layout description (the structure), sample / container
//! format, bit-rate mode, EBU R128 loudness, ReplayGain normalization,
//! fingerprint, embedded metadata tags + cover art.

pub mod bit_rate_mode;
pub mod channel_layout;
pub mod channel_layout_description;
pub mod channel_order;
pub mod channel_spec;
pub mod cover_art;
pub mod fingerprint;
pub mod format;
pub mod loudness;
pub mod replay_gain;
pub mod tags;

pub use bit_rate_mode::BitRateMode;
pub use channel_layout::ChannelLayout;
pub use channel_layout_description::ChannelLayoutDescription;
pub use channel_order::ChannelOrder;
pub use channel_spec::ChannelSpec;
pub use cover_art::{CoverArt, CoverArtError};
pub use fingerprint::{Fingerprint, FingerprintError};
pub use format::{ContainerFormat, SampleFormat};
pub use loudness::Loudness;
pub use replay_gain::ReplayGain;
pub use tags::Tags;
