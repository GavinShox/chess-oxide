// #![allow(warnings)]
#![allow(unused_must_use)]

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};
use chess::*;
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
//     //let pos = Position::new_starting();
//     let mut pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
//     //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
//     pos.print_board();

//     perft(&pos, 5);

//     //game_loop();
// }

slint::include_modules!();

type PieceUI = slint_generatedBoard_UI::Piece_UI;
type PieceColourUI = slint_generatedBoard_UI::PieceColour_UI;
type PieceTypeUI = slint_generatedBoard_UI::PieceType_UI;

fn ui_convert_piece(piece: chess::Piece) -> PieceUI {
    match piece.pcolour {
        chess::PieceColour::White => match piece.ptype {
            chess::PieceType::Pawn => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Pawn},
            chess::PieceType::Bishop => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Bishop},
            chess::PieceType::Knight => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Knight},
            chess::PieceType::Rook => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Rook},
            chess::PieceType::Queen => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::Queen},
            chess::PieceType::King => PieceUI {piece_colour: PieceColourUI::White, piece_type: PieceTypeUI::King},
            chess::PieceType::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None},
        },
        chess::PieceColour::Black => match piece.ptype {
            chess::PieceType::Pawn => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Pawn},
            chess::PieceType::Knight => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Knight},
            chess::PieceType::Bishop => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Bishop},
            chess::PieceType::Rook => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Rook},
            chess::PieceType::Queen => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::Queen},
            chess::PieceType::King => PieceUI {piece_colour: PieceColourUI::Black, piece_type: PieceTypeUI::King},
            chess::PieceType::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None},
        },
        chess::PieceColour::None => PieceUI {piece_colour: PieceColourUI::None, piece_type: PieceTypeUI::None}
    }
}

fn main() -> Result<(), slint::PlatformError> {

    let mut board = Arc::new(Mutex::new(board::Board::new()));    
    
    
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
        *board_new_game.lock().unwrap() = board::Board::new();    
        ui.invoke_refresh_position();
    });

    ui.on_refresh_position(move || {
        let ui = ui_weak_refresh_position.upgrade().unwrap();
        let mut ui_position: Vec<PieceUI> = vec![];
        for s in board_refresh_position.lock().unwrap().current_state.get_pos64() {
            match s {
                chess::Square::Piece(p) => ui_position.push(ui_convert_piece(*p)),
                chess::Square::Empty => ui_position.push(ui_convert_piece(chess::NULL_PIECE))
            }
        }
        let pos = std::rc::Rc::new(slint::VecModel::from(ui_position));
        ui.set_position(pos.into());
    });
    
    ui.on_make_move(move || -> bool{
        let ui = ui_weak_make_move.upgrade().unwrap();

        let from = ui.get_selected_from_square();
        let to = ui.get_selected_to_square();
        let mut legal_mv: chess::Move = NULL_MOVE;

        for mv in board_make_move.lock().unwrap().current_state.legal_moves.clone() {
            if mv.from as i32 == from && mv.to as i32 == to {
                legal_mv = mv;
            }
        }
        match board_make_move.lock().unwrap().make_move(&legal_mv) {
            Ok(_) => {
                return true;
            }
            Err(_) => {
                return false;
            }
        }
    });

    ui.on_engine_make_move(move || {
        let ui = ui_weak_engine_make_move.clone();
        let mut bmem: Arc<Mutex<Board>> = board_engine_make_move.clone();

        std::thread::spawn(move || {
            bmem.lock().unwrap().make_engine_move(4);
            slint::invoke_from_event_loop(move || {
                ui.upgrade().unwrap().invoke_refresh_position();
                ui.upgrade().unwrap().set_engine_made_move(true);
            });
        });
    });
    
    ui.invoke_refresh_position();
    ui.run()
}