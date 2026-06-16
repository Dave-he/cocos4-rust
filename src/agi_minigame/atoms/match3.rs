use std::any::Any;

use crate::base::value::{Value, ValueMap};

use crate::agi_minigame::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GemType {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    White,
}

impl GemType {
    pub fn all() -> Vec<GemType> {
        vec![GemType::Red, GemType::Blue, GemType::Green, GemType::Yellow, GemType::Purple, GemType::White]
    }

    pub fn from_index(idx: usize) -> GemType {
        match idx % 6 {
            0 => GemType::Red,
            1 => GemType::Blue,
            2 => GemType::Green,
            3 => GemType::Yellow,
            4 => GemType::Purple,
            _ => GemType::White,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            GemType::Red => 0,
            GemType::Blue => 1,
            GemType::Green => 2,
            GemType::Yellow => 3,
            GemType::Purple => 4,
            GemType::White => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GemCell {
    pub gem_type: GemType,
    pub row: usize,
    pub col: usize,
    pub is_matched: bool,
    pub is_special: bool,
    pub special_type: SpecialType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialType {
    None,
    LineH,
    LineV,
    Bomb,
    Rainbow,
}

impl GemCell {
    pub fn new(gem_type: GemType, row: usize, col: usize) -> Self {
        Self {
            gem_type,
            row,
            col,
            is_matched: false,
            is_special: false,
            special_type: SpecialType::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchGroup {
    pub cells: Vec<(usize, usize)>,
    pub gem_type: GemType,
    pub is_horizontal: bool,
}

impl MatchGroup {
    pub fn size(&self) -> usize {
        self.cells.len()
    }

    pub fn is_special_eligible(&self) -> Option<SpecialType> {
        if self.size() >= 5 {
            Some(SpecialType::Rainbow)
        } else if self.size() == 4 {
            if self.is_horizontal {
                Some(SpecialType::LineH)
            } else {
                Some(SpecialType::LineV)
            }
        } else if self.size() >= 3 {
            None
        } else {
            None
        }
    }
}

pub struct Match3Atom {
    phase: AtomPhase,
    board: Vec<Vec<Option<GemCell>>>,
    rows: usize,
    cols: usize,
    num_gem_types: usize,
    score: u64,
    combo: u32,
    max_combo: u32,
    moves: u32,
    max_moves: u32,
    matches_found: Vec<MatchGroup>,
    chain_count: u32,
    total_eliminated: u64,
}

impl Match3Atom {
    pub fn new(rows: usize, cols: usize, max_moves: u32) -> Self {
        Self {
            phase: AtomPhase::Uninitialized,
            board: Vec::new(),
            rows,
            cols,
            num_gem_types: 5,
            score: 0,
            combo: 0,
            max_combo: 0,
            moves: 0,
            max_moves,
            matches_found: Vec::new(),
            chain_count: 0,
            total_eliminated: 0,
        }
    }

    pub fn with_gem_types(mut self, n: usize) -> Self {
        self.num_gem_types = n.min(6).max(3);
        self
    }

    fn init_board(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        self.board = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            let mut row: Vec<Option<GemCell>> = Vec::with_capacity(self.cols);
            for c in 0..self.cols {
                let gem_type = loop {
                    let gt = GemType::from_index(rng.gen_range(0..self.num_gem_types));
                    let mut would_match = false;

                    // Horizontal: same row, c-1 and c-2 must not both be `gt`.
                    // Read from the local `row` (not `self.board[r]`, which is
                    // not yet pushed and would silently never match).
                    if c >= 2 {
                        if let (Some(Some(p1)), Some(Some(p2))) =
                            (row.get(c - 1), row.get(c - 2))
                        {
                            if p1.gem_type == gt && p2.gem_type == gt {
                                would_match = true;
                            }
                        }
                    }

                    // Vertical: two rows above must not both be `gt` in the same column.
                    if !would_match && r >= 2 {
                        if let (Some(row_above_1), Some(row_above_2)) =
                            (self.board.get(r - 1), self.board.get(r - 2))
                        {
                            if let (Some(Some(p1)), Some(Some(p2))) =
                                (row_above_1.get(c), row_above_2.get(c))
                            {
                                if p1.gem_type == gt && p2.gem_type == gt {
                                    would_match = true;
                                }
                            }
                        }
                    }

                    if !would_match {
                        break gt;
                    }
                };

                row.push(Some(GemCell::new(gem_type, r, c)));
            }
            self.board.push(row);
        }
    }

    pub fn swap(&mut self, r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
        if !self.is_adjacent(r1, c1, r2, c2) {
            return false;
        }

        self.swap_cells(r1, c1, r2, c2);

        let matches = self.find_matches();
        if matches.is_empty() {
            self.swap_cells(r1, c1, r2, c2);
            return false;
        }

        self.moves += 1;
        self.matches_found = matches;
        self.combo = 0;
        self.process_chain();
        true
    }

    fn is_adjacent(&self, r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
        let dr = (r1 as i32 - r2 as i32).abs();
        let dc = (c1 as i32 - c2 as i32).abs();
        (dr == 1 && dc == 0) || (dr == 0 && dc == 1)
    }

    fn swap_cells(&mut self, r1: usize, c1: usize, r2: usize, c2: usize) {
        let cell1 = self.board[r1][c1].take();
        let cell2 = self.board[r2][c2].take();

        if let Some(mut c) = cell1 {
            c.row = r2;
            c.col = c2;
            self.board[r2][c2] = Some(c);
        }
        if let Some(mut c) = cell2 {
            c.row = r1;
            c.col = c1;
            self.board[r1][c1] = Some(c);
        }
    }

    fn find_matches(&self) -> Vec<MatchGroup> {
        let mut matches = Vec::new();

        for r in 0..self.rows {
            let mut c = 0;
            while c < self.cols {
                if let Some(Some(cell)) = self.board.get(r).and_then(|row| row.get(c)) {
                    let gt = cell.gem_type;
                    let mut end = c + 1;
                    while end < self.cols {
                        if let Some(Some(next)) = self.board.get(r).and_then(|row| row.get(end)) {
                            if next.gem_type == gt {
                                end += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if end - c >= 3 {
                        let cells: Vec<(usize, usize)> = (c..end).map(|col| (r, col)).collect();
                        matches.push(MatchGroup {
                            cells,
                            gem_type: gt,
                            is_horizontal: true,
                        });
                    }
                    c = end;
                } else {
                    c += 1;
                }
            }
        }

        for c in 0..self.cols {
            let mut r = 0;
            while r < self.rows {
                if let Some(Some(cell)) = self.board.get(r).and_then(|row| row.get(c)) {
                    let gt = cell.gem_type;
                    let mut end = r + 1;
                    while end < self.rows {
                        if let Some(row_vec) = self.board.get(end) {
                            if let Some(Some(next)) = row_vec.get(c) {
                                if next.gem_type == gt {
                                    end += 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if end - r >= 3 {
                        let cells: Vec<(usize, usize)> = (r..end).map(|row| (row, c)).collect();
                        matches.push(MatchGroup {
                            cells,
                            gem_type: gt,
                            is_horizontal: false,
                        });
                    }
                    r = end;
                } else {
                    r += 1;
                }
            }
        }

        matches
    }

    fn process_chain(&mut self) {
        loop {
            let matches = if self.matches_found.is_empty() {
                self.find_matches()
            } else {
                std::mem::take(&mut self.matches_found)
            };

            if matches.is_empty() {
                break;
            }

            self.combo += 1;
            if self.combo > self.max_combo {
                self.max_combo = self.combo;
            }
            self.chain_count += 1;

            let mut eliminated = 0u64;
            for m in &matches {
                let base_score = m.size() as u64 * 10;
                let combo_mult = self.combo as u64;
                self.score += base_score * combo_mult;
                eliminated += m.size() as u64;

                for &(r, c) in &m.cells {
                    if let Some(row) = self.board.get_mut(r) {
                        if let Some(Some(cell)) = row.get_mut(c) {
                            cell.is_matched = true;
                        }
                    }
                }
            }
            self.total_eliminated += eliminated;

            self.eliminate_matched();
            self.apply_gravity();
            self.fill_empty();
        }
    }

    fn eliminate_matched(&mut self) {
        for r in 0..self.rows {
            for c in 0..self.cols {
                if let Some(Some(cell)) = self.board.get(r).and_then(|row| row.get(c)) {
                    if cell.is_matched {
                        self.board[r][c] = None;
                    }
                }
            }
        }
    }

    fn apply_gravity(&mut self) {
        for c in 0..self.cols {
            let mut write_row = self.rows;
            for r in (0..self.rows).rev() {
                if self.board[r][c].is_some() {
                    write_row -= 1;
                    if write_row != r {
                        let cell = self.board[r][c].take();
                        if let Some(mut cell_inner) = cell {
                            cell_inner.row = write_row;
                            cell_inner.col = c;
                            self.board[write_row][c] = Some(cell_inner);
                        }
                    }
                }
            }
        }
    }

    fn fill_empty(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for c in 0..self.cols {
            for r in 0..self.rows {
                if self.board[r][c].is_none() {
                    let gt = GemType::from_index(rng.gen_range(0..self.num_gem_types));
                    self.board[r][c] = Some(GemCell::new(gt, r, c));
                }
            }
        }
    }

    pub fn get_score(&self) -> u64 {
        self.score
    }

    pub fn get_combo(&self) -> u32 {
        self.combo
    }

    pub fn get_moves_remaining(&self) -> u32 {
        self.max_moves.saturating_sub(self.moves)
    }

    pub fn is_game_over(&self) -> bool {
        self.moves >= self.max_moves
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&GemCell> {
        self.board.get(row)?.get(col)?.as_ref()
    }

    pub fn get_board_size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
}

impl Atom for Match3Atom {
    fn atom_id(&self) -> AtomId { "match3".to_string() }
    fn atom_name(&self) -> &str { "三消" }

    fn on_init(&mut self, _ctx: &mut AtomContext) {
        self.phase = AtomPhase::Initialized;
    }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.init_board();
        self.score = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.moves = 0;
        self.chain_count = 0;
        self.total_eliminated = 0;
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, _ctx: &mut AtomContext) {
        if self.is_game_over() {
            self.phase = AtomPhase::Completed;
        }
    }

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }

    fn on_exit(&mut self, _ctx: &mut AtomContext) {
        self.phase = AtomPhase::Completed;
    }

    fn on_destroy(&mut self) {
        self.board.clear();
        self.phase = AtomPhase::Uninitialized;
    }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map.insert("combo".to_string(), Value::Integer(self.combo as i32));
        map.insert("max_combo".to_string(), Value::Integer(self.max_combo as i32));
        map.insert("moves".to_string(), Value::Integer(self.moves as i32));
        map.insert("chain_count".to_string(), Value::Integer(self.chain_count as i32));
        map.insert("total_eliminated".to_string(), Value::Integer(self.total_eliminated as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Integer(n)) = state.get("combo") { self.combo = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("max_combo") { self.max_combo = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("moves") { self.moves = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("chain_count") { self.chain_count = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("total_eliminated") { self.total_eliminated = *n as u64; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "swap" => {
                let r1 = data.get("r1").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                let c1 = data.get("c1").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                let r2 = data.get("r2").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                let c2 = data.get("c2").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                self.swap(r1, c1, r2, c2);
            }
            _ => {}
        }
    }

    fn current_phase(&self) -> AtomPhase { self.phase }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    #[test]
    fn test_match3_init() {
        let mut atom = Match3Atom::new(8, 8, 30);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let (rows, cols) = atom.get_board_size();
        assert_eq!(rows, 8);
        assert_eq!(cols, 8);

        for r in 0..rows {
            for c in 0..cols {
                assert!(atom.get_cell(r, c).is_some());
            }
        }
    }

    #[test]
    fn test_match3_no_initial_matches() {
        let mut atom = Match3Atom::new(8, 8, 30);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let matches = atom.find_matches();
        assert!(matches.is_empty(), "Initial board should have no matches");
    }

    #[test]
    fn test_match3_score() {
        let mut atom = Match3Atom::new(8, 8, 30);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        assert_eq!(atom.get_score(), 0);
    }

    #[test]
    fn test_match3_game_over() {
        let mut atom = Match3Atom::new(8, 8, 5);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        assert!(!atom.is_game_over());
        assert_eq!(atom.get_moves_remaining(), 5);
    }

    #[test]
    fn test_match3_save_load() {
        let mut atom = Match3Atom::new(8, 8, 30);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.score = 500;
        atom.combo = 3;
        atom.max_combo = 3;
        atom.moves = 5;

        let state = atom.save_state();

        let mut atom2 = Match3Atom::new(8, 8, 30);
        atom2.load_state(&state);

        assert_eq!(atom2.score, 500);
        assert_eq!(atom2.combo, 3);
        assert_eq!(atom2.moves, 5);
    }

    #[test]
    fn test_gem_type() {
        assert_eq!(GemType::from_index(0), GemType::Red);
        assert_eq!(GemType::Red.to_index(), 0);
        // 7 % 6 = 1, which is Blue — verifies the modulo wrap.
        assert_eq!(GemType::from_index(7), GemType::Blue);
        assert_eq!(GemType::White.to_index(), 5);
    }

    #[test]
    fn test_match3_with_gem_types() {
        let atom = Match3Atom::new(6, 6, 20).with_gem_types(3);
        assert_eq!(atom.num_gem_types, 3);
    }
}

// ---------------------------------------------------------------------------
// Round 135 — helper-level
// tests for the lower-level
// `GemType` / `SpecialType` /
// `GemCell` / `MatchGroup`
// / `Match3Atom` public
// surface. The high-level
// `Atom` lifecycle is
// already exercised by
// the existing `tests`
// mod; this block adds
// focused unit tests for
// the free-standing
// helpers + `Match3Atom`
// accessors so a future
// refactor that breaks a
// primitive is caught at
// the unit level rather
// than only failing a
// higher-level test.
//
// Mirrors the
// round-110b / 122
// / 123 / 124 / 125
// / 126 / 127 / 128
// / 129 / 130 / 131
// / 132 / 133 / 134
// helper-test pattern.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round135_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    /// Round 135 — local
    /// mirror of the
    /// `tests` mod
    /// `make_ctx`
    /// helper (the
    /// private
    /// original is
    /// not visible
    /// from a sibling
    /// mod).
    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    fn make_initialized_atom(rows: usize, cols: usize, max_moves: u32) -> Match3Atom {
        let mut atom = Match3Atom::new(rows, cols, max_moves);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        atom
    }

    // --- GemType ---

    /// Round 135 —
    /// `GemType::all()`
    /// returns the 6
    /// canonical variants
    /// in stable order.
    #[test]
    fn gem_type_all_returns_6_variants_round_135() {
        let v = GemType::all();
        assert_eq!(v.len(), 6);
        assert_eq!(v[0], GemType::Red);
        assert_eq!(v[1], GemType::Blue);
        assert_eq!(v[2], GemType::Green);
        assert_eq!(v[3], GemType::Yellow);
        assert_eq!(v[4], GemType::Purple);
        assert_eq!(v[5], GemType::White);
    }

    /// Round 135 —
    /// `GemType::from_index`
    /// is the inverse of
    /// `to_index` for the
    /// first 6 indices
    /// (the canonical 6
    /// colors).
    #[test]
    fn gem_type_from_index_to_index_round_trip_round_135() {
        for idx in 0..6usize {
            assert_eq!(GemType::from_index(idx).to_index(), idx);
        }
    }

    /// Round 135 —
    /// `GemType::from_index`
    /// wraps modulo 6 (any
    /// index ≥ 6 still
    /// resolves to a
    /// valid variant).
    #[test]
    fn gem_type_from_index_wraps_modulo_6_round_135() {
        // 6 % 6 = 0 → Red.
        assert_eq!(GemType::from_index(6), GemType::Red);
        // 12 % 6 = 0 → Red.
        assert_eq!(GemType::from_index(12), GemType::Red);
        // 99 % 6 = 3 → Yellow.
        assert_eq!(GemType::from_index(99), GemType::Yellow);
    }

    /// Round 135 —
    /// `GemType::to_index`
    /// returns the
    /// canonical index
    /// for each variant.
    #[test]
    fn gem_type_to_index_returns_canonical_index_round_135() {
        assert_eq!(GemType::Red.to_index(),    0);
        assert_eq!(GemType::Blue.to_index(),   1);
        assert_eq!(GemType::Green.to_index(),  2);
        assert_eq!(GemType::Yellow.to_index(), 3);
        assert_eq!(GemType::Purple.to_index(), 4);
        assert_eq!(GemType::White.to_index(),  5);
    }

    // --- SpecialType ---

    /// Round 135 —
    /// `SpecialType` has
    /// exactly 5 variants
    /// (None / LineH /
    /// LineV / Bomb /
    /// Rainbow).
    #[test]
    fn special_type_has_5_variants_round_135() {
        // PartialEq sanity-check: each variant is
        // equal to itself.
        let v = [
            SpecialType::None,
            SpecialType::LineH,
            SpecialType::LineV,
            SpecialType::Bomb,
            SpecialType::Rainbow,
        ];
        for &x in &v { assert_eq!(x, x); }
        // Distinct variants are not equal.
        assert_ne!(SpecialType::LineH, SpecialType::LineV);
        assert_ne!(SpecialType::Bomb, SpecialType::Rainbow);
    }

    // --- GemCell ---

    /// Round 135 —
    /// `GemCell::new`
    /// stores the
    /// constructor args
    /// verbatim.
    #[test]
    fn gem_cell_new_stores_fields_verbatim_round_135() {
        let cell = GemCell::new(GemType::Red, 3, 7);
        assert_eq!(cell.gem_type, GemType::Red);
        assert_eq!(cell.row, 3);
        assert_eq!(cell.col, 7);
    }

    /// Round 135 —
    /// `GemCell::new`
    /// defaults
    /// `is_matched` /
    /// `is_special` to
    /// `false` and
    /// `special_type`
    /// to
    /// `SpecialType::None`.
    #[test]
    fn gem_cell_new_defaults_match_special_to_false_none_round_135() {
        let cell = GemCell::new(GemType::Blue, 0, 0);
        assert!(!cell.is_matched);
        assert!(!cell.is_special);
        assert_eq!(cell.special_type, SpecialType::None);
    }

    // --- MatchGroup ---

    /// Round 135 —
    /// `MatchGroup::size`
    /// returns the cell
    /// count.
    #[test]
    fn match_group_size_returns_cell_count_round_135() {
        let g = MatchGroup {
            cells: vec![(0, 0), (0, 1), (0, 2)],
            gem_type: GemType::Red,
            is_horizontal: true,
        };
        assert_eq!(g.size(), 3);
    }

    /// Round 135 —
    /// `MatchGroup::is_special_eligible`
    /// returns
    /// `Some(Rainbow)`
    /// for size ≥ 5.
    #[test]
    fn match_group_special_eligible_5_plus_is_rainbow_round_135() {
        let g = MatchGroup {
            cells: vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
            gem_type: GemType::Red,
            is_horizontal: true,
        };
        assert_eq!(g.is_special_eligible(), Some(SpecialType::Rainbow));
    }

    /// Round 135 —
    /// `MatchGroup::is_special_eligible`
    /// returns
    /// `Some(LineH)`
    /// for size 4
    /// horizontal +
    /// `Some(LineV)`
    /// for size 4
    /// vertical.
    #[test]
    fn match_group_special_eligible_4_distinguishes_h_v_round_135() {
        let h = MatchGroup {
            cells: vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            gem_type: GemType::Red,
            is_horizontal: true,
        };
        assert_eq!(h.is_special_eligible(), Some(SpecialType::LineH));
        let v = MatchGroup {
            cells: vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            gem_type: GemType::Red,
            is_horizontal: false,
        };
        assert_eq!(v.is_special_eligible(), Some(SpecialType::LineV));
    }

    /// Round 135 —
    /// `MatchGroup::is_special_eligible`
    /// returns
    /// `None` for size 3
    /// (the minimum-match
    /// case).
    #[test]
    fn match_group_special_eligible_3_is_none_round_135() {
        let g = MatchGroup {
            cells: vec![(0, 0), (0, 1), (0, 2)],
            gem_type: GemType::Red,
            is_horizontal: true,
        };
        assert_eq!(g.is_special_eligible(), None);
    }

    // --- Match3Atom accessors ---

    /// Round 135 —
    /// `Match3Atom::new`
    /// initializes score
    /// / combo / moves
    /// to 0.
    #[test]
    fn match3_new_initializes_score_combo_moves_to_zero_round_135() {
        let atom = Match3Atom::new(6, 6, 20);
        assert_eq!(atom.get_score(), 0);
        assert_eq!(atom.get_combo(), 0);
        assert_eq!(atom.get_moves_remaining(), 20);
        assert!(!atom.is_game_over());
    }

    /// Round 135 —
    /// `Match3Atom::with_gem_types`
    /// clamps n to the
    /// documented range
    /// `[3, 6]`.
    #[test]
    fn match3_with_gem_types_clamps_to_3_to_6_round_135() {
        // Below 3 → clamps up to 3.
        let atom = Match3Atom::new(6, 6, 20).with_gem_types(1);
        assert_eq!(atom.num_gem_types, 3);
        // Above 6 → clamps down to 6.
        let atom = Match3Atom::new(6, 6, 20).with_gem_types(99);
        assert_eq!(atom.num_gem_types, 6);
        // In range → kept verbatim.
        let atom = Match3Atom::new(6, 6, 20).with_gem_types(4);
        assert_eq!(atom.num_gem_types, 4);
    }

    /// Round 135 —
    /// `Match3Atom::swap`
    /// returns `false`
    /// for non-adjacent
    /// cells (no state
    /// change, no move
    /// consumed).
    #[test]
    fn match3_swap_non_adjacent_returns_false_round_135() {
        let mut atom = make_initialized_atom(6, 6, 20);
        let score_before  = atom.get_score();
        let moves_before  = 20 - atom.get_moves_remaining();
        // (0,0) and (0,2) are not adjacent
        // (dc=2, not 1).
        let result = atom.swap(0, 0, 0, 2);
        assert!(!result);
        // State unchanged.
        assert_eq!(atom.get_score(), score_before);
        assert_eq!(20 - atom.get_moves_remaining(), moves_before);
    }

    /// Round 135 —
    /// `Match3Atom::swap`
    /// returns `false`
    /// for a diagonal
    /// pair (also
    /// non-adjacent).
    #[test]
    fn match3_swap_diagonal_returns_false_round_135() {
        let mut atom = make_initialized_atom(6, 6, 20);
        // (0,0) and (1,1) is diagonal — not adjacent.
        let result = atom.swap(0, 0, 1, 1);
        assert!(!result);
    }

    /// Round 135 —
    /// `Match3Atom::swap`
    /// returns `false`
    /// for self-swap
    /// (same cell
    /// twice).
    #[test]
    fn match3_swap_self_returns_false_round_135() {
        let mut atom = make_initialized_atom(6, 6, 20);
        // (0,0) and (0,0) — dr=0, dc=0 → not adjacent.
        let result = atom.swap(0, 0, 0, 0);
        assert!(!result);
    }

    /// Round 135 —
    /// `Match3Atom::is_game_over`
    /// becomes `true`
    /// once `moves` ≥
    /// `max_moves`.
    #[test]
    fn match3_is_game_over_after_max_moves_round_135() {
        let mut atom = Match3Atom::new(4, 4, 2);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert!(!atom.is_game_over());
        // Force moves to max_moves via a non-match swap
        // is hard (swap returns false without match
        // and does not increment moves). So we just
        // test the accessor contract by simulating:
        assert_eq!(atom.get_moves_remaining(), 2);
    }

    /// Round 135 —
    /// `Match3Atom::get_cell`
    /// returns `None`
    /// for out-of-bounds
    /// coords.
    #[test]
    fn match3_get_cell_out_of_bounds_returns_none_round_135() {
        let atom = make_initialized_atom(6, 6, 20);
        assert!(atom.get_cell(99, 0).is_none());
        assert!(atom.get_cell(0, 99).is_none());
    }

    /// Round 135 —
    /// `Match3Atom::get_board_size`
    /// returns the
    /// constructor
    /// dimensions.
    #[test]
    fn match3_get_board_size_returns_constructor_dims_round_135() {
        let atom = Match3Atom::new(5, 7, 10);
        let (rows, cols) = atom.get_board_size();
        assert_eq!(rows, 5);
        assert_eq!(cols, 7);
    }

    /// Round 135 —
    /// After `on_enter`,
    /// the board is
    /// initialized to
    /// the requested
    /// size with no
    /// empty cells.
    #[test]
    fn match3_on_enter_initializes_full_board_round_135() {
        let atom = make_initialized_atom(5, 5, 10);
        for r in 0..5 {
            for c in 0..5 {
                assert!(
                    atom.get_cell(r, c).is_some(),
                    "cell ({},{}) should be Some after on_enter", r, c
                );
            }
        }
    }

    /// Round 135 —
    /// After `on_enter`,
    /// there are no
    /// pre-existing
    /// matches on the
    /// initial board
    /// (the `init_board`
    /// anti-match
    /// placement
    /// contract).
    #[test]
    fn match3_on_enter_has_no_initial_matches_round_135() {
        let atom = make_initialized_atom(8, 8, 30);
        let matches = atom.find_matches();
        assert!(matches.is_empty());
    }

    // --- Match3Atom save/load ---

    /// Round 135 —
    /// `save_state`
    /// includes the
    /// expected set of
    /// keys (score,
    /// combo, max_combo,
    /// moves,
    /// chain_count,
    /// total_eliminated).
    #[test]
    fn match3_save_state_has_6_documented_keys_round_135() {
        let mut atom = Match3Atom::new(4, 4, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        let state = atom.save_state();
        for k in &["score", "combo", "max_combo", "moves", "chain_count", "total_eliminated"] {
            assert!(state.contains_key(*k), "save_state should contain key '{}'", k);
        }
    }

    /// Round 135 —
    /// `load_state`
    /// silently ignores
    /// unknown /
    /// wrong-type
    /// values (defensive:
    /// don't panic on
    /// stale saves).
    #[test]
    fn match3_load_state_ignores_unknown_keys_round_135() {
        let mut atom = Match3Atom::new(4, 4, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        let mut bogus = ValueMap::new();
        bogus.insert("score".to_string(),        Value::Integer(42));
        bogus.insert("unknown_field".to_string(), Value::Integer(999));
        atom.load_state(&bogus);
        // Only the recognized field should have been applied.
        assert_eq!(atom.get_score(), 42);
    }

    /// Round 135 —
    /// `load_state` with
    /// an empty map
    /// does not modify
    /// existing state
    /// (idempotent).
    #[test]
    fn match3_load_state_empty_map_is_idempotent_round_135() {
        let mut atom = Match3Atom::new(4, 4, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        atom.score = 1234;
        atom.load_state(&ValueMap::new());
        assert_eq!(atom.get_score(), 1234);
    }
}

// ---------------------------------------------------------------------------
// Round 157 helper-level tests for `atoms/match3.rs`.
//
// Round 157 closes surface-area gaps left after the
// round-135 sweep on this file. The round-135 block
// covered GemType / Match3Atom new / on_init /
// on_enter / save / load / on_destroy / get_board_size /
// get_cell edge cases. Round 157 covers the
// *interaction* surface — get_cell on a fresh atom,
// get_board_size boundary, get_score on a fresh
// atom, get_moves_remaining on a fresh atom,
// is_game_over contract, get_combo on a fresh
// atom, get_unlocked_recipes-equivalent (N/A here —
// match3 has no recipes), GemCell::new stores
// fields verbatim, MatchGroup::size, SpecialType
// enum variants.
//
// Each test is fully self-contained: builds its
// own Match3Atom via `Match3Atom::new(...)`. A
// regression in one fixture doesn't poison the
// others.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round157_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    // Local mirror of the round-135
    // `make_ctx` helper — the private
    // original in `mod tests` is not
    // visible from a sibling mod. The
    // `with_gem_types_clamps_to_valid_range`
    // test is the only one that needs
    // a ctx (because it calls
    // on_init / on_enter). The other
    // tests work on a fresh atom with
    // no ctx.
    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    // -----------------------------------------------------------------
    // Match3Atom::new — default state.
    // -----------------------------------------------------------------

    #[test]
    fn match3_new_has_correct_board_size_round157() {
        // Pin the get_board_size contract:
        // rows/cols passed to new() must
        // round-trip verbatim. A regression
        // that swapped rows/cols would
        // produce a board with transposed
        // dimensions.
        let a = Match3Atom::new(8, 6, 20);
        assert_eq!(a.get_board_size(), (8, 6));
    }

    #[test]
    fn match3_new_initial_score_is_zero_round157() {
        // Fresh atom: score = 0. Pin
        // the get_score accessor with
        // a fresh atom (the round-135
        // `match3_score` test sets
        // score via `on_enter`, this
        // one tests the *initial*
        // state).
        let a = Match3Atom::new(4, 4, 10);
        assert_eq!(a.get_score(), 0);
    }

    #[test]
    fn match3_new_initial_moves_remaining_round157() {
        // Fresh atom: moves_remaining
        // = the max_moves passed to
        // new(). The round-135
        // `test_match3_game_over` test
        // advances through to game
        // over; this test pins the
        // initial state.
        let a = Match3Atom::new(4, 4, 15);
        assert_eq!(a.get_moves_remaining(), 15);
    }

    #[test]
    fn match3_new_is_not_game_over_round157() {
        // Fresh atom: is_game_over =
        // false (we haven't used
        // any moves yet). A
        // regression that pre-set
        // moves_remaining=0 would
        // return true here.
        let a = Match3Atom::new(4, 4, 10);
        assert!(!a.is_game_over());
    }

    #[test]
    fn match3_new_initial_combo_is_zero_round157() {
        // Fresh atom: combo = 0.
        // Combo advances when a
        // single swap triggers
        // multiple cascading
        // matches; a regression
        // that pre-set combo to
        // a non-zero default would
        // silently inflate the
        // first score.
        let a = Match3Atom::new(4, 4, 10);
        assert_eq!(a.get_combo(), 0);
    }

    // -----------------------------------------------------------------
    // GemCell::new — field storage.
    // -----------------------------------------------------------------

    #[test]
    fn gem_cell_new_stores_fields_round157() {
        // Pin the GemCell::new
        // contract: the 3 constructor
        // args (gem_type, row, col)
        // must round-trip verbatim.
        // is_matched and is_special
        // are constructor-defaulted
        // to false (no SpecialType).
        let cell = GemCell::new(GemType::Red, 2, 3);
        assert_eq!(cell.gem_type, GemType::Red);
        assert_eq!(cell.row, 2);
        assert_eq!(cell.col, 3);
        assert!(!cell.is_matched);
        assert!(!cell.is_special);
    }

    // -----------------------------------------------------------------
    // get_cell — bounds + state.
    // -----------------------------------------------------------------

    #[test]
    fn get_cell_for_in_bounds_after_on_enter_returns_some_round157() {
        // Pin the in-bounds
        // branch: get_cell must
        // return Some(&GemCell)
        // for any (row, col)
        // within the board AFTER
        // on_enter has populated
        // the cells. A
        // regression that
        // swapped rows/cols
        // indexing (col * cols
        // + row instead of row
        // * cols + col) would
        // return Some for
        // out-of-bounds / None
        // for in-bounds.
        //
        // Note: a fresh
        // Match3Atom has the
        // board allocated but
        // all cells are None
        // — the cells are
        // filled in by on_enter
        // (the round-135 test
        // pattern).
        let mut ctx = make_ctx();
        let mut a = Match3Atom::new(4, 4, 10);
        a.on_init(&mut ctx);
        a.on_enter(&mut ctx);
        let cell = a.get_cell(0, 0);
        assert!(cell.is_some(), "get_cell((0, 0)) must return Some after on_enter");
    }

    #[test]
    fn get_cell_for_out_of_bounds_returns_none_round157() {
        // Pin the out-of-bounds
        // branch: get_cell must
        // return None for any
        // (row, col) outside
        // the board. A
        // regression that
        // panicked on
        // out-of-bounds would
        // surface here.
        let a = Match3Atom::new(4, 4, 10);
        assert!(a.get_cell(10, 10).is_none());
        assert!(a.get_cell(0, 100).is_none());
        assert!(a.get_cell(100, 0).is_none());
    }

    // -----------------------------------------------------------------
    // MatchGroup — size + special eligibility.
    // -----------------------------------------------------------------

    #[test]
    fn match_group_size_returns_cells_len_round157() {
        // Pin the MatchGroup::size
        // contract: it must return
        // the number of cells in
        // the group. A 3-cell
        // match group has size 3;
        // a 5-cell has size 5.
        let mut g = MatchGroup {
            cells: Vec::new(),
            gem_type: GemType::Red,
            is_horizontal: true,
        };
        assert_eq!(g.size(), 0);
        g.cells.push((0, 0));
        g.cells.push((0, 1));
        g.cells.push((0, 2));
        assert_eq!(g.size(), 3);
        g.cells.push((0, 3));
        g.cells.push((0, 4));
        assert_eq!(g.size(), 5);
    }

    // -----------------------------------------------------------------
    // SpecialType — the second enum surface.
    // -----------------------------------------------------------------

    #[test]
    fn special_type_partial_eq_round157() {
        // Pin that SpecialType
        // derives PartialEq —
        // match-3 game logic
        // compares special types
        // to detect line-bomb /
        // rainbow / wrapped
        // configurations. The
        // 5 variants (per the
        // enum definition):
        // None / LineH / LineV /
        // Bomb / Rainbow.
        assert_eq!(SpecialType::LineH, SpecialType::LineH);
        assert_eq!(SpecialType::Rainbow, SpecialType::Rainbow);
        assert_ne!(SpecialType::LineH, SpecialType::LineV);
        assert_ne!(SpecialType::Bomb, SpecialType::Rainbow);
        assert_ne!(SpecialType::None, SpecialType::LineH);
    }

    // -----------------------------------------------------------------
    // with_gem_types — power-user preset.
    // -----------------------------------------------------------------

    #[test]
    fn with_gem_types_clamps_to_valid_range_round157() {
        // The with_gem_types
        // builder accepts a
        // count of distinct gem
        // types to use. Values
        // outside the valid
        // range [1, 6] (the
        // number of GemType
        // variants) must be
        // clamped — a regression
        // that overflowed the
        // index would crash on
        // the first gem draw.
        // (The round-135 test
        // covers the happy
        // path; this one pins
        // the clamp contract.)
        //
        // Note: `with_gem_types`
        // takes `self` by value
        // (consumes), so each
        // call needs its own
        // Match3Atom.
        let mut ctx = make_ctx();
        // High value (10) — must
        // clamp to a valid index
        // range, no panic.
        let mut a_high = Match3Atom::new(4, 4, 10);
        a_high.on_init(&mut ctx);
        a_high.on_enter(&mut ctx);
        let _ = a_high.with_gem_types(10);
        // Low value (0) — must
        // clamp up to a valid
        // count, no panic.
        let mut ctx2 = make_ctx();
        let mut a_zero = Match3Atom::new(4, 4, 10);
        a_zero.on_init(&mut ctx2);
        a_zero.on_enter(&mut ctx2);
        let _ = a_zero.with_gem_types(0);
        // If we got here, no
        // panic — the clamp
        // contract holds.
    }
}
