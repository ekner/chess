use std::{convert::TryInto};

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
    GameDone,
    NoSourcePiece,
    IncorrectSourceColor,
    MoveToSamePos,
    InvalidTargetPosition,
    MoveToSameColor,
    InvalidMove,
    ResultsInCheck,
}

#[derive(Copy, Clone, Debug)]
pub enum MoveSuccess {
    Ok,
    GameWonByWhite,
    GameWonByBlack,
}

impl MoveError {
    pub fn to_string(&self) -> &str {
        match self {
            &Self::GameDone => "The game has finished",
            &Self::NoSourcePiece => "You have not marked a square with a piece to move",
            &Self::IncorrectSourceColor => "You have picked the wrong color to move",
            &Self::MoveToSamePos => "You are trying to move to the same position",
            &Self::InvalidTargetPosition => "You cannot move outside the board",
            &Self::MoveToSameColor => "You cannot move to your own pieces",
            &Self::InvalidMove => "Invalid move for selected piece",
            &Self::ResultsInCheck => "This move places you in check",
        }
    }
}

impl MoveSuccess {
    pub fn to_string(&self) -> &str {
        match self {
            &Self::Ok => "Ok",
            &Self::GameWonByWhite => "White has won",
            &Self::GameWonByBlack => "Black has won",
        }
    }

