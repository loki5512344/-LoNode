//! SponsorBlock client tests.

use lonode_plugin_sponsorblock::{Category, Segment, SponsorBlockClient};

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

#[test]
fn client_constructs_with_default_base() {
    let c = SponsorBlockClient::new();
    assert!(c.base_url().contains("sponsor.ajay.app"));
}

#[test]
fn client_accepts_custom_base() {
    let c = SponsorBlockClient::with_base_url("https://custom.example".into());
    assert_eq!(c.base_url(), "https://custom.example");
}

#[test]
fn urlencoding_encodes_brackets() {
    assert_eq!(
        SponsorBlockClient::urlencoding_encode("[\"sponsor\"]"),
        "%5B%22sponsor%22%5D"
    );
}

#[test]
fn urlencoding_preserves_alphanumerics() {
    assert_eq!(SponsorBlockClient::urlencoding_encode("abc123"), "abc123");
}
