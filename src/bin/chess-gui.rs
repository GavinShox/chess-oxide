use std::env;
use std::sync::{Arc, Mutex};

use env_logger::{Builder, Target};
use slint::{ComponentHandle, SharedString};

use chess::fen::FEN;
use chess::hash_to_string;
use chess::pgn::PGN;

slint::include_modules!();

type PieceUI = slint_generatedBoard_UI::Piece_UI;
type PieceColourUI = slint_generatedBoard_UI::PieceColour_UI;
type PieceTypeUI = slint_generatedBoard_UI::PieceType_UI;
type MoveNotationUI = slint_generatedBoard_UI::MoveNotation_UI;
//type MoveUI = slint_generatedBoard_UI::Move_UI;

fn ui_convert_piece_colour(colour: chess::PieceColour) -> PieceColourUI {
    match colour {
        chess::PieceColour::White => PieceColourUI::White,
        chess::PieceColour::Black => PieceColourUI::Black,
    }
}

fn ui_convert_piece(piece: chess::Piece) -> PieceUI {
    let piece_colour = match piece.pcolour {
        chess::PieceColour::White => PieceColourUI::White,
        chess::PieceColour::Black => PieceColourUI::Black,
    };

    let piece_type = match piece.ptype {
        chess::PieceType::Pawn => PieceTypeUI::Pawn,
        chess::PieceType::Bishop => PieceTypeUI::Bishop,
        chess::PieceType::Knight => PieceTypeUI::Knight,
        chess::PieceType::Rook => PieceTypeUI::Rook,
        chess::PieceType::Queen => PieceTypeUI::Queen,
        chess::PieceType::King => PieceTypeUI::King,
    };

    PieceUI {
        piece_colour,
        piece_type,
    }
}

