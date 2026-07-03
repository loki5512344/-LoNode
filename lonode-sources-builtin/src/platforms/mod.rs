//! Platform-specific sources (SoundCloud, Bandcamp, YouTube, Twitch, Vimeo).

pub mod audio_platforms;
pub mod video_platforms;

pub use audio_platforms::{BandcampSource, SoundCloudSource, YoutubeSource};
pub use video_platforms::{TwitchSource, VimeoSource};
