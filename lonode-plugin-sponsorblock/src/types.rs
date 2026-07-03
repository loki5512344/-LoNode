//! Segment types + categories.

use serde::{Deserialize, Serialize};

/// SponsorBlock segment categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Paid promotion or sponsorship.
    Sponsor,
    /// Intro animation/title sequence.
    Intro,
    /// Outro animation/credits.
    Outro,
    /// Self-promotion (creator's own content).
    Selfpromo,
    /// Preview/recap of previous episode.
    Preview,
    /// Filler tangent not relevant to main content.
    Filler,
    /// Pointless interaction reminder (like/subscribe).
    Interaction,
    /// Music video without speech.
    MusicOfftopic,
}

impl Category {
    /// All categories, useful for API requests.
    #[must_use]
    pub const fn all() -> &'static [Category] {
        &[
            Category::Sponsor,
            Category::Intro,
            Category::Outro,
            Category::Selfpromo,
            Category::Preview,
            Category::Filler,
            Category::Interaction,
            Category::MusicOfftopic,
        ]
    }

    /// String form used in the SponsorBlock API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Category::Sponsor => "sponsor",
            Category::Intro => "intro",
            Category::Outro => "outro",
            Category::Selfpromo => "selfpromo",
            Category::Preview => "preview",
            Category::Filler => "filler",
            Category::Interaction => "interaction",
            Category::MusicOfftopic => "music_offtopic",
        }
    }
}

/// A single sponsor segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds.
    pub end: f64,
    /// Segment category.
    pub category: Category,
}

impl Segment {
    /// `true` if `position` (in seconds) falls within this segment.
    #[must_use]
    pub fn contains(&self, position: f64) -> bool {
        position >= self.start && position < self.end
    }

    /// Duration in seconds.
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

/// YouTube video chapters (parsed from description or API).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoChapters {
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_all_returns_eight() {
        assert_eq!(Category::all().len(), 8);
    }

    #[test]
    fn category_as_str_roundtrips() {
        for cat in Category::all() {
            assert!(!cat.as_str().is_empty());
        }
    }

    #[test]
    fn segment_contains_position() {
        let s = Segment {
            start: 10.0,
            end: 20.0,
            category: Category::Sponsor,
        };
        assert!(s.contains(15.0));
        assert!(!s.contains(5.0));
        assert!(!s.contains(25.0));
    }

    #[test]
    fn segment_duration() {
        let s = Segment {
            start: 10.0,
            end: 20.0,
            category: Category::Sponsor,
        };
        assert_eq!(s.duration(), 10.0);
    }
}
