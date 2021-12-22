use std::{convert::TryInto, ops::Range};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceType {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

impl PieceType {
    pub fn to_string(&self) -> &str {
        match self {
            &Self::King => "K",
            &Self::Queen => "Q",
            &Self::Rook => "R",
            &Self::Bishop => "B",
            &Self::Knight => "N",
            &Self::Pawn => "P",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MoveError {
    NoSourcePiece,
    IncorrectSourceColor,
    MoveToSamePos,
    InvalidTargetPosition,
    MoveToSameColor,
    InvalidMove,
}

pub enum MoveSuccess {
    
}

impl MoveError {
    pub fn to_string(&self) -> &str {
        match self {
            &Self::NoSourcePiece => "You have not marked a square with a piece to move",
            &Self::IncorrectSourceColor => "You have picked the wrong color to move",
            &Self::MoveToSamePos => "You are trying to move to the same position",
            &Self::InvalidTargetPosition => "You cannot move outside the board",
            &Self::MoveToSameColor => "You cannot move to your own pieces",
            &Self::InvalidMove => "Invalid move for selected piece",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Player {
    White,
    Black,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }

    pub fn index(&self) -> usize {
        (self.y * 8 + self.x).try_into().unwrap()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}

impl Piece {
    fn new(piece_type: PieceType, player: Player) -> Piece {
        Piece {piece_type, player}
    }
}

fn range(a: i32, b: i32) -> Box<dyn Iterator<Item = i32>> {
    if b > a {
        Box::new(a..b)
    } else {
        Box::new(((b+1)..(a+1)).rev())
    }
}

#[derive(Clone)]
pub struct State {
    board: [Option<Piece>; 64],
    current_player: Player,
    total_steps: u32,
    white_eliminated: Vec<PieceType>,
    black_eliminated: Vec<PieceType>,
}

impl State {
    pub fn new() -> State {
        State {
            board: State::init_board(),
            current_player: Player::White,
            total_steps: 0,
            white_eliminated: Vec::new(),
            black_eliminated: Vec::new(),
        }
    }

    fn init_board() -> [Option<Piece>; 64] {
        let mut board: [Option<Piece>; 64] = [None; 64];
        for i in 8..16 {
            board[i] = Some(Piece::new(PieceType::Pawn, Player::White));
        }
        for i in 48..56 {
            board[i] = Some(Piece::new(PieceType::Pawn, Player::Black));
        }
        board[0] = Some(Piece::new(PieceType::Rook, Player::White));
        board[1] = Some(Piece::new(PieceType::Knight, Player::White));
        board[2] = Some(Piece::new(PieceType::Bishop, Player::White));
        board[3] = Some(Piece::new(PieceType::Queen, Player::White));
        board[4] = Some(Piece::new(PieceType::King, Player::White));
        board[5] = Some(Piece::new(PieceType::Bishop, Player::White));
        board[6] = Some(Piece::new(PieceType::Knight, Player::White));
        board[7] = Some(Piece::new(PieceType::Rook, Player::White));
        board[56] = Some(Piece::new(PieceType::Rook, Player::Black));
        board[57] = Some(Piece::new(PieceType::Knight, Player::Black));
        board[58] = Some(Piece::new(PieceType::Bishop, Player::Black));
        board[59] = Some(Piece::new(PieceType::Queen, Player::Black));
        board[60] = Some(Piece::new(PieceType::King, Player::Black));
        board[61] = Some(Piece::new(PieceType::Bishop, Player::Black));
        board[62] = Some(Piece::new(PieceType::Knight, Player::Black));
        board[63] = Some(Piece::new(PieceType::Rook, Player::Black));
        board
    }

    pub fn get(&self, pos: Pos) -> Option<Piece> {
        self.board[pos.index()]
    }

    fn set(&mut self, pos: Pos, piece: Option<Piece>) {
        self.board[pos.index()] = piece;
    }

    fn check_piece_at_source(&self, pos: Pos) -> Result<(), MoveError> {
        match self.get(pos) {
            None => Err(MoveError::NoSourcePiece),
            Some(_) => Ok(())
        }
    }

    fn check_correct_color_at_source(&self, pos: Pos) -> Result<(), MoveError> {
        let p = self.get(pos).unwrap();
        if p.player == self.current_player {
            Ok(())
        } else {
            Err(MoveError::IncorrectSourceColor)
        }
    }

    fn check_not_same_position(from: Pos, to: Pos) -> Result<(), MoveError> {
        if from == to {
            Err(MoveError::MoveToSamePos)
        } else {
            Ok(())
        }
    }

    fn check_valid_bounds(pos: Pos) -> Result<(), MoveError> {
        if pos.x >= 0 && pos.x < 8 && pos.y >= 0 && pos.y < 8 {
            Ok(())
        } else {
            Err(MoveError::InvalidTargetPosition)
        }
    }

    fn check_not_move_to_same_color(&self, from: Pos, to: Pos) -> Result<(), MoveError> {
        match self.get(to) {
            None => Ok(()),
            Some(to_piece) => {
                let from_piece = self.get(from).unwrap();
                if to_piece.player == from_piece.player {
                    Err(MoveError::MoveToSameColor)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn check_valid_move_pawn(&self, from: Pos, to: Pos) -> bool {
        let piece = self.get(from).unwrap();

        match piece.player {
            Player::White => {
                if from.x == to.x && self.get(to).is_none() {
                    to.y == from.y + 1 ||
                    (from.y == 1 && to.y == 3 && self.get(Pos::new(from.x, 2)).is_none())
                } else if (from.x - to.x).abs() == 1 && to.y == from.y + 1 {
                    match self.get(to) {
                        Some(to_piece) => to_piece.player == Player::Black,
                        None => false 
                    }
                } else {
                    false
                }
            },
            Player::Black => {
                if from.x == to.x && self.get(to).is_none() {
                    to.y == from.y - 1 ||
                    (from.y == 6 && to.y == 4 && self.get(Pos::new(from.x, 5)).is_none())
                } else if (from.x - to.x).abs() == 1 && to.y == from.y - 1 {
                    match self.get(to) {
                        Some(to_piece) => to_piece.player == Player::White,
                        None => false 
                    }
                } else {
                    false
                }
            }
        }
    }

    fn check_all_squares_between<F: Fn(Pos) -> bool>(&self, from: Pos, to: Pos, fun: F) -> bool {
        if from.y == to.y {
            //for x in (from.x..to.x).skip(1) {
            for x in range(from.x, to.x).skip(1) {
                if !fun(Pos::new(x, from.y)) {
                    return false
                }
            }
        } else if from.x == to.x {
            //for y in (from.y..to.y).skip(1) {
            for y in range(from.y, to.y).skip(1) {
                if !fun(Pos::new(from.x, y)) {
                    return false
                }
            }
        } else if (from.x - to.x).abs() == (from.y - to.y).abs() {
            //for x in (from.x..to.x).skip(1) {
            for x in range(from.x, to.x).skip(1) {
                let y = if to.y >= from.y {
                    from.y + (x - from.x).abs()
                } else {
                    from.y - (x - from.x).abs()
                };
                if !fun(Pos::new(x, y)) {
                    return false
                }
            }
        } else {
            panic!("check_all_squares_between was run with inconsistent parameters");
        }
        true
    }

    fn check_all_squares_between_clear(&self, from: Pos, to: Pos) -> bool {
        self.check_all_squares_between(from, to, |pos| {
            self.get(pos).is_none()
        })
    }

    fn check_valid_move_rook(&self, from: Pos, to: Pos) -> bool {
        (from.x == to.x || from.y == to.y) && self.check_all_squares_between_clear(from, to)
    }

    fn check_valid_move_knight(&self, from: Pos, to: Pos) -> bool {
        (from.x - to.x).abs() == 1 && (from.y - to.y).abs() == 2 ||
        (from.x - to.x).abs() == 2 && (from.y - to.y).abs() == 1
    }

    fn check_valid_move_bishop(&self, from: Pos, to: Pos) -> bool {
        (from.x - to.x).abs() == (from.y - to.y).abs() && self.check_all_squares_between_clear(from, to)
    }

    fn check_valid_move_queen(&self, from: Pos, to: Pos) -> bool {
        (
            (from.x - to.x).abs() == (from.y - to.y).abs() ||
            from.x == to.x ||
            from.y == to.y
        )
        && self.check_all_squares_between_clear(from, to)
    }

    fn check_valid_move_king(&self, from: Pos, to: Pos) -> bool {
        to.x >= from.x - 1 && to.x <= from.x + 1 &&
        to.y >= from.y - 1 && to.y <= from.y + 1
    }

    fn check_valid_move(&self, from: Pos, to: Pos) -> Result<(), MoveError> {
        let piece = self.get(from).unwrap();

        let res = match piece.piece_type {
            PieceType::Pawn => self.check_valid_move_pawn(from, to),
            PieceType::Rook => self.check_valid_move_rook(from, to),
            PieceType::Knight => self.check_valid_move_knight(from, to),
            PieceType::Bishop => self.check_valid_move_bishop(from, to),
            PieceType::Queen => self.check_valid_move_queen(from, to),
            PieceType::King => self.check_valid_move_king(from, to),
        };

        if res {
            Ok(())
        } else {
            Err(MoveError::InvalidMove)
        }
    }

    fn eliminate_target(&mut self, to: Pos) {
        if let Some(target_piece) = self.get(to) {
            match self.current_player {
                Player::White => self.black_eliminated.push(target_piece.piece_type),
                Player::Black => self.white_eliminated.push(target_piece.piece_type),
            }
            self.set(to, None);
        }
    }

    fn perform_move(&mut self, from: Pos, to: Pos) {
        let piece = self.get(from).unwrap();
        self.set(to, Some(piece));
        self.set(from, None);
    }

    fn swap_current_player(&mut self) {
        match self.current_player {
            Player::White => self.current_player = Player::Black,
            Player::Black => self.current_player = Player::White,
        }
    }

    fn get_all_pieces_for_player(&self, player: Player) -> Vec<Pos> {
        let mut list: Vec<Pos> = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                let pos = Pos::new(x, y);
                if let Some(piece) = self.get(pos) {
                    if piece.player == player {
                        list.push(pos);
                    }
                }
            }   
        }
        list
    }

    fn get_threatening_pieces(A) {
        let list: Vec<Pos> = Vec::new();
        for all B pieces
            if b can do valid move to A:s king
                list.append(pos of b)
        return list
    }

    fn is_player_check_mate(A) {
        check_players = is_player_check(A) // innehåller positioner

        // inga spelare chackar spelare A
        if check_players == []
            return false
        
        if can_avoid_by_moving_king(A)
            return false

        // Det finns 2 spelare som chackar, med andra ord är det kört:
        if check_players.len > 1
            return true

        if can_avoid_by_attack(A, check_players[0])
            return false

        if can_avoid_by_block(A, check_players[0])
            return false

        return true
    }

    fn can_avoid_by_moving_king(A) {
        for pos in positionsAroundKing
            if A-player is at pos
                continue
            state_copy = state.copy()
            move the king to pos in state_copy
            if is_player_check(A, state_copy) == []
                return true
        return false
    }

    fn can_avoid_by_attack(A, check_player) {
        for piece in A-players except kung
            if piece can do valid move to check_player
                state_copy = state.copy()
                move piece to check_player in state_copy
                if is_player_check(A, state_copy) == []
                    return true
        return false
    }

    fn can_avoid_by_block(A, check_player) {
        if check_player != queen/rook/bishop
            return false
        for pos in positions between check_player and A:s king
            for piece in A-players except king 
                if piece can do valid move to pos
                    state_copy = state.copy()
                    move piece to check_player in state_copy
                    if is_player_check(A, state_copy) == []
                        return true
    }

    pub fn move_piece(&mut self, from: Pos, to: Pos) -> Result<(), MoveError> {
        self.check_piece_at_source(from)?;     // Check that we move something
        self.check_correct_color_at_source(from)?;
        State::check_not_same_position(from, to)?; // Check that we don't move to the same position
        State::check_valid_bounds(from)?;
        State::check_valid_bounds(to)?;
        self.check_not_move_to_same_color(from, to)?;
        self.check_valid_move(from, to)?;

        // Check for check mate

        self.eliminate_target(to);
        self.perform_move(from, to);
        self.swap_current_player();

        self.total_steps += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_test() {
        let mut list = Vec::new();

        for i in range(3, 6) {
            list.push(i);
        }

        for i in range(13, 11) {
            list.push(i);
        }

        assert_eq!(list, [3, 4, 5, 13, 12]);
    }

    #[test]
    fn get_test() {
        let state = State::new();
        assert_eq!(state.get(Pos::new(0, 0)).unwrap(), Piece::new(PieceType::Rook, Player::White));
        assert_eq!(state.get(Pos::new(2, 0)).unwrap(), Piece::new(PieceType::Bishop, Player::White));
        assert_eq!(state.get(Pos::new(7, 7)).unwrap(), Piece::new(PieceType::Rook, Player::Black));
        assert_eq!(state.get(Pos::new(2, 7)).unwrap(), Piece::new(PieceType::Bishop, Player::Black));
    }

    #[test]
    fn set_test() {
        let mut state = State::new();
        assert_eq!(state.board[Pos::new(0, 0).index()], Some(Piece::new(PieceType::Rook, Player::White)));
        state.set(Pos::new(0, 0), Some(Piece::new(PieceType::King, Player::White)));
        assert_eq!(state.board[Pos::new(0, 0).index()], Some(Piece::new(PieceType::King, Player::White)));
    }

    #[test]
    fn check_piece_at_source_test() {
        let state = State::new();
        assert!(state.check_piece_at_source(Pos::new(1, 1)).is_ok());
        assert!(state.check_piece_at_source(Pos::new(1, 2)).is_err());
    }

    #[test]
    fn check_correct_color_at_source_test() {
        let state = State::new();
        assert!(state.check_correct_color_at_source(Pos::new(0, 1)).is_ok());
        assert!(state.check_correct_color_at_source(Pos::new(0, 6)).is_err());
    }

    #[test]
    fn check_not_same_position_test() {
        assert!(State::check_not_same_position(Pos::new(0, 0), Pos::new(1, 0)).is_ok());
        assert!(State::check_not_same_position(Pos::new(1, 0), Pos::new(1, 0)).is_err());
    }

    #[test]
    fn check_valid_bounds_test() {
        assert!(State::check_valid_bounds(Pos::new(0, 0)).is_ok());
        assert!(State::check_valid_bounds(Pos::new(7, 7)).is_ok());
        assert!(State::check_valid_bounds(Pos::new(-1, 0)).is_err());
        assert!(State::check_valid_bounds(Pos::new(0, -1)).is_err());
        assert!(State::check_valid_bounds(Pos::new(8, 0)).is_err());
        assert!(State::check_valid_bounds(Pos::new(0, 8)).is_err());
    }

    #[test]
    fn check_not_move_to_same_color_test() {
        let mut state = State::new();
        state.current_player = Player::Black;
        assert!(state.check_not_move_to_same_color(Pos::new(0, 7), Pos::new(0, 5)).is_ok());
        assert!(state.check_not_move_to_same_color(Pos::new(0, 7), Pos::new(0, 0)).is_ok());
        assert!(state.check_not_move_to_same_color(Pos::new(0, 7), Pos::new(0, 6)).is_err());
    }

    #[test]
    fn check_valid_move_pawn_test() {
        let state = State::new();
        assert!(state.check_valid_move_pawn(Pos::new(0, 1), Pos::new(0, 2)));
        assert!(state.check_valid_move_pawn(Pos::new(0, 1), Pos::new(0, 3)));
        assert!(! state.check_valid_move_pawn(Pos::new(0, 1), Pos::new(1, 2)));
        assert!(! state.check_valid_move_pawn(Pos::new(0, 1), Pos::new(0, 4)));
    }

    #[test]
    fn check_all_squares_between_clear_test() {
        let mut state = State::new();

        // Straight
        assert!(state.check_all_squares_between_clear(Pos::new(0, 1), Pos::new(0, 6)));
        assert!(! state.check_all_squares_between_clear(Pos::new(0, 0), Pos::new(0, 6)));
        assert!(! state.check_all_squares_between_clear(Pos::new(0, 1), Pos::new(0, 7)));

        // Other way around
        assert!(state.check_all_squares_between_clear(Pos::new(0, 6), Pos::new(0, 1)));
        assert!(! state.check_all_squares_between_clear(Pos::new(0, 6), Pos::new(0, 0)));
        assert!(! state.check_all_squares_between_clear(Pos::new(0, 7), Pos::new(0, 1)));

        state.set(Pos::new(3, 4), Some(Piece::new(PieceType::Pawn, Player::White)));
        
        // Diagonal
        assert!(state.check_all_squares_between_clear(Pos::new(0, 6), Pos::new(5, 1)));
        assert!(state.check_all_squares_between_clear(Pos::new(5, 1), Pos::new(0, 6)));
        
        // Other way around
        assert!(! state.check_all_squares_between_clear(Pos::new(1, 6), Pos::new(6, 1)));
        assert!(! state.check_all_squares_between_clear(Pos::new(6, 1), Pos::new(1, 6)));
    }

    #[test]
    fn check_valid_move_rook_test() {
        let mut state = State::new();
        state.set(Pos::new(3, 4), Some(Piece::new(PieceType::Pawn, Player::White)));
        assert!(! state.check_valid_move_rook(Pos::new(0, 7), Pos::new(0, 4)));
        state.set(Pos::new(0, 6), None);
        assert!(state.check_valid_move_rook(Pos::new(0, 7), Pos::new(0, 4)));
        assert!(! state.check_valid_move_rook(Pos::new(0, 4), Pos::new(7, 4)));
        state.set(Pos::new(3, 4), None);
        assert!(state.check_valid_move_rook(Pos::new(0, 4), Pos::new(7, 4)));
    }

    #[test]
    fn check_valid_move_knight_test() {
        let state = State::new();
        assert!(state.check_valid_move_knight(Pos::new(1, 0), Pos::new(0, 2)));
        assert!(state.check_valid_move_knight(Pos::new(1, 0), Pos::new(2, 2)));
        assert!(state.check_valid_move_knight(Pos::new(1, 0), Pos::new(3, 1)));
        assert!(! state.check_valid_move_knight(Pos::new(1, 0), Pos::new(1, 2)));
        assert!(! state.check_valid_move_knight(Pos::new(1, 0), Pos::new(1, 1)));
        assert!(! state.check_valid_move_knight(Pos::new(1, 0), Pos::new(0, 1)));
    }

    #[test]
    fn check_valid_move_bishop_test() {
        let mut state = State::new();
        assert!(! state.check_valid_move_bishop(Pos::new(2, 7), Pos::new(0, 5)));
        state.set(Pos::new(1, 6), None);
        assert!(state.check_valid_move_bishop(Pos::new(2, 7), Pos::new(0, 5)));
    }

    #[test]
    fn check_valid_move_queen_test() {
        let mut state = State::new();
        state.set(Pos::new(1, 4), Some(Piece::new(PieceType::Pawn, Player::White)));
        assert!(! state.check_valid_move_queen(Pos::new(3, 7), Pos::new(3, 4)));
        state.set(Pos::new(3, 6), None);
        assert!(state.check_valid_move_queen(Pos::new(3, 7), Pos::new(3, 4)));
        assert!(! state.check_valid_move_queen(Pos::new(3, 4), Pos::new(0, 4)));
        state.set(Pos::new(1, 4), None);
        assert!(state.check_valid_move_queen(Pos::new(3, 4), Pos::new(0, 4)));
    }

    #[test]
    fn check_valid_move_king_test() {
        let state = State::new();
        assert!(state.check_valid_move_king(Pos::new(4, 4), Pos::new(3, 4)));
        assert!(state.check_valid_move_king(Pos::new(4, 4), Pos::new(4, 3)));
        assert!(state.check_valid_move_king(Pos::new(4, 4), Pos::new(3, 3)));
        assert!(! state.check_valid_move_king(Pos::new(4, 4), Pos::new(6, 4)));
        assert!(! state.check_valid_move_king(Pos::new(4, 4), Pos::new(4, 6)));
    }

    #[test]
    fn check_valid_move_test() {
        let mut state = State::new();
        assert!(state.check_valid_move(Pos::new(0, 1), Pos::new(0, 4)).is_err());
        state.set(Pos::new(0, 1), Some(Piece::new(PieceType::Rook, Player::White)));
        assert!(state.check_valid_move(Pos::new(0, 1), Pos::new(0, 4)).is_ok());
    }

    #[test]
    fn eliminate_target_test() {
        let mut state = State::new();
        assert!(state.black_eliminated.is_empty());
        state.eliminate_target(Pos::new(0, 6));
        assert!(state.get(Pos::new(0, 6)).is_none());
        assert!(state.black_eliminated.len() == 1);
        assert_eq!(state.black_eliminated[0], PieceType::Pawn);
    }

    #[test]
    fn perform_move_test() {
        let mut state = State::new();
        assert!(state.get(Pos::new(0, 7)).unwrap() == Piece::new(PieceType::Rook, Player::Black));
        assert!(state.get(Pos::new(7, 2)) == None);
        state.perform_move(Pos::new(0, 7), Pos::new(7, 2));
        assert!(state.get(Pos::new(0, 7)) == None);
        assert!(state.get(Pos::new(7, 2)).unwrap() == Piece::new(PieceType::Rook, Player::Black));
    }

    #[test]
    fn swap_current_player_test() {
        let mut state = State::new();
        assert!(state.current_player == Player::White);
        state.swap_current_player();
        assert!(state.current_player == Player::Black);
    }

    #[test]
    fn move_piece_test() {
        let mut state = State::new();
        assert!(state.move_piece(Pos::new(0, 1), Pos::new(0, 4)).is_err());
        assert!(state.move_piece(Pos::new(0, 6), Pos::new(0, 5)).is_err());
        assert!(state.move_piece(Pos::new(0, 1), Pos::new(0, 3)).is_ok());

        assert!(state.move_piece(Pos::new(0, 3), Pos::new(0, 4)).is_err());
        assert!(state.move_piece(Pos::new(4, 6), Pos::new(4, 5)).is_ok());

        assert!(state.move_piece(Pos::new(1, 1), Pos::new(1, 3)).is_ok());

        assert!(state.move_piece(Pos::new(5, 7), Pos::new(1, 3)).is_ok());
        assert!(state.black_eliminated.len() == 0);
        assert!(state.white_eliminated.len() == 1);
        assert_eq!(state.white_eliminated[0], PieceType::Pawn);
    }
}