//! Timer logic is independent of the tray, webview, and scheduling callbacks.
//! Wall-clock deadlines include time spent asleep (unlike a decrementing counter).
use std::time::{Duration, SystemTime};

pub const BLINK_INTERVAL: Duration = Duration::from_millis(500);
pub const BLINK_DURATION: Duration = Duration::from_secs(15);
pub const ANIMATION_INTERVAL: Duration = Duration::from_millis(500);
pub const SAND_STAGES: usize = 8;
pub const SAND_PHASES: usize = 4;
pub const ANIMATION_FRAMES: usize = SAND_STAGES * SAND_PHASES;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    #[default]
    Idle,
    Running {
        end_time: SystemTime,
        duration: Duration,
    },
    Paused {
        remaining: Duration,
        duration: Duration,
    },
    Finished {
        blink_started: SystemTime,
    },
}

impl Timer {
    /// Returns true exactly once when the deadline is crossed.
    pub fn advance(&mut self, now: SystemTime) -> bool {
        if let Self::Running { end_time, .. } = *self {
            if now >= end_time {
                *self = Self::Finished { blink_started: now };
                return true;
            }
        }
        false
    }

    /// A finished timer is acknowledged first; the next click starts a new one.
    pub fn click(&mut self, duration: Duration, now: SystemTime) -> bool {
        let finished = self.advance(now);
        *self = match *self {
            Self::Idle => Self::Running {
                end_time: now + duration,
                duration,
            },
            Self::Running { end_time, duration } => Self::Paused {
                remaining: end_time.duration_since(now).unwrap_or_default(),
                duration,
            },
            Self::Paused {
                remaining,
                duration,
            } => Self::Running {
                end_time: now + remaining,
                duration,
            },
            Self::Finished { .. } => Self::Idle,
        };
        finished
    }

    pub fn can_restart(&self, now: SystemTime) -> bool {
        match *self {
            Self::Running { end_time, .. } => now < end_time,
            Self::Paused { .. } => true,
            _ => false,
        }
    }

    /// Restart with the requested full duration and reset animation progress.
    /// Restarting a paused timer also resumes it.
    pub fn restart(&mut self, duration: Duration, now: SystemTime) -> bool {
        if !self.can_restart(now) {
            return false;
        }
        *self = Self::Running {
            end_time: now + duration,
            duration,
        };
        true
    }

    /// Animation follows elapsed session time, so pausing freezes both sand and grains.
    pub fn icon_frame(&self, now: SystemTime) -> Option<usize> {
        let duration = match *self {
            Self::Running { duration, .. } | Self::Paused { duration, .. } => duration,
            Self::Finished { .. } => return Some(ANIMATION_FRAMES - SAND_PHASES),
            Self::Idle => return None,
        };
        let elapsed = duration.saturating_sub(self.remaining(now));
        let stage = (elapsed.as_nanos() * (SAND_STAGES - 1) as u128 / duration.as_nanos().max(1))
            .min((SAND_STAGES - 1) as u128) as usize;
        let phase =
            (elapsed.as_millis() / ANIMATION_INTERVAL.as_millis() % SAND_PHASES as u128) as usize;
        Some(stage * SAND_PHASES + phase)
    }

    pub fn remaining(&self, now: SystemTime) -> Duration {
        match *self {
            Self::Running { end_time, .. } => end_time.duration_since(now).unwrap_or_default(),
            Self::Paused { remaining, .. } => remaining,
            _ => Duration::ZERO,
        }
    }

    pub fn blinking(&self, now: SystemTime) -> bool {
        matches!(*self, Self::Finished { blink_started }
            if now.duration_since(blink_started).unwrap_or_default() < BLINK_DURATION)
    }

    pub fn icon_visible(&self, now: SystemTime) -> bool {
        match *self {
            Self::Finished { blink_started } if self.blinking(now) => (now
                .duration_since(blink_started)
                .unwrap_or_default()
                .as_millis()
                / BLINK_INTERVAL.as_millis())
            .is_multiple_of(2),
            _ => true,
        }
    }

