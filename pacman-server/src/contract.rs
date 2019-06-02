#![allow(unused)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Program {
    rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Rule {
    current_state: Option<RuleState>,
    up: Option<RuleCell>,
    down: Option<RuleCell>,
    left: Option<RuleCell>,
    right: Option<RuleCell>,
    berry: Option<RuleBerry>,
    next_move: Move,
    next_state: RuleState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum RuleState {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum RuleCell {
    Wall,
    Empty,
    Ghost,
    Berry,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum RuleBerry {
    Taken,
    NotTaken,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum Move {
    Up,
    Down,
    Left,
    Right,
    Wait,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Submit {
    program: Program,
    user: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Submissions {
    submissions: Vec<Submission>,
    level_closed: bool,
    level: LevelState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Submission {
    id: u64,
    user: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubmissionDetails {
    initial_state: LevelState,
    steps: Vec<Step>,
    outcome: Outcome,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum Outcome {
    Success,
    Fail,
    OutOfMoves,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LevelState {
    cells: Vec<Vec<Cell>>,
    ojects: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum Cell {
    Wall,
    Empty,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Step {
    objects: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Object {
    id: u64,
    x: u64,
    y: u64,
    alive: bool,
    kind: ObjectKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum ObjectKind {
    Ghost,
    Berry,
    Pacman,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Scoreboards {
    scoreboards: Vec<Scoreboard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Scoreboard {
    title: String,
    entries: Vec<ScoreboardEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScoreboardEntry {
    user: String,
    solved: u64,
    tie_breaker: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum SubmitResponse {
    Ok,
    RateLimitExceeded,
    LevelClosed,
    Unauthorized,
}
