// #![allow(warnings)]
#![allow(unused_must_use)]

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use board::Player;
use chess::PieceType;
use movegen::NULL_MOVE;

mod board;
mod engine;
mod movegen;
mod position;
mod mailbox;
mod perft;
mod test;

// use std::{ io };
// use std::time::{ Instant };


// struct HumanPlayer;

// impl Player for HumanPlayer {
//     fn get_move(&self, bstate: &BoardState) -> Move {
//         let stdin = io::stdin();
//         let mut input1 = String::new();
//         let mut input2 = String::new();

//         loop {
//             println!("Move from:");
//             stdin.read_line(&mut input1);
//             println!("Move to:");
//             stdin.read_line(&mut input2);
//             let _illegal = true;
//             let (i, j) = Position::move_as_notation(&input1, &input2);

//             for mv in &bstate.legal_moves {
//                 if mv.from == i && mv.to == j {
//                     return *mv;
//                 }
//             }
//             println!("Move isn't legal!");
//             input1.clear();
//             input2.clear();
//             continue;
//         }
//     }
// }



// fn game_loop() {
//     let white_player = RandomPlayer;
//     let black_player = EnginePlayer { depth: 4 };
//     let mut board = Board::new(Box::new(white_player), Box::new(black_player));

//     loop {
//         match board.make_move() {
//             Ok(_) => {}
//             Err(e) => {
//                 println!("{:?}", e);
//                 break;
//             }
//         }
//         let game_state = board.current_state.get_gamestate();
//         println!("Game state: {:?}", game_state);

//         board.current_state.position.print_board();

//         if game_state != GameState::Active && game_state != GameState::Check {
//             println!("Game over, gamestate: {:?}", game_state);
//             println!("{:#?}", board.current_state);
//             break;
//         }
//     }
// }

// fn main() {
//     let pos = Position::new_starting();
//     //let mut pos = Position::new_position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
//     //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
//     pos.print_board();

//     perft(&pos, 5);

//     //game_loop();
// }

slint::include_modules!();

struct EnginePlayer {
    depth: i32
}
impl board::Player for EnginePlayer {
    fn get_move(&self, bstate: &board::BoardState) -> movegen::Move {
        *engine::choose_move(bstate, 4).1
    }
    
}

struct HumanPlayer;
impl Player for HumanPlayer {
    fn get_move(&self, bstate: &board::BoardState) -> movegen::Move {
        movegen::NULL_MOVE
    }
    
}

type PieceUI = slint_generatedBoard_UI::Piece_UI;
type PieceColourUI = slint_generatedBoard_UI::PieceColour_UI;
type PieceTypeUI = slint_generatedBoard_UI::PieceType_UI;

fn ui_convert_piece(piece: movegen::Piece) -> PieceUI {
    match piece.pcolour {
        movegen::PieceColour::White => match piece.ptype {
            movegen::PieceType::Pawn => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Pawn},
            movegen::PieceType::Knight => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Knight},
            movegen::PieceType::Bishop => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Bishop},
            movegen::PieceType::Rook => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Rook},
            movegen::PieceType::Queen => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Queen},
            movegen::PieceType::King => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::King},
            movegen::PieceType::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None},
        },
        movegen::PieceColour::Black => match piece.ptype {
            movegen::PieceType::Pawn => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Pawn},
            movegen::PieceType::Knight => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Knight},
            movegen::PieceType::Bishop => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Bishop},
            movegen::PieceType::Rook => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Rook},
            movegen::PieceType::Queen => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Queen},
            movegen::PieceType::King => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::King},
            movegen::PieceType::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None},
        },
        movegen::PieceColour::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None}
    }
}

fn make_engine_move(board: Arc<Rc<RefCell<board::Board>>>) {
    let mut b = board.borrow_mut();
    let mv = EnginePlayer { depth: 4 }.get_move(&b.current_state);
    b.make_move(&mv);
}

fn main() -> Result<(), slint::PlatformError> {

    let white_player = HumanPlayer;
    let black_player = EnginePlayer { depth: 4 };
    let mut board = Arc::new(Mutex::new(board::Board::new(Box::new(white_player), Box::new(black_player))));    
    
    
    let ui = Board_UI::new().unwrap();
    let ui_weak_new_game = ui.as_weak();
    let ui_weak_refresh_position = ui.as_weak();
    let ui_weak_make_move = ui.as_weak();
    let ui_weak_engine_make_move = ui.as_weak();

    let board_new_game = board.clone();
    let board_refresh_position = board.clone();
    let board_make_move = board.clone();
    let board_engine_make_move = board.clone();

    ui.on_new_game(move || {
        let ui = ui_weak_new_game.upgrade().unwrap();
        let white_player = HumanPlayer;
        let black_player = EnginePlayer { depth: 4 };
        *board_new_game.lock().unwrap() = board::Board::new(Box::new(white_player), Box::new(black_player));    
        ui.invoke_refresh_position();
    });

    ui.on_refresh_position(move || {
        let ui = ui_weak_refresh_position.upgrade().unwrap();
        let mut ui_position: Vec<PieceUI> = vec![];
        for s in board_refresh_position.lock().unwrap().current_state.position.position {
            match s {
                movegen::Square::Piece(p) => ui_position.push(ui_convert_piece(p)),
                movegen::Square::Empty => ui_position.push(ui_convert_piece(movegen::NULL_PIECE))
            }
        }
        let pos = std::rc::Rc::new(slint::VecModel::from(ui_position));
        ui.set_position(pos.into());
    });
    
    ui.on_make_move(move || {
        let ui = ui_weak_make_move.upgrade().unwrap();

        let from = ui.get_selected_from_square();
        let to = ui.get_selected_to_square();
        let mut legal_mv: movegen::Move = NULL_MOVE;

        for mv in board_make_move.lock().unwrap().current_state.legal_moves.clone() {
            if mv.from as i32 == from && mv.to as i32 == to {
                legal_mv = mv;
            }
        }
        board_make_move.lock().unwrap().make_move(&legal_mv);
    });

    ui.on_engine_make_move(move || {
        let ui = ui_weak_engine_make_move.upgrade().unwrap();
        board_engine_make_move.lock().unwrap().player_make_move();
    });
    
    ui.invoke_refresh_position();
    ui.run()
}