//! Player + queue integration tests.

use lonode_core::player::{GuildPlayer, PlayState, Queue, Track};

#[test]
fn guild_player_play_pause_resume() {
    let mut p = GuildPlayer::new();
    assert_eq!(p.state(), PlayState::Stopped);
    p.play(Track::new("a", "A"));
    assert_eq!(p.state(), PlayState::Playing);
    p.pause();
    assert_eq!(p.state(), PlayState::Paused);
    p.resume();
    assert_eq!(p.state(), PlayState::Playing);
}

#[test]
fn guild_player_skip_advances_and_stops() {
    let mut p = GuildPlayer::new();
    p.play(Track::new("a", "A"));
    p.play(Track::new("b", "B"));
    assert!(p.skip());
    assert_eq!(p.current_track().unwrap().id.0, "b");
    assert!(!p.skip());
    assert_eq!(p.state(), PlayState::Stopped);
}

#[test]
fn guild_player_volume_clamped_to_thousand() {
    let mut p = GuildPlayer::new();
    p.set_volume(5_000);
    assert_eq!(p.volume(), 1_000);
}

#[test]
fn queue_starts_empty_and_drains_fifo() {
    let mut q = Queue::new();
    assert!(q.is_empty());
    q.push(Track::new("a", "A"));
    q.push(Track::new("b", "B"));
    assert_eq!(q.len(), 2);
    assert_eq!(q.advance().unwrap().id.0, "a");
    assert_eq!(q.advance().unwrap().id.0, "b");
    assert!(q.advance().is_none());
}

#[test]
fn queue_clear_keeps_current_track() {
    let mut q = Queue::new();
    q.push(Track::new("a", "A"));
    q.push(Track::new("b", "B"));
    q.push(Track::new("c", "C"));
    q.advance();
    q.clear();
    assert_eq!(q.len(), 0);
    assert!(q.current().is_some());
}
