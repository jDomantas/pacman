use chrono::{DateTime, Duration, Utc};

pub struct RateLimitExceeded;

pub struct RateLimiter {
    max_submissions: usize,
    time_window: Duration,
    entries: Vec<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new(max_submissions: usize, time_window: Duration) -> Self {
        assert!(max_submissions > 0, "rate limiter must allow at least one submission");
        RateLimiter {
            max_submissions,
            time_window,
            entries: Vec::new(),
        }
    }

    pub fn submit(&mut self, time: DateTime<Utc>) -> Result<(), RateLimitExceeded> {
        if self.entries.len() < self.max_submissions {
            self.entries.push(time);
            return Ok(());
        }
        let oldest = self.entries
            .iter()
            .enumerate()
            .min_by_key(|item| item.1);
        if let Some((index, instant)) = oldest {
            if *instant + self.time_window < time {
                self.entries[index] = time;
                Ok(())
            } else {
                Err(RateLimitExceeded)
            }
        } else {
            self.entries.push(time);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn instant_submissions() {
        let mut limiter = RateLimiter::new(3, Duration::minutes(3));
        let time = Utc.timestamp(123456789, 0);
        assert!(limiter.submit(time).is_ok());
        assert!(limiter.submit(time).is_ok());
        assert!(limiter.submit(time).is_ok());
        assert!(limiter.submit(time).is_err());
    }

    #[test]
    fn spaced_out_submissions() {
        let mut limiter = RateLimiter::new(3, Duration::minutes(3));
        let time = Utc.timestamp(123456789, 0);
        assert!(limiter.submit(time + Duration::minutes(2)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(4)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(6)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(8)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(10)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(12)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(14)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(16)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(16)).is_ok());
        assert!(limiter.submit(time + Duration::minutes(16)).is_err());
        assert!(limiter.submit(time + Duration::minutes(18)).is_ok());
    }
}
