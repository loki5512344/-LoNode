//! Platform-specific sources (SoundCloud, Bandcamp, Twitch, Vimeo).

pub mod audio_platforms;
pub mod video_platforms;

pub use audio_platforms::{BandcampSource, SoundCloudSource};
pub use video_platforms::{TwitchSource, VimeoSource};
