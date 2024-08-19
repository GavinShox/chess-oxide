// #![allow(warnings)]
//#![allow(unused_must_use)]

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use chess::*;
use env_logger::{Builder, Target};
use slint::SharedString;
use std::env;
use std::sync::{Arc, Mutex};

slint::include_modules!();

type PieceUI = slint_generatedBoard_UI::Piece_UI;
type PieceColourUI = slint_generatedBoard_UI::PieceColour_UI;
type PieceTypeUI = slint_generatedBoard_UI::PieceType_UI;
type MoveNotationUI = slint_generatedBoard_UI::MoveNotation_UI;
//type MoveUI = slint_generatedBoard_UI::Move_UI;

fn ui_convert_piece(piece: chess::Piece) -> PieceUI {
    match piece.pcolour {
        chess::PieceColour::White => match piece.ptype {
            chess::PieceType::Pawn => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::Pawn,
            },
            chess::PieceType::Bishop => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::Bishop,
            },
            chess::PieceType::Knight => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::Knight,
            },
            chess::PieceType::Rook => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::Rook,
            },
            chess::PieceType::Queen => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::Queen,
            },
            chess::PieceType::King => PieceUI {
                piece_colour: PieceColourUI::White,
                piece_type: PieceTypeUI::King,
            },
            chess::PieceType::None => PieceUI {
                piece_colour: PieceColourUI::None,
                piece_type: PieceTypeUI::None,
            },
        },
        chess::PieceColour::Black => match piece.ptype {
            chess::PieceType::Pawn => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::Pawn,
            },
            chess::PieceType::Knight => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::Knight,
            },
            chess::PieceType::Bishop => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::Bishop,
            },
            chess::PieceType::Rook => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::Rook,
            },
            chess::PieceType::Queen => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::Queen,
            },
            chess::PieceType::King => PieceUI {
                piece_colour: PieceColourUI::Black,
                piece_type: PieceTypeUI::King,
            },
            chess::PieceType::None => PieceUI {
                piece_colour: PieceColourUI::None,
                piece_type: PieceTypeUI::None,
            },
        },
        chess::PieceColour::None => PieceUI {
            piece_colour: PieceColourUI::None,
            piece_type: PieceTypeUI::None,
        },
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // board::Board::from_fen("8/8/8/5R2/8/P1P3PP/P2QPP2/k5K1 b - - 0 1"

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let board = Arc::new(Mutex::new(Board::new()));

    let ui = Board_UI::new().unwrap();

    let ui_weak_get_gamestate = ui.as_weak();
    let board_get_gamestate = board.clone();
    ui.on_get_gamestate(move || {
        let ui = ui_weak_get_gamestate.upgrade().unwrap();
        let board = board_get_gamestate.lock().unwrap();
        let side_to_move = if board.current_state.side_to_move == chess::PieceColour::White {
            "White"
        } else {
            "Black"
        };
        let gamestate = board.current_state.get_gamestate().to_string();
        ui.set_gamestate(format!("{}'s turn: {}", side_to_move, gamestate).into());
    });

    let ui_weak_new_game = ui.as_weak();
    let board_new_game = board.clone();
    ui.on_new_game(move || {
        let ui = ui_weak_new_game.upgrade().unwrap();
        *board_new_game.lock().unwrap() = board::Board::new();
        ui.invoke_refresh_position();
    });

    let ui_weak_refresh_position = ui.as_weak();
    let board_refresh_position = board.clone();
    ui.on_refresh_position(move || {
        let ui = ui_weak_refresh_position.upgrade().unwrap();
        let mut ui_position: Vec<PieceUI> = vec![];
        for s in board_refresh_position
            .lock()
            .unwrap()
            .current_state
            .get_pos64()
        {
            match s {
                chess::Square::Piece(p) => ui_position.push(ui_convert_piece(*p)),
                chess::Square::Empty => ui_position.push(ui_convert_piece(chess::NULL_PIECE)),
            }
        }
        // reverse board if player is black
        if ui.get_player_colour() == PieceColour_UI::Black {
            ui_position.reverse();
        }
        let pos = std::rc::Rc::new(slint::VecModel::from(ui_position));

        // generate move history as vector of move notations
        let mut ui_moves: Vec<SharedString> = board_refresh_position
            .lock()
            .unwrap()
            .state_history
            .iter()
            .map(|x| x.last_move_as_notation().unwrap_or("".into()).into())
            .collect();
        ui_moves.remove(0); // remove first null move empty string

        let mut ui_move_history: Vec<MoveNotationUI> = vec![];
        for (i, chunk) in ui_moves.chunks(2).enumerate() {
            let mut mv_notation = MoveNotationUI {
                move_number: (i + 1) as i32,
                notation1: "".into(),
                notation2: "".into(),
            };
            if chunk.len() == 2 {
                mv_notation.notation1 = chunk[0].clone();
                mv_notation.notation2 = chunk[1].clone();
            } else {
                mv_notation.notation1 = chunk[0].clone();
            }
            ui_move_history.push(mv_notation);
        }

        ui.set_move_history(std::rc::Rc::new(slint::VecModel::from(ui_move_history)).into());

        // set gamestate
        ui.invoke_get_gamestate();

        // set current BoardState FEN
        ui.set_fen(
            board_refresh_position
                .lock()
                .unwrap()
                .current_state
                .to_fen()
                .into(),
        );

        // only set last move in GUI if it is not NULL_MOVE, then unwrap() is safe
        if board_refresh_position
            .lock()
            .unwrap()
            .current_state
            .last_move
            != NULL_MOVE
        {
            let last_move = board_refresh_position
                .lock()
                .unwrap()
                .current_state
                .last_move;
            let last_move_notation = board_refresh_position
                .lock()
                .unwrap()
                .current_state
                .last_move_as_notation()
                .unwrap();

            if ui.get_player_colour() == PieceColour_UI::Black {
                // reverse index if player is black as the board is flipped
                ui.set_last_move(Move_UI {
                    from_square: 63 - last_move.from as i32,
                    to_square: 63 - last_move.to as i32,
                    string: last_move_notation.into(),
                });
            } else {
                ui.set_last_move(Move_UI {
                    from_square: last_move.from as i32,
                    to_square: last_move.to as i32,
                    string: last_move_notation.into(),
                });
            }
        }
        ui.set_position(pos.into());
    });

    let ui_weak_make_move = ui.as_weak();
    let board_make_move = board.clone();
    ui.on_make_move(move || -> bool {
        let ui = ui_weak_make_move.upgrade().unwrap();

        let from = ui.get_selected_from_square();
        let to = ui.get_selected_to_square();
        let mut legal_mv: chess::Move = NULL_MOVE;

        for mv in board_make_move
            .lock()
            .unwrap()
            .current_state
            .legal_moves
            .clone()
        {
            // ui indexes are reversed if player is black
            if ui.get_player_colour() == PieceColour_UI::Black {
                if mv.from as i32 == 63 - from && mv.to as i32 == 63 - to {
                    legal_mv = mv;
                }
            } else if mv.from as i32 == from && mv.to as i32 == to {
                legal_mv = mv;
            }
        }
        // make move and return true if successful
        board_make_move.lock().unwrap().make_move(&legal_mv).is_ok()
    });

    let ui_weak_engine_make_move = ui.as_weak();
    let board_engine_make_move = board.clone();
    ui.on_engine_make_move(move || {
        let ui = ui_weak_engine_make_move.clone();
        let bmem: Arc<Mutex<Board>> = board_engine_make_move.clone();
        let depth = ui
            .upgrade()
            .unwrap()
            .get_depth()
            .to_string()
            .parse::<i32>()
            .unwrap();
        std::thread::spawn(move || {
            bmem.lock().unwrap().make_engine_move(depth).unwrap();
            slint::invoke_from_event_loop(move || {
                ui.upgrade().unwrap().invoke_refresh_position();
                ui.upgrade().unwrap().set_engine_made_move(true);
            })
            .unwrap();
        });
    });

    ui.invoke_refresh_position();
    ui.run()
}
