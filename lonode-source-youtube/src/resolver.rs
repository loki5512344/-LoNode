//! YouTube resolver — `AudioSource` impl + URL parsing.

use crate::reader::YtdlReader;
use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use rusty_ytdl::{Video, VideoOptions, VideoQuality};
use tokio::io::AsyncRead;

/// YouTube audio source. Downloads videos via `rusty_ytdl` and returns the
/// audio stream as `AsyncRead`.
pub struct YoutubeSource {
    options: VideoOptions,
}

impl Default for YoutubeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl YoutubeSource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: VideoOptions {
                quality: VideoQuality::LowestAudio,
                ..Default::default()
            },
        }
    }

    /// `true` if `url` is a YouTube watch/short URL.
    #[must_use]
    pub fn is_youtube_url(url: &str) -> bool {
        url.contains("youtube.com/watch")
            || url.contains("youtu.be/")
            || url.contains("youtube.com/shorts/")
    }
}

#[async_trait]
impl AudioSource for YoutubeSource {
    fn name(&self) -> &str {
        "youtube"
    }

    fn supports(&self, url: &str) -> bool {
        Self::is_youtube_url(url)
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        let video = Video::new(url).map_err(|e| PluginError::Resolve(e.to_string()))?;
        let info = video
            .get_basic_info()
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        let details = &info.video_details;
        let duration_ms = details.length_seconds.parse::<u64>().unwrap_or(0) * 1000;
        Ok(TrackInfo {
            title: details.title.clone(),
            author: details.owner_channel_name.clone(),
            duration_ms,
            url: url.to_string(),
        })
    }

    async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        let video = Video::new_with_options(url, self.options.clone())
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        let stream = video
            .stream()
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        let reader = YtdlReader::from_stream(stream)
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        Ok(Box::new(reader))
    }
}

/// Extract the 11-char video ID from a YouTube URL.
#[must_use]
pub fn extract_video_id(url: &str) -> Option<String> {
    if let Some(pos) = url.find("youtu.be/") {
        return Some(
            url[pos + "youtu.be/".len()..]
                .split('&')
                .next()?
                .split('?')
                .next()?
                .to_string(),
        );
    }
    if let Some(pos) = url.find("v=") {
        return Some(url[pos + "v=".len()..].split('&').next()?.to_string());
    }
    for prefix in [
        "youtube.com/shorts/",
        "youtube.com/embed/",
        "youtube.com/v/",
    ] {
        if let Some(pos) = url.find(prefix) {
            return Some(
                url[pos + prefix.len()..]
                    .split('?')
                    .next()?
                    .split('&')
                    .next()?
                    .to_string(),
            );
        }
    }
    None
}