fn main() -> Result<(), slint::PlatformError> {
    // board::Board::from_fen("8/8/8/5R2/8/P1P3PP/P2QPP2/k5K1 b - - 0 1"

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let board = Arc::new(Mutex::new(chess::Board::new()));

    let ui = Board_UI::new()?;
    let import_fen_dialog = ImportFen_UI::new()?;
    let settings_dialog = SettingsDialog_UI::new()?;
    let import_pgn_dialog = ImportPGN_UI::new()?;

    let ui_weak_get_gamestate = ui.as_weak();
    let board_get_gamestate = board.clone();
    ui.on_get_gamestate(move || {
        let ui = ui_weak_get_gamestate.upgrade().unwrap();
        let board = board_get_gamestate.lock().unwrap();
        let side_to_move = if board.get_current_state().side_to_move == chess::PieceColour::White {
            "White"
        } else {
            "Black"
        };
        let gamestate = board.get_current_state().get_gamestate().to_string();
        ui.set_gamestate(format!("{}'s turn: {}", side_to_move, gamestate).into());
    });

    let ui_weak_new_game = ui.as_weak();
    let board_new_game = board.clone();
    ui.on_new_game(move || {
        let ui = ui_weak_new_game.upgrade().unwrap();
        *board_new_game.lock().unwrap() = chess::board::Board::new();
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
            .get_current_state()
            .get_pos64()
            .iter()
        {
            match s {
                chess::Square::Piece(p) => ui_position.push(ui_convert_piece(*p)),
                chess::Square::Empty => ui_position.push(PieceUI {
                    piece_colour: PieceColourUI::None,
                    piece_type: PieceTypeUI::None,
                }),
            }
        }
        // reverse board if player is black
        if ui.get_player_colour() == PieceColour_UI::Black {
            ui_position.reverse();
        }
        let pos = std::rc::Rc::new(slint::VecModel::from(ui_position));

        // generate move history as vector of move notations
        let ui_moves: Vec<SharedString> = board_refresh_position
            .lock()
            .unwrap()
            .move_history_string_notation()
            .iter()
            .map(|x| x.into())
            .collect();

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
                .get_current_state()
                .to_fen()
                .to_string()
                .into(),
        );
        log::debug!(
            "FEN: {} generated from boardstate with hash: {}",
            ui.get_fen(),
            hash_to_string(
                board_refresh_position
                    .lock()
                    .unwrap()
                    .get_current_state()
                    .board_hash
            )
        );
        log::debug!(
            "Current position hash: {}",
            hash_to_string(
                board_refresh_position
                    .lock()
                    .unwrap()
                    .get_current_state()
                    .position_hash
            )
        );

        // only set last move in GUI if it is not NULL_MOVE, then unwrap() is safe
        if board_refresh_position
            .lock()
            .unwrap()
            .get_current_state()
            .last_move
            != chess::NULL_MOVE
        {
            let last_move = board_refresh_position
                .lock()
                .unwrap()
                .get_current_state()
                .last_move;

            if ui.get_player_colour() == PieceColour_UI::Black {
                // reverse index if player is black as the board is flipped
                ui.set_last_move(Move_UI {
                    from_square: 63 - last_move.from as i32,
                    to_square: 63 - last_move.to as i32,
                });
            } else {
                ui.set_last_move(Move_UI {
                    from_square: last_move.from as i32,
                    to_square: last_move.to as i32,
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
        let mut legal_mv: chess::Move = chess::NULL_MOVE;

        for mv in board_make_move
            .lock()
            .unwrap()
            .get_current_state()
            .get_legal_moves()
            .unwrap()
        // unwrap is safe as we are not using lazy legal move generation
        {
            // ui indexes are reversed if player is black
            if ui.get_player_colour() == PieceColour_UI::Black {
                if mv.from as i32 == 63 - from && mv.to as i32 == 63 - to {
                    legal_mv = *mv;
                }
            } else if mv.from as i32 == from && mv.to as i32 == to {
                legal_mv = *mv;
            }
        }
        // make move and return true if successful
        board_make_move.lock().unwrap().make_move(&legal_mv).is_ok()
    });

    let ui_weak_engine_make_move = ui.as_weak();
    let board_engine_make_move = board.clone();
    ui.on_engine_make_move(move || {
        let ui = ui_weak_engine_make_move.clone();
        let bmem: Arc<Mutex<chess::Board>> = board_engine_make_move.clone();
        let depth = ui
            .upgrade()
            .unwrap()
            .get_depth()
            .to_string()
            .parse::<i32>()
            .unwrap();
        std::thread::spawn(move || {
            if let Err(e) = bmem.lock().unwrap().make_engine_move(depth as u8) {
                log::error!("BoardStateError on making engine move: {e}");
                return;
            }
            slint::invoke_from_event_loop(move || {
                ui.upgrade().unwrap().invoke_refresh_position();
                ui.upgrade().unwrap().set_engine_made_move(true);
            })
            .unwrap();
        });
    });

    let import_fen_dialog_weak_run = import_fen_dialog.as_weak();
    ui.on_import_fen_dialog(move || {
        let import_fen_dialog = import_fen_dialog_weak_run.upgrade().unwrap();
        import_fen_dialog.show().unwrap();
    });

    // close all child dialogs/windows on main window close
    let import_fen_dialog_weak_close = import_fen_dialog.as_weak();
    let settings_dialog_weak_close = settings_dialog.as_weak();
    ui.window()
        .on_close_requested(move || -> slint::CloseRequestResponse {
            let import_fen_dialog = import_fen_dialog_weak_close.upgrade().unwrap();
            let settings_dialog = settings_dialog_weak_close.upgrade().unwrap();
            import_fen_dialog.hide().unwrap();
            settings_dialog.hide().unwrap();
            slint::CloseRequestResponse::HideWindow
        });

    let ui_weak_import_fen = ui.as_weak();
    let import_fen_dialog_weak_import = import_fen_dialog.as_weak();
    let board_import_fen = board.clone();
    import_fen_dialog.on_import_fen(move |fen: SharedString| {
        let import_fen_dialog = import_fen_dialog_weak_import.upgrade().unwrap();
        let ui = ui_weak_import_fen.upgrade().unwrap();

        let new_board = chess::board::Board::from_fen(match &FEN::from_str(&fen.trim()) {
            Ok(f) => {
                import_fen_dialog.set_error(false);
                import_fen_dialog.set_fen_str("".into());
                f
            }
            Err(e) => {
                import_fen_dialog.set_error(true);
                import_fen_dialog.set_error_message(e.to_string().into());
                return;
            }
        });

        let side_to_move = ui_convert_piece_colour(new_board.get_current_state().side_to_move);
        let player_side = if import_fen_dialog.get_as_white() {
            PieceColour_UI::White
        } else {
            PieceColour_UI::Black
        };

        *board_import_fen.lock().unwrap() = new_board;

        ui.invoke_reset_properties(player_side, side_to_move);
        ui.invoke_refresh_position();
        import_fen_dialog.hide().unwrap();
    });

    let import_fen_dialog_weak_close = import_fen_dialog.as_weak();
    import_fen_dialog.on_close(move || {
        let import_fen_dialog = import_fen_dialog_weak_close.upgrade().unwrap();
        import_fen_dialog.set_error(false);
        import_fen_dialog.set_error_message("".into());
        import_fen_dialog.set_fen_str("".into());
        import_fen_dialog.hide().unwrap();
    });

    // on close window, invoke on_close to reset state
    let import_fen_dialog_weak_close_requested = import_fen_dialog.as_weak();
    import_fen_dialog
        .window()
        .on_close_requested(move || -> slint::CloseRequestResponse {
            let import_fen_dialog = import_fen_dialog_weak_close_requested.upgrade().unwrap();
            import_fen_dialog.invoke_close();
            slint::CloseRequestResponse::HideWindow
        });

    let settings_dialog_weak_run = settings_dialog.as_weak();
    ui.on_settings_dialog(move || {
        let settings_dialog = settings_dialog_weak_run.upgrade().unwrap();
        settings_dialog.show().unwrap();
    });

    let ui_weak_set_theme = ui.as_weak();
    settings_dialog.on_set_theme(move |theme| {
        let ui = ui_weak_set_theme.upgrade().unwrap();
        ui.set_board_theme(theme);
    });

    let ui_weak_set_depth = ui.as_weak();
    settings_dialog.on_set_depth(move |depth| {
        let ui = ui_weak_set_depth.upgrade().unwrap();
        ui.set_depth(depth);
    });

    let ui_weak_set_piece_theme = ui.as_weak();
    settings_dialog.on_set_piece_theme(move |theme| {
        let ui = ui_weak_set_piece_theme.upgrade().unwrap();
        ui.set_piece_theme(theme);
    });

    let settings_dialog_weak_close = settings_dialog.as_weak();
    settings_dialog.on_close(move || {
        let settings_dialog = settings_dialog_weak_close.upgrade().unwrap();
        settings_dialog.hide().unwrap();
    });

    let import_pgn_dialog_weak_run = import_pgn_dialog.as_weak();
    ui.on_import_pgn_dialog(move || {
        let import_pgn_dialog = import_pgn_dialog_weak_run.upgrade().unwrap();
        import_pgn_dialog.show().unwrap();
    });

    let ui_weak_import_pgn = ui.as_weak();
    let import_pgn_dialog_weak_import_pgn = import_pgn_dialog.as_weak();
    let board_import_pgn = board.clone();
    import_pgn_dialog.on_import_pgn(move |pgn: SharedString| {
        let import_pgn_dialog = import_pgn_dialog_weak_import_pgn.upgrade().unwrap();
        let ui = ui_weak_import_pgn.upgrade().unwrap();

        log::debug!("Importing PGN: {}", pgn);

        let pgn_import = PGN::from_str(pgn.as_str());
        match pgn_import {
            Ok(p) => {
                log::debug!("Successfully parsed PGN: {:?}", p);
                let new_board = chess::board::Board::from_pgn(&p);
                match new_board {
                    Ok(b) => {
                        log::debug!("Successfully created board from PGN");
                        import_pgn_dialog.set_error(false);
                        import_pgn_dialog.set_error_message("".into());
                        log::debug!("Resetting UI properties and refreshing position");
                        let side = b.get_side_to_move();
                        *board_import_pgn.lock().unwrap() = b;
                        // TODO for now set both to sidetomove so engine doesnt make move
                        ui.invoke_reset_properties(
                            ui_convert_piece_colour(side),
                            ui_convert_piece_colour(side),
                        );
                        ui.invoke_refresh_position();
                        import_pgn_dialog.hide().unwrap();
                    }
                    Err(e) => {
                        log::error!("Error creating board from PGN: {}", e);
                        import_pgn_dialog.set_error(true);
                        import_pgn_dialog.set_error_message(e.to_string().into());
                        return;
                    }
                }
            }
            Err(e) => {
                log::error!("Error parsing PGN: {}", e);
                import_pgn_dialog.set_error(true);
                import_pgn_dialog.set_error_message(e.to_string().into());
                return;
            }
        }
    });

    let import_pgn_dialog_weak_close = import_pgn_dialog.as_weak();
    import_pgn_dialog.on_close(move || {
        let import_pgn_dialog = import_pgn_dialog_weak_close.upgrade().unwrap();
        import_pgn_dialog.set_error(false);
        import_pgn_dialog.set_error_message("".into());
        import_pgn_dialog.set_pgn_str("sssssssssssssssssssssssss".into());
        import_pgn_dialog.hide().unwrap();
    });

    // on close window, invoke on_close to reset state
    let import_pgn_dialog_weak_close_requested = import_pgn_dialog.as_weak();
    import_pgn_dialog
        .window()
        .on_close_requested(move || -> slint::CloseRequestResponse {
            let import_pgn_dialog = import_pgn_dialog_weak_close_requested.upgrade().unwrap();
            import_pgn_dialog.invoke_close();
            slint::CloseRequestResponse::HideWindow
        });

    ui.invoke_refresh_position();
    ui.run()
}
