use std::{fmt::Display, time::Duration, ops::{Add, Sub}};
use serde::{Serialize, Deserialize};

const MILLISECONDS_PER_HOUR: u128 = 3_600_000;
const MILLISECONDS_PER_MINUTE: u128 = 60_000;
const MILLISECONDS_PER_SECOND: u128 = 1000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StopWatchReadout {
    hours: u128,
    minutes: u128,
    seconds: u128,
    milliseconds: u128,
}

impl StopWatchReadout {
    pub fn new() -> Self {
        StopWatchReadout {
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 0,
        }
    }

    fn to_milliseconds(self) -> u128 {
        self.hours * MILLISECONDS_PER_HOUR + self.minutes * MILLISECONDS_PER_MINUTE + self.seconds * MILLISECONDS_PER_SECOND + self.milliseconds
    }

    pub fn from_duration(duration: Duration) -> StopWatchReadout {
        let mut milliseconds = duration.as_millis();
        let hours = milliseconds / MILLISECONDS_PER_HOUR;
        milliseconds %= MILLISECONDS_PER_HOUR;
        let minutes = milliseconds/ MILLISECONDS_PER_MINUTE;
        milliseconds %= MILLISECONDS_PER_MINUTE;
        let seconds = milliseconds / MILLISECONDS_PER_SECOND;
        milliseconds %= MILLISECONDS_PER_SECOND;
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }

    pub fn from_millis(mut milliseconds: u128) -> StopWatchReadout {
        let hours = milliseconds / MILLISECONDS_PER_HOUR;
        milliseconds %= MILLISECONDS_PER_HOUR;
        let minutes = milliseconds/ MILLISECONDS_PER_MINUTE;
        milliseconds %= MILLISECONDS_PER_MINUTE;
        let seconds = milliseconds / MILLISECONDS_PER_SECOND;
        milliseconds %= MILLISECONDS_PER_SECOND;
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }
}

impl Add for StopWatchReadout {
    type Output = StopWatchReadout;

    fn add(self, rhs: Self) -> Self::Output {
        let mut milliseconds = self.to_milliseconds() + rhs.to_milliseconds();
        let hours = milliseconds / MILLISECONDS_PER_HOUR;
        milliseconds %= MILLISECONDS_PER_HOUR;
        let minutes = milliseconds/ MILLISECONDS_PER_MINUTE;
        milliseconds %= MILLISECONDS_PER_MINUTE;
        let seconds = milliseconds / MILLISECONDS_PER_SECOND;
        milliseconds %= MILLISECONDS_PER_SECOND;
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }
}

impl Sub for StopWatchReadout {
    type Output = StopWatchReadout;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut milliseconds = self.to_milliseconds() - rhs.to_milliseconds();
        let hours = milliseconds / MILLISECONDS_PER_HOUR;
        milliseconds %= MILLISECONDS_PER_HOUR;
        let minutes = milliseconds/ MILLISECONDS_PER_MINUTE;
        milliseconds %= MILLISECONDS_PER_MINUTE;
        let seconds = milliseconds / MILLISECONDS_PER_SECOND;
        milliseconds %= MILLISECONDS_PER_SECOND;
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }
}

impl Display for StopWatchReadout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}hrs:{:02}min:{:02}secs", self.hours, self.minutes, self.seconds)
    }
}
