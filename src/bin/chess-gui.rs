use std::fmt::format;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use iced::Command;
use iced::color;
use iced::theme::Svg;
use iced::widget::Button;
use iced::Alignment;
use iced::alignment;
use iced::widget::{Row, button, column, text, container, svg, Column, Text, Container, Scrollable};
use iced::Sandbox;
use iced::Application;
use iced::Settings;
use iced::executor;
use chess::*;


const SQUARE_SIZE: u16 = 75;


struct ChessSquare {
    idx: usize,
    is_selected: bool,
    last_move: Move,
    is_legal_move: bool,
}
impl ChessSquare {
    fn from(idx: usize, is_selected: bool, last_move: Move, is_legal_move: bool) -> Self {
        Self {
            idx,
            is_selected,
            last_move,
            is_legal_move
        }
    }

    fn get_bg_colour(&self) -> iced::Color {
        let mut dark_colour = iced::Color{r: 0.125, g: 0.57, b: 0.73, a: 1.0};
        let mut light_colour = iced::Color::WHITE;
        if self.last_move.from == self.idx || self.last_move.to == self.idx {
            dark_colour = iced::Color{r: 0.7, g: 0.5, b: 0.5, a: 1.0};
            light_colour = iced::Color{r: 0.8, g: 0.5, b: 0.5, a: 1.0};
        }
        if self.is_selected {
            dark_colour = iced::Color{r: 1.0, g: 0.5, b: 0.5, a: 1.0};
            light_colour = iced::Color{r: 1.0, g: 0.5, b: 0.5, a: 1.0};
        }
        if self.is_legal_move {
            dark_colour = iced::Color{r: 1.0, g: 0.5, b: 0.5, a: 0.5};
            light_colour = iced::Color{r: 1.0, g: 0.5, b: 0.5, a: 0.5};
        }
        let rank = self.idx / 8;
        if rank % 2 == 0 {
            return if self.idx % 2 == 0 {light_colour} else {dark_colour};
        } else {
            return if self.idx % 2 == 0 {dark_colour} else {light_colour};
        }
        
    }
}

impl button::StyleSheet for ChessSquare {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.get_bg_colour())),
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.get_bg_colour())),
            ..button::Appearance::default()
        }
    }

}


// TODO maybe make this a single widget focused on just the chessboard?
struct ChessBoard {
    // The counter value
    board: board::Board,
    selected: Option<usize>,
    move_made: bool,
    players_move: bool,
    game_state: board::GameState
}


#[derive(Debug, Clone, Copy)]
pub enum ChessBoardMessage {
    Select(usize),
    EngineMove,
    Continue
}