    pub fn get_game_won_by_player(player: Player) -> MoveSuccess {
        match player {
            Player::White => MoveSuccess::GameWonByWhite,
            Player::Black => MoveSuccess::GameWonByBlack,
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
    game_running: bool,
}

impl State {
    pub fn new() -> State {
        State {
            board: State::init_board(),
            current_player: Player::White,
            total_steps: 0,
            white_eliminated: Vec::new(),
            black_eliminated: Vec::new(),
            game_running: true,
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

    fn check_game_running(&self) -> Result<(), MoveError> {
        if self.game_running {
            Ok(())
        } else {
            Err(MoveError::GameDone)
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

    fn get_all_pos_between(from: Pos, to: Pos) -> Vec<Pos> {
        let mut list: Vec<Pos> = Vec::new();
        if from.y == to.y {
            for x in range(from.x, to.x).skip(1) {
                list.push(Pos::new(x, from.y));
            }
        } else if from.x == to.x {
            for y in range(from.y, to.y).skip(1) {
                list.push(Pos::new(from.x, y));
            }
        } else if (from.x - to.x).abs() == (from.y - to.y).abs() {
            for x in range(from.x, to.x).skip(1) {
                let y = if to.y >= from.y {
                    from.y + (x - from.x).abs()
                } else {
                    from.y - (x - from.x).abs()
                };
                list.push(Pos::new(x, y));
            }
        } else {
            panic!("check_all_squares_between was run with inconsistent parameters");
        }
        list
    }

    fn check_all_squares_between_clear(&self, from: Pos, to: Pos) -> bool {
        let list = State::get_all_pos_between(from, to);
        for pos in list {
            if !self.get(pos).is_none() {
                return false;
            }
        }
        true
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

    fn get_other_player(player: Player) -> Player {
        match player {
            Player::White => Player::Black,
            Player::Black => Player::White,
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

    fn get_king_pos(&self, player: Player) -> Option<Pos> {
        for x in 0..8 {
            for y in 0..8 {
                let pos = Pos::new(x, y);
                if let Some(piece) = self.get(pos) {
                    if piece.player == player && piece.piece_type == PieceType::King {
                        return Some(pos)
                    }
                }
            }   
        };
        None
    }

    fn get_threatening_pieces(&self, player: Player) -> Vec<Pos> {
        let mut list: Vec<Pos> = Vec::new();
        let king_pos = self.get_king_pos(player).unwrap();
        for pos in self.get_all_pieces_for_player(State::get_other_player(player)) {
            if let Ok(()) = self.check_valid_move(pos, king_pos) {
                list.push(pos);
            }
        }
        list
    }

    fn check_if_move_results_in_check(&self, from: Pos, to: Pos) -> Result<(), MoveError> {
        let mut state_copy = self.clone();
        state_copy.perform_move(from, to);
        if state_copy.is_player_check(self.current_player) {
            Err(MoveError::ResultsInCheck)
        } else {
            Ok(())
        }
    }

    fn is_player_check(&self, player: Player) -> bool {
        let list = self.get_threatening_pieces(player);
        list.len() != 0
    }

    fn is_player_check_mate(&self, player: Player) -> bool {
        let threatening_pieces = self.get_threatening_pieces(player);

        //println!("begin------");
        //println!("{:?}", threatening_pieces);

        // inga spelare chackar spelare A
        if threatening_pieces.len() == 0 {
            false
        }
        else if self.can_avoid_by_moving_king(player) {
            //println!("can avoid by moving king");
            false
        }
        // Det finns 2 spelare som chackar, med andra ord är det kört:
        else if threatening_pieces.len() > 1 {
            true
        }
        else if self.can_avoid_by_attack(player, threatening_pieces[0]) {
            //println!("can avoid by attack");
            false
        }
        else if self.can_avoid_by_block(player, threatening_pieces[0]) {
            //println!("can avoid by block");
            false
        }
        else {
            true
        }
    }

    fn get_positions_around(pos: Pos) -> Vec<Pos> {
        let mut list: Vec<Pos> = Vec::new();
        for x in (pos.x-1)..(pos.x+1) {
            for y in (pos.y-1)..(pos.y+1) {
                if x >= 0 && x < 8 && y >= 0 && y < 8 && (x != pos.x || y != pos.y) {
                    list.push(Pos::new(x, y));
                }
            }    
        }
        list
    }

    fn pos_contains_player(&self, pos: Pos, player: Player) -> bool {
        if let Some(piece) = self.get(pos) {
            if piece.player == player {
                return true;
            }
        }
        false
    }

    fn can_avoid_by_moving_king(&self, player: Player) -> bool {
        let king_position = self.get_king_pos(player).unwrap();
        for pos in State::get_positions_around(king_position) {
            if self.pos_contains_player(pos, player) {
                continue;
            }
            let mut state_copy = self.clone();
            state_copy.perform_move(king_position, pos);
            if !state_copy.is_player_check(player) {
                return true;
            }
        }
        false
    }

    fn can_avoid_by_attack(&self, player: Player, threatening_player: Pos) -> bool {
        for pos in self.get_all_pieces_for_player(player) {
            if let Ok(()) = self.check_valid_move(pos, threatening_player) {
                let mut state_copy = self.clone();
                state_copy.perform_move(pos, threatening_player);
                if !state_copy.is_player_check(player) {
                    return true;
                }
            }
        }
        false
    }

    fn can_avoid_by_block(&self, player: Player, threatening_player: Pos) -> bool {
        let threatening_piece = self.get(threatening_player).unwrap();
        if threatening_piece.piece_type != PieceType::Queen  &&
           threatening_piece.piece_type != PieceType::Rook   &&
           threatening_piece.piece_type != PieceType::Bishop
        {
            return false;
        }

        let king_pos = self.get_king_pos(player).unwrap();

        //println!("{:?}", self.get_all_pieces_for_player(player));

        for between_pos in State::get_all_pos_between(threatening_player, king_pos) {
            for piece_pos in self.get_all_pieces_for_player(player) {
                if let Ok(()) = self.check_valid_move(piece_pos, between_pos) {

                    //println!("this happens");
                    //println!("{:?}, {:?}", piece_pos, between_pos);

                    let mut state_copy = self.clone();
                    state_copy.perform_move(piece_pos, between_pos);

                    if !state_copy.is_player_check(player) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn handle_post_move(&mut self) -> Result<MoveSuccess, MoveError> {
        if self.is_player_check_mate(self.current_player) {
            //println!("is check mate");
            self.game_running = false;
            Ok(MoveSuccess::get_game_won_by_player(State::get_other_player(self.current_player)))
        } else {
            //println!("is not check mate");
            Ok(MoveSuccess::Ok)
        }
    }

    pub fn move_piece(&mut self, from: Pos, to: Pos) -> Result<MoveSuccess, MoveError> {
        self.check_game_running()?;
        State::check_valid_bounds(from)?;
        State::check_valid_bounds(to)?;
        State::check_not_same_position(from, to)?; // Check that we don't move to the same position
        self.check_piece_at_source(from)?;     // Check that we move something
        self.check_correct_color_at_source(from)?;
        self.check_not_move_to_same_color(from, to)?;
        self.check_valid_move(from, to)?;
        self.check_if_move_results_in_check(from, to)?;

        self.eliminate_target(to);
        self.perform_move(from, to);
        self.swap_current_player();
        self.total_steps += 1;

        self.handle_post_move()
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

    #[test]
    fn fools_mate_test() {
        let mut state = State::new();
        assert!(state.move_piece(Pos::new(5, 1), Pos::new(5, 2)).is_ok());
        assert!(state.move_piece(Pos::new(4, 6), Pos::new(4, 4)).is_ok());
        assert!(state.move_piece(Pos::new(6, 1), Pos::new(6, 3)).is_ok());
        assert!(state.game_running);
        assert!(state.move_piece(Pos::new(3, 7), Pos::new(7, 3)).is_ok());
        assert!(!state.game_running);
    }
}