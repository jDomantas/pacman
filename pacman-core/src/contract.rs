use serde::{Deserialize, Serialize};
use chrono::Duration;

/// Makes everything public, adds serde attributes, derives Debug and Clone.
macro_rules! contract {
    () => {};
    ($(#[$attr:meta])?
    struct $name:ident {
        $($field:ident: $ty:ty),* $(,)?
    }
    $($rest:tt)*) => {
        #[derive(Debug, Serialize, Deserialize, Clone)]
        #[serde(rename_all = "camelCase")]
        $(#[$attr])?
        pub struct $name {
            $(pub $field: $ty),*
        }

        contract! { $($rest)* }
    };
    ($(#[$attr:meta])?
    enum $name:ident {
        $($case:ident),* $(,)?
    }
    $($rest:tt)*) => {
        #[derive(Debug, Serialize, Deserialize, Clone)]
        #[serde(rename_all = "camelCase")]
        $(#[$attr])?
        pub enum $name {
            $($case),*
        }

        contract! { $($rest)* }
    };
}

contract! {
    struct Program {
        rules: Vec<Rule>,
    }

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

    #[derive(PartialEq, Eq, Copy)]
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

    #[derive(PartialEq, Eq, Copy)]
    enum RuleCell {
        Wall,
        Empty,
        Ghost,
        Berry,
        Pacman,
    }

    #[derive(PartialEq, Eq, Copy)]
    enum RuleBerry {
        Taken,
        NotTaken,
    }

    #[derive(Copy)]
    enum Move {
        Up,
        Down,
        Left,
        Right,
        Wait,
    }

    struct Submit {
        user: Option<String>,
        password: Option<String>,
        program: Program,
    }

    struct Submissions {
        submissions: Vec<Submission>,
        level_closed: bool,
        level: LevelState,
    }

    struct Submission {
        id: u64,
        user: String,
    }

    struct SubmissionDetails {
        initial_state: LevelState,
        steps: Vec<Step>,
        outcome: Outcome,
    }

    #[derive(PartialEq, Eq, Copy)]
    enum Outcome {
        Success,
        Fail,
        OutOfMoves,
    }

    struct LevelState {
        cells: Vec<Vec<Cell>>,
        objects: Vec<Object>,
    }

    #[derive(Copy)]
    enum Cell {
        Wall,
        Empty,
    }

    struct Step {
        objects: Vec<Object>,
    }

    struct Object {
        id: u64,
        row: u64,
        col: u64,
        current_move: Move,
        intended_move: Move,
        state: DeathState,
        kind: ObjectKind,
    }

    #[derive(PartialEq, Eq, Copy)]
    enum DeathState {
        Alive,
        DiesAtEnd,
        DiesInMiddle,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Copy)]
    enum ObjectKind {
        Berry,
        Ghost,
        Pacman,
    }

    struct Scoreboards {
        scoreboards: Vec<Scoreboard>,
    }

    struct Scoreboard {
        title: String,
        entries: Vec<ScoreboardEntry>,
    }

    struct ScoreboardEntry {
        user: String,
        solved: u64,
        tie_breaker: String,
    }

    #[derive(Copy)]
    enum SubmitResponse {
        Ok,
        RateLimitExceeded,
        LevelClosed,
        Unauthorized,
    }

    struct Level {
        state: LevelState,
        ghost_program: Program,
    }

    struct SetLevel {
        admin_token: String,
        level: Level,
    }

    struct SetLevelState {
        admin_token: String,
        is_closed: bool,
    }

    struct Reset {
        admin_token: String
    }

    struct RateLimit {
        admin_token: String,
        user: String,
        count: u32,
        window: u32,
    }

    struct Authenticate {
        user: String,
        password: String,
    }
}