fn get_svg_path(square: &Square) -> String {
    match square {
        Square::Piece(p) => {
            match p.pcolour {
                PieceColour::White => {
                    match p.ptype {
                        PieceType::Pawn => {
                            format!("{}/resources/Chess_plt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Knight => {
                            format!("{}/resources/Chess_nlt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Bishop => {
                            format!("{}/resources/Chess_blt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Rook => {
                            format!("{}/resources/Chess_rlt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Queen => {
                            format!("{}/resources/Chess_qlt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::King => {
                            format!("{}/resources/Chess_klt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::None => {
                            "".to_owned()
                        }
                    }
                }
                PieceColour::Black => {
                    match p.ptype {
                        PieceType::Pawn => {
                            format!("{}/resources/Chess_pdt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Knight => {
                            format!("{}/resources/Chess_ndt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Bishop => {
                            format!("{}/resources/Chess_bdt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Rook => {
                            format!("{}/resources/Chess_rdt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::Queen => {
                            format!("{}/resources/Chess_qdt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::King => {
                            format!("{}/resources/Chess_kdt45.svg", env!("CARGO_MANIFEST_DIR"))
                        }
                        PieceType::None => {
                            "".to_owned()
                        }
                    }
                }
                PieceColour::None => {
                    "".to_owned()
                }
            }
        }
        Square::Empty => {
            "".to_owned()
        }
    }

}


const ENGINE_DEPTH: i32 = 2;

impl ChessBoard {
    fn get_gamestate_text(&self) -> String {
        let game_state = self.board.get_gamestate();
        match game_state {
            board::GameState::Active => {
                if self.players_move {
                    "Player's Move".to_owned()
                } else {
                    "Engine's Move".to_owned()
                }
            }
            board::GameState::Checkmate => {
                if self.players_move {
                    "Checkmate - Engine Wins".to_owned()
                } else {
                    "Checkmate - Player Wins".to_owned()
                }
            }
            board::GameState::Stalemate => {
                "Draw - Stalemate".to_owned()
            }
            board::GameState::Repetition => {
                "Draw - Repetition".to_owned()
            }
            board::GameState::FiftyMove => {
                "Draw - Fifty Move Rule".to_owned()
            }
            board::GameState::Check => {
                if self.players_move {
                    "Check - Player's Move".to_owned()
                } else {
                    "Check - Engine's Move".to_owned()
                }
            }
            board::GameState::InsufficientMaterial => {
                "Draw - Insufficient Material".to_owned()
            }
        }
    }
    
}
#[derive(Debug, Clone, Copy, Default)]
struct ChessBoardTheme;
impl container::StyleSheet for ChessBoardTheme {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            ..container::Appearance::default()
        }
    }
}
const PLAYER_COLOUR: PieceColour = PieceColour::White;

impl Application for ChessBoard {
    type Message = ChessBoardMessage;
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(flags: ()) -> (Self, iced::Command<Self::Message>) {
        let board = board::Board::new();
        (Self {
            board,
            selected: None,
            move_made: false,
            players_move: true,
            game_state: board::GameState::Active
        }, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Chess Oxide")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message>{
        match message {
            ChessBoardMessage::Select(idx) => {
                if self.players_move {
                    if self.selected.is_none() {
                        self.selected = Some(idx);
                    } else {
                        let from = self.selected.unwrap();
                        let to = idx;
                        self.selected = None;
                        if self.players_move {
                            let mut play_mv = NULL_MOVE;
                            for mv in self.board.current_state.legal_moves.iter() {
                                if mv.from == from && mv.to == to {
                                    play_mv = *mv;
                                    break;
                                }
                            }
                            if play_mv != NULL_MOVE {
                                self.board.make_move(&play_mv);
                                self.game_state = self.board.current_state.get_gamestate();
                                self.players_move = false;
                                return Command::perform(async {}, |()| ChessBoardMessage::EngineMove);
                            } else {
                                self.selected = Some(idx);
                            }
    
                        }
                    }
                }
                
            }
            ChessBoardMessage::EngineMove => {
                self.game_state = self.board.make_engine_move(4).unwrap(); //*engine::choose_move(&self.board.current_state.position, ENGINE_DEPTH);
                self.players_move = true;
            },
            ChessBoardMessage::Continue => {},
        }


        iced::Command::none()
    }
    

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let mut chess_board = Column::new().spacing(0).align_items(Alignment::Center);
        // chess_board = chess_board.push(Text::new(self.get_gamestate_text()).size(20));
        let mut row = Row::new().spacing(0).align_items(Alignment::Center);
        let mut legal_moves_selected = Vec::new();
        if self.selected.is_some() {
            let from = self.selected.unwrap();
            for mv in self.board.current_state.legal_moves.iter() {
                if mv.from == from {
                    legal_moves_selected.push(mv.to);
                }
            }
        }
        for i in 0..64 {
            // only highlight players own pieces
            let is_selected = if let Square::Piece(p) = self.board.current_state.get_pos64()[i] {
                if p.pcolour == PLAYER_COLOUR {
                    self.selected.unwrap_or(i+1) == i
                } else {
                    false
                }
            } else {
                false
            };


            let svg = svg::Svg::from_path(get_svg_path(&self.board.current_state.get_pos64()[i]));
            
            row = row.push(
                Button::new(
                    svg
                    .width(iced::Length::Fill).height(iced::Length::Fill)

                )
                .on_press(ChessBoardMessage::Select(i))
                .width(SQUARE_SIZE)
                .height(SQUARE_SIZE)
                .style(iced::theme::Button::Custom(Box::new(ChessSquare::from(i, is_selected, self.board.current_state.last_move, legal_moves_selected.contains(&i)))))
            );
            if (i+1) % 8 == 0 {
                chess_board = chess_board.push(row);
                row = Row::new().spacing(0).align_items(Alignment::Center);
            }
        }
        
        Container::new(
            Row::new().push(
                Column::new()
                .push(Text::new(self.get_gamestate_text()).size(20))
                .spacing(10)
                .align_items(iced::Alignment::Center)
                .push(
                    Container::new(chess_board)
                    .width(iced::Length::Shrink)
                    .height(iced::Length::Shrink)
                    // .style(iced::theme::Container::Custom(Box::new(ChessBoardTheme)))
                    .padding(5)
                    .center_x()
                    .center_y()    
                )
                .align_items(iced::Alignment::Center)
                .spacing(10)
            )
            .spacing(20)
            .push(
                Scrollable::new(
                    self.board.state_history
                    .iter()
                    .fold(Column::new(), |column, state| {
                        column.push(
                            Text::new(

                                if state.side_to_move == PieceColour::Black && state.last_move != NULL_MOVE {
                                    format!("{}. {}", state.get_move_count(), state.last_move_as_notation().unwrap())
                                } else if state.side_to_move == PieceColour::White && state.last_move != NULL_MOVE {
                                    format!("{}", state.last_move_as_notation().unwrap())
                                } else {
                                    String::from("Move History:")
                                }
                            )
                        )
                    })
                )
                .height(SQUARE_SIZE * 8)
            )
            .align_items(Alignment::End)
        )
        .center_x()
        .center_y()
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()

    }
}
fn main() -> iced::Result {
    // let pos = Position::new_starting();
    // //let mut pos = Position::new_position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    // //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    // pos.print_board();

    // perft(&pos, 5);
    ChessBoard::run(Settings {
        window: iced::window::Settings {
            //size: ((SQUARE_SIZE as u32 * 8) + 120, (SQUARE_SIZE as u32 * 8) + 100),
            resizable: true,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}