use crate::contract::{
    Cell,
    DeathState,
    Level,
    LevelState,
    Move,
    Object,
    ObjectKind,
    Outcome,
    Program,
    RuleBerry,
    RuleCell,
    RuleState,
    Step,
    SubmissionDetails,
};

pub fn evaluate_program(
    level: &Level,
    program: &Program,
    move_limit: u64,
) -> SubmissionDetails {
    let initial_state = level.state.clone();
    let mut steps = Vec::new();
    let mut steps_taken = 0;
    let mut evaluator = Evaluator {
        cells: &level.state.cells,
        objects: level.state.objects
            .iter()
            .map(|obj| ObjectInfo {
                obj: obj.clone(),
                state: RuleState::A,
                next_row: obj.row as usize,
                next_col: obj.col as usize,
            })
            .collect(),
        pacman_program: &program,
        ghost_program: &level.ghost_program,
    };

    let outcome = loop {
        if steps_taken == move_limit {
            break Outcome::OutOfMoves;
        }
        if evaluator.is_victory() {
            break Outcome::Success;
        }
        if evaluator.is_defeat() {
            break Outcome::Fail;
        }
        steps_taken += 1;
        evaluator.cleanup_objects();
        evaluator.prepare_moves();
        steps.push(evaluator.get_step());
        evaluator.finish_moves();
    };

    SubmissionDetails { initial_state, steps, outcome }
}

struct ObjectInfo {
    obj: Object,
    state: RuleState,
    next_row: usize,
    next_col: usize,
}

impl ObjectInfo {
    fn pos(&self) -> (usize, usize) {
        (self.obj.row as usize, self.obj.col as usize)
    }

    fn next_pos(&self) -> (usize, usize) {
        (self.next_row, self.next_col)
    }
}

struct Evaluator<'a> {
    cells: &'a [Vec<Cell>],
    objects: Vec<ObjectInfo>,
    pacman_program: &'a Program,
    ghost_program: &'a Program,
}

impl<'a> Evaluator<'a> {
    fn get_step(&self) -> Step {
        Step {
            objects: self.objects
                .iter()
                .map(|obj| obj.obj.clone())
                .collect()
        }
    }

    fn cleanup_objects(&mut self) {
        self.objects.retain(|o| o.obj.state == DeathState::Alive);
    }

    fn prepare_moves(&mut self) {
        for i in 0..self.objects.len() {
            self.objects[i].obj.current_move = Move::Wait;
            let program = match self.objects[i].obj.kind {
                ObjectKind::Pacman => self.pacman_program,
                ObjectKind::Ghost => self.ghost_program,
                ObjectKind::Berry => continue,
            };
            let (next_state, next_move) = self.pick_move(
                program,
                self.objects[i].state,
                self.objects[i].obj.row as usize,
                self.objects[i].obj.col as usize,
            );
            self.objects[i].state = next_state;
            self.objects[i].obj.current_move = next_move;
            self.objects[i].obj.intended_move = next_move;
            let (blocked, row, col) = self.next_pos(&self.objects[i]);
            self.objects[i].next_row = row;
            self.objects[i].next_col = col;
            if blocked {
                self.objects[i].obj.current_move = Move::Wait;
            }
        }
        let is_berry_taken = self.is_berry_taken();
        // check if pacman finished in a cell with ghost
        for i in 0..self.objects.len() {
            if self.objects[i].obj.kind != ObjectKind::Pacman {
                continue;
            }
            let pacman_pos = self.objects[i].next_pos();
            for j in 0..self.objects.len() {
                if self.objects[j].obj.kind != ObjectKind::Ghost {
                    continue;
                }
                let ghost_pos = self.objects[j].next_pos();
                if pacman_pos == ghost_pos {
                    if is_berry_taken {
                        self.objects[j].obj.state = DeathState::DiesAtEnd;
                    } else {
                        self.objects[i].obj.state = DeathState::DiesAtEnd;
                    }
                }
            }
        }
        // check if pacman walked into a ghost that walked into pacman
        for i in 0..self.objects.len() {
            if self.objects[i].obj.kind != ObjectKind::Pacman {
                continue;
            }
            let old_pacman_pos = self.objects[i].pos();
            let pacman_pos = self.objects[i].next_pos();
            for j in 0..self.objects.len() {
                if self.objects[j].obj.kind != ObjectKind::Ghost {
                    continue;
                }
                let old_ghost_pos = self.objects[j].pos();
                let ghost_pos = self.objects[j].next_pos();
                if pacman_pos == old_ghost_pos && old_pacman_pos == ghost_pos {
                    if is_berry_taken {
                        self.objects[j].obj.state = DeathState::DiesInMiddle;
                    } else {
                        self.objects[i].obj.state = DeathState::DiesInMiddle;
                    }
                }
            }
        }
        // check if alive pacman ate a berry
        for i in 0..self.objects.len() {
            if self.objects[i].obj.kind != ObjectKind::Pacman {
                continue;
            }
            if self.objects[i].obj.state != DeathState::Alive {
                continue;
            }
            let pacman_pos = self.objects[i].next_pos();
            for j in 0..self.objects.len() {
                if self.objects[j].obj.kind != ObjectKind::Berry {
                    continue;
                }
                if pacman_pos == self.objects[j].pos() {
                    self.objects[j].obj.state = DeathState::DiesAtEnd;
                    break;
                }
            }
        }
    }

