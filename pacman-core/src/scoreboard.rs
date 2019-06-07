use std::cmp::Reverse;
use std::collections::HashMap;
use crate::contract;

#[derive(Debug, Clone, Default)]
pub struct Scoreboard {
    user_scores: HashMap<String, UserScore>,
}

impl Scoreboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_user_evaluation(&mut self, user: &str, time: i64, size: usize) {
        if !self.user_scores.contains_key(user) {
            self.user_scores.insert(user.to_owned(), UserScore {
                solved_levels: 0,
                time_penalty: 0,
                size_penalty: 0,
            });
        }
        let score = self.user_scores.get_mut(user).unwrap();
        if score.solved_levels == 0 {
            score.solved_levels = 1;
            score.time_penalty = time;
            score.size_penalty = size;
        } else {
            score.time_penalty = std::cmp::min(score.time_penalty, time);
            score.size_penalty = std::cmp::min(score.size_penalty, size);
        }
    }

    pub fn add_level_scores(&mut self, level_scores: &Scoreboard) {
        for (user, score) in &level_scores.user_scores {
            if let Some(total) = self.user_scores.get_mut(user) {
                total.solved_levels += score.solved_levels;
                total.time_penalty += score.time_penalty;
                total.size_penalty += score.size_penalty; 
            } else {
                self.user_scores.insert(user.to_owned(), score.clone());
            }
        }
    }

    pub fn to_contract_with_time(&self, title: &str) -> contract::Scoreboard {
        let mut entries = self.user_scores
            .iter()
            .map(|(user, score)| (
                user.clone(),
                score.solved_levels,
                score.time_penalty,
            ))
            .collect::<Vec<_>>();
        entries.sort_by(|(user1, score1, penalty1), (user2, score2, penalty2)|
            (score1, Reverse(penalty1), user1).cmp(&(score2, Reverse(penalty2), user2)).reverse()
        );
        contract::Scoreboard {
            title: title.to_owned(),
            entries: entries
                .into_iter()
                .map(|(user, solved, penalty)| contract::ScoreboardEntry {
                    user,
                    solved,
                    tie_breaker: format_time_penalty(penalty),
                })
                .collect(),
        }
    }

    pub fn to_contract_with_size(&self, title: &str) -> contract::Scoreboard {
        let mut entries = self.user_scores
            .iter()
            .map(|(user, score)| (
                user.clone(),
                score.solved_levels,
                score.size_penalty,
            ))
            .collect::<Vec<_>>();
        entries.sort_by(|(user1, score1, penalty1), (user2, score2, penalty2)|
            (score1, Reverse(penalty1), user1).cmp(&(score2, Reverse(penalty2), user2)).reverse()
        );
        contract::Scoreboard {
            title: title.to_owned(),
            entries: entries
                .into_iter()
                .map(|(user, solved, penalty)| contract::ScoreboardEntry {
                    user,
                    solved,
                    tie_breaker: penalty.to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserScore {
    solved_levels: u64,
    time_penalty: i64,
    size_penalty: usize,
}

fn format_time_penalty(time: i64) -> String {
    if time >= 0 {
        format!("{}:{:>02}", time / 60, time % 60)
    } else {
        // pls no
        format!("-{}:{:>02}", -time / 60, -time % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tests() {
        fn check(time: i64, expected: &str) {
            assert_eq!(&format_time_penalty(time), expected);
        }
        check(0, "0:00");
        check(42, "0:42");
        check(60, "1:00");
        check(72, "1:12");
        check(153, "2:33");
        check(1801, "30:01");
        check(3600, "60:00");
        check(5438, "90:38");
    }
}
