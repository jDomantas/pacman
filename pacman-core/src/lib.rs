#![allow(unused)]

pub mod contract;
mod rate_limiter;
mod scoreboard;
mod evaluator;

use std::collections::HashMap;
use chrono::{DateTime, Duration, TimeZone, Utc};
use rate_limiter::{RateLimiter, RateLimitExceeded};
use scoreboard::Scoreboard;

#[derive(Debug, Copy, Clone)]
pub struct GameConfig {
    pub max_steps: u64,
    pub rate_limit_count: usize,
    pub rate_limit_window: Duration,
}

struct UserSubmission {
    user: String,
    details: contract::SubmissionDetails,
}

pub struct PacmanGame {
    global_scores: Scoreboard,
    level_scores: Scoreboard,
    current_level: contract::Level,
    limiters: HashMap<String, RateLimiter>,
    is_level_closed: bool,
    config: GameConfig,
    level_start: DateTime<Utc>,
    submissions: Vec<UserSubmission>,
}

impl PacmanGame {
    pub fn new(config: GameConfig) -> Self {
        PacmanGame {
            global_scores: Scoreboard::new(),
            level_scores: Scoreboard::new(),
            current_level: empty_level(),
            limiters: HashMap::new(),
            is_level_closed: true,
            config,
            level_start: Utc.timestamp(0, 0),
            submissions: Vec::new(),
        }
    }

    pub fn set_config(&mut self, config: GameConfig) {
        self.config = config;
    }

    pub fn set_level(&mut self, level: contract::Level, now: DateTime<Utc>) {
        self.global_scores.add_level_scores(&self.level_scores);
        self.level_scores = Scoreboard::new();
        self.current_level = level;
        self.limiters.clear();
        self.is_level_closed = false;
        self.level_start = now;
        self.submissions.clear();
    }

    pub fn set_level_state(&mut self, closed: bool) {
        self.is_level_closed = closed;
    }

    pub fn get_scores(&self) -> contract::Scoreboards {
        contract::Scoreboards {
            scoreboards: vec![
                self.level_scores.to_contract_with_time("Results"),
                self.level_scores.to_contract_with_size("Results (by size)"),
                self.global_scores.to_contract_with_time("Total"),
                self.global_scores.to_contract_with_size("Total (by size)"),
            ],
        }
    }

    pub fn submit_program(&mut self, user: &str, program: &contract::Program, now: DateTime<Utc>) -> contract::SubmitResponse {
        if self.is_level_closed {
            return contract::SubmitResponse::LevelClosed;
        }
        let config = self.config;
        let can_submit = self.limiters
            .entry(user.to_owned())
            .or_insert_with(|| RateLimiter::new(
                config.rate_limit_count,
                config.rate_limit_window,
            ))
            .submit(now);
        match can_submit {
            Ok(()) => {
                let details = evaluator::evaluate_program(
                    &self.current_level,
                    &program,
                    self.config.max_steps,
                );
                if details.outcome == contract::Outcome::Success {
                    let mut time_penalty = (now - self.level_start).num_seconds();
                    self.level_scores.add_user_evaluation(
                        user,
                        time_penalty,
                        program.rules.len(),
                    );
                }
                self.submissions.push(UserSubmission {
                    user: user.to_owned(),
                    details,
                });
                contract::SubmitResponse::Ok
            }
            Err(RateLimitExceeded) => {
                contract::SubmitResponse::RateLimitExceeded
            }
        }
    }

    pub fn all_submissions(&self) -> contract::Submissions {
        contract::Submissions {
            submissions: self.submissions
                .iter()
                .enumerate()
                .map(|(id, sub)| contract::Submission {
                    id: id as u64,
                    user: sub.user.clone(),
                })
                .collect(),
            level_closed: self.is_level_closed,
            level: self.current_level.state.clone(),
        }
    }

    pub fn submission_details(&self, id: u64) -> Option<contract::SubmissionDetails> {
        self.submissions.get(id as usize).map(|s| &s.details).cloned()
    }
}

fn empty_level() -> contract::Level {
    contract::Level {
        state: contract::LevelState {
            cells: vec![vec![contract::Cell::Wall]],
            objects: Vec::new(),
        },
        ghost_program: contract::Program {
            rules: Vec::new(),
        },
    }
}