    /// None means the worker can block indefinitely until a user action.
    pub fn next_wake(&self, now: SystemTime) -> Option<Duration> {
        match *self {
            Self::Running { end_time, duration } => {
                let remaining = end_time.duration_since(now).unwrap_or_default();
                // Align to the next displayed second, without accumulating drift.
                let millis = remaining.as_millis();
                let next = if millis.is_multiple_of(1000) {
                    1000
                } else {
                    millis % 1000
                };
                let elapsed = duration.saturating_sub(remaining);
                let animation_next = ANIMATION_INTERVAL.as_millis()
                    - elapsed.as_millis() % ANIMATION_INTERVAL.as_millis();
                Some(
                    Duration::from_millis(next.min(animation_next) as u64)
                        .min(remaining)
                        .max(Duration::from_millis(1)),
                )
            }
            Self::Finished { blink_started } if self.blinking(now) => {
                let elapsed = now.duration_since(blink_started).unwrap_or_default();
                let next = 500 - (elapsed.as_millis() % 500) as u64;
                Some(Duration::from_millis(next).min(BLINK_DURATION - elapsed))
            }
            _ => None,
        }
    }
}

pub fn format_remaining(remaining: Duration) -> String {
    // Round up: a fresh 50 minute timer never immediately reads 49:59.
    let seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0);
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn sleep_and_delayed_callbacks_use_the_deadline() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(3000), at(100));
        assert_eq!(timer.remaining(at(1300)), Duration::from_secs(1800));
        assert!(!timer.advance(at(1300)));
        assert!(timer.advance(at(4000)));
        assert!(!timer.advance(at(4001)), "notify only once");
        assert_eq!(timer.remaining(at(4001)), Duration::ZERO);
    }

    #[test]
    fn pause_preserves_fractional_time_and_resume_gets_a_new_deadline() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(60), at(100));
        timer.click(
            Duration::from_secs(60),
            at(110) + Duration::from_millis(250),
        );
        assert_eq!(timer.remaining(at(9000)), Duration::from_millis(49750));
        assert_eq!(timer.next_wake(at(9000)), None);
        timer.click(Duration::from_secs(60), at(9000));
        assert_eq!(timer.remaining(at(9020)), Duration::from_millis(29750));
    }

    #[test]
    fn click_at_expiry_acknowledges_instead_of_pausing_zero_seconds() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(60), at(100));
        assert!(timer.click(Duration::from_secs(60), at(160)));
        assert_eq!(timer, Timer::Idle);
        timer.click(Duration::from_secs(60), at(161));
        assert_eq!(timer.remaining(at(161)), Duration::from_secs(60));
    }

    #[test]
    fn blinking_is_bounded_and_acknowledge_stops_it() {
        let mut timer = Timer::Finished {
            blink_started: at(100),
        };
        assert!(timer.icon_visible(at(100)));
        assert!(!timer.icon_visible(at(100) + BLINK_INTERVAL));
        assert!(timer.icon_visible(at(101)));
        assert!(!timer.blinking(at(115)));
        assert!(timer.icon_visible(at(115)));
        assert_eq!(timer.next_wake(at(115)), None);
        timer.click(Duration::from_secs(60), at(101));
        assert_eq!(timer, Timer::Idle);
        assert_eq!(timer.next_wake(at(101)), None);
    }

    #[test]
    fn display_rounds_up_and_supports_long_timers() {
        assert_eq!(format_remaining(Duration::from_millis(1)), "00:01");
        assert_eq!(format_remaining(Duration::from_millis(59999)), "01:00");
        assert_eq!(format_remaining(Duration::from_secs(7200)), "120:00");
        assert_eq!(format_remaining(Duration::ZERO), "00:00");
    }

    #[test]
    fn idle_and_paused_have_no_periodic_wakeups() {
        assert_eq!(Timer::Idle.next_wake(at(100)), None);
        let timer = Timer::Paused {
            remaining: Duration::from_secs(30),
            duration: Duration::from_secs(60),
        };
        assert_eq!(timer.next_wake(at(100)), None);
    }

    #[test]
    fn restart_after_ten_minutes_restores_the_full_session() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(3000), at(100));
        assert_eq!(timer.remaining(at(700)), Duration::from_secs(2400));
        assert!(timer.restart(Duration::from_secs(3000), at(700)));
        assert_eq!(timer.remaining(at(700)), Duration::from_secs(3000));
        assert_eq!(timer.icon_frame(at(700)), Some(0));
        assert!(!timer.advance(at(3100)), "the old deadline is discarded");
        assert!(timer.advance(at(3700)));
        assert!(!timer.advance(at(3701)));
    }

    #[test]
    fn restart_uses_the_new_duration_and_resumes_a_paused_timer() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(3000), at(100));
        // Passing new settings to a pause click must not change this session.
        timer.click(Duration::from_secs(1500), at(700));
        assert!(timer.restart(Duration::from_secs(1500), at(900)));
        assert_eq!(timer.remaining(at(900)), Duration::from_secs(1500));
        assert_eq!(timer.remaining(at(901)), Duration::from_secs(1499));
        assert_eq!(timer.icon_frame(at(900)), Some(0));
        assert!(
            matches!(timer, Timer::Running { duration, .. } if duration == Duration::from_secs(1500))
        );
        assert!(timer.advance(at(2400)));
        assert!(!timer.advance(at(3100)));
    }

    #[test]
    fn restart_applies_shorter_and_longer_durations_to_a_running_timer() {
        for seconds in [1500, 5400] {
            let mut timer = Timer::Idle;
            timer.click(Duration::from_secs(3000), at(100));
            let duration = Duration::from_secs(seconds);
            assert!(timer.restart(duration, at(700)));
            assert_eq!(timer.remaining(at(700)), duration);
            assert_eq!(timer.icon_frame(at(700)), Some(0));
            assert!(matches!(timer, Timer::Running { duration: actual, .. } if actual == duration));
            assert!(!timer.advance(at(700 + seconds - 1)));
            assert!(timer.advance(at(700 + seconds)));
            assert!(!timer.advance(at(701 + seconds)));
        }
    }

    #[test]
    fn restart_does_not_start_an_idle_or_finished_timer() {
        let mut timer = Timer::Idle;
        assert!(!timer.restart(Duration::from_secs(1500), at(100)));
        timer.click(Duration::from_secs(60), at(100));
        assert!(
            !timer.restart(Duration::from_secs(1500), at(160)),
            "expired timers cannot be restarted"
        );
        assert!(
            timer.advance(at(160)),
            "expiry still emits its notification"
        );
        assert!(!timer.restart(Duration::from_secs(1500), at(161)));
    }

    #[test]
    fn grains_animate_and_pause_freezes_the_frame() {
        let mut timer = Timer::Idle;
        timer.click(Duration::from_secs(60), at(100));
        assert_eq!(timer.icon_frame(at(100)), Some(0));
        assert_eq!(timer.icon_frame(at(100) + ANIMATION_INTERVAL), Some(1));
        assert_eq!(timer.next_wake(at(100)), Some(ANIMATION_INTERVAL));
        timer.click(Duration::from_secs(60), at(112) + ANIMATION_INTERVAL);
        let frame = timer.icon_frame(at(112) + ANIMATION_INTERVAL);
        assert_eq!(timer.icon_frame(at(9000)), frame);
        assert_eq!(timer.next_wake(at(9000)), None);
        timer.click(Duration::from_secs(60), at(9000));
        assert_eq!(timer.icon_frame(at(9000)), frame);
        assert_ne!(timer.icon_frame(at(9000) + ANIMATION_INTERVAL), frame);
    }
}