    fn finish_moves(&mut self) {
        for obj in &mut self.objects {
            obj.obj.row = obj.next_row as u64;
            obj.obj.col = obj.next_col as u64;
        }
    }

    fn pick_move(&self, program: &Program, state: RuleState, row: usize, col: usize) -> (RuleState, Move) {
        for rule in &program.rules {
            if let Some(expected_state) = rule.current_state {
                if expected_state != state {
                    continue;
                }
            }
            if let Some(expected) = rule.up {
                let actual = self.get_cell(row.wrapping_sub(1), col);
                if expected != actual {
                    continue;
                }
            }
            if let Some(expected) = rule.down {
                let actual = self.get_cell(row.wrapping_add(1), col);
                if expected != actual {
                    continue;
                }
            }
            if let Some(expected) = rule.left {
                let actual = self.get_cell(row, col.wrapping_sub(1));
                if expected != actual {
                    continue;
                }
            }
            if let Some(expected) = rule.right {
                let actual = self.get_cell(row, col.wrapping_add(1));
                if expected != actual {
                    continue;
                }
            }
            match rule.berry {
                Some(RuleBerry::Taken) if !self.is_berry_taken() => continue,
                Some(RuleBerry::NotTaken) if self.is_berry_taken() => continue,
                _ => {}
            }
            return (rule.next_state, rule.next_move);
        }
        (state, Move::Wait)
    }

    fn get_cell(&self, row: usize, col: usize) -> RuleCell {
        let static_cell = self.cells
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(Cell::Wall);
        let mut obj = None;
        for object in &self.objects {
            if object.obj.row == row as u64 && object.obj.col == col as u64 {
                if let Some(ref mut o) = obj {
                    *o = std::cmp::max(*o, object.obj.kind);
                } else {
                    obj = Some(object.obj.kind);
                }
            }
        }
        match (obj, static_cell) {
            (Some(ObjectKind::Pacman), _) => RuleCell::Pacman,
            (Some(ObjectKind::Ghost), _) => RuleCell::Ghost,
            (Some(ObjectKind::Berry), _) => RuleCell::Berry,
            (None, Cell::Wall) => RuleCell::Wall,
            (None, Cell::Empty) => RuleCell::Empty,
        }
    }

    fn is_berry_taken(&self) -> bool {
        self.objects.iter().all(|o| o.obj.kind != ObjectKind::Berry)
    }

    fn is_victory(&self) -> bool {
        self.objects.len() == 1 && self.objects[0].obj.kind == ObjectKind::Pacman
    }

    fn is_defeat(&self) -> bool {
        self.objects.iter().all(|o| o.obj.kind != ObjectKind::Pacman)
    }

    fn can_pass(&self, row: usize, col: usize) -> bool {
        let static_cell = self.cells
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(Cell::Wall);
        match static_cell {
            Cell::Empty => true,
            Cell::Wall => false,
        }
    }

    fn next_pos(&self, obj: &ObjectInfo) -> (bool, usize, usize) {
        let row = obj.obj.row as usize;
        let col = obj.obj.col as usize;
        let (new_row, new_col) = match obj.obj.current_move {
            Move::Up => (row.wrapping_sub(1), col),
            Move::Down => (row.wrapping_add(1), col),
            Move::Left => (row, col.wrapping_sub(1)),
            Move::Right => (row, col.wrapping_add(1)),
            Move::Wait => (row, col),
        };
        if self.can_pass(new_row, new_col) {
            (false, new_row, new_col)
        } else {
            (true, row, col)
        }
    }
}
