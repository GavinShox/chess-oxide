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
use iced::widget::row;
use iced::widget::{
    Row,
    button,
    column,
    text,
    container,
    svg,
    Column,
    Text,
    Container,
    Scrollable,
};

use iced::widget::scrollable::*;
use iced::Sandbox;
use iced::Application;
use iced::Settings;
use iced::executor;
use chess::*;

const SQUARE_SIZE: u16 = 75;

struct EnginePlayer {
    depth: i32,
}

impl Player for EnginePlayer {
    fn get_move(&self, board_state: &BoardState) -> Move {
        *choose_move(&board_state.position, self.depth).1
    }
}

struct RandomPlayer;
impl Player for RandomPlayer {
    fn get_move(&self, bstate: &BoardState) -> Move {
        *random_move(&bstate.position)
    }
}

#[derive(Default)]
struct HumanPlayer {
    from: Option<usize>,
    to: Option<usize>,
}
impl board::Player for HumanPlayer {
    fn get_move(&self, board_state: &board::BoardState) -> movegen::Move {
        for mv in &board_state.legal_moves {
            if
                self.from.is_some() &&
                self.to.is_some() &&
                mv.from == self.from.unwrap() &&
                mv.to == self.to.unwrap()
            {
                return *mv;
            }
        }
        movegen::NULL_MOVE
    }
}
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
            is_legal_move,
        }
    }

    fn get_bg_colour(&self) -> iced::Color {
        let mut dark_colour = iced::Color { r: 0.125, g: 0.57, b: 0.73, a: 1.0 };
        let mut light_colour = iced::Color::WHITE;
        if self.last_move.from == self.idx || self.last_move.to == self.idx {
            dark_colour = iced::Color { r: 0.5, g: 0.6, b: 0.7, a: 1.0 };
            light_colour = iced::Color { r: 0.5, g: 0.6, b: 0.8, a: 1.0 };
        }
        if self.is_legal_move {
            dark_colour = iced::Color { r: 0.7, g: 0.5, b: 0.5, a: 1.0 };
            light_colour = iced::Color { r: 0.8, g: 0.5, b: 0.5, a: 1.0 };
        }
        if self.is_selected {
            dark_colour = iced::Color { r: 1.0, g: 0.5, b: 0.5, a: 1.0 };
            light_colour = iced::Color { r: 1.0, g: 0.5, b: 0.5, a: 1.0 };
        }
        let rank = self.idx / 8;
        if rank % 2 == 0 {
            return if self.idx % 2 == 0 { light_colour } else { dark_colour };
        } else {
            return if self.idx % 2 == 0 { dark_colour } else { light_colour };
        }
    }
}

impl button::StyleSheet for ChessSquare {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.get_bg_colour())),
            border_radius: 0.0,
            border_width: 0.0,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.get_bg_colour())),
            border_radius: 0.0,
            border_width: 2.0,
            border_color: iced::Color { a: 0.7, ..iced::Color::BLACK },
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
    game_state: board::GameState,
    state_idx: usize,
    engine_eval: f32,
    scrollable_id: iced::widget::scrollable::Id
}

#[derive(Debug, Clone, Copy)]
pub enum ChessBoardMessage {
    Select(usize),
    EngineMove,
    Continue,
    Previous,
    Next,
    GoStart,
    GoEnd,
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
                        PieceType::None => { "".to_owned() }
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
                        PieceType::None => { "".to_owned() }
                    }
                }
                PieceColour::None => { "".to_owned() }
            }
        }
        Square::Empty => { "".to_owned() }
    }
}

const ENGINE_DEPTH: i32 = 5;

impl ChessBoard {
    fn get_gamestate_text(&self) -> String {
        let game_state = self.board.get_gamestate();
        match game_state {
            board::GameState::Active => {
                if self.players_move {
                    "Player's Move".to_owned()
                } else {
                    "Engine Thinking...".to_owned()
                }
            }
            board::GameState::Checkmate => {
                if self.players_move {
                    "Checkmate - Engine Wins".to_owned()
                } else {
                    "Checkmate - Player Wins".to_owned()
                }
            }
            board::GameState::Stalemate => { "Draw - Stalemate".to_owned() }
            board::GameState::Repetition => { "Draw - Repetition".to_owned() }
            board::GameState::FiftyMove => { "Draw - Fifty Move Rule".to_owned() }
            board::GameState::Check => {
                if self.players_move {
                    "Check - Player's Move".to_owned()
                } else {
                    "Check - Engine Thinking...".to_owned()
                }
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
            border_radius: 0.0,
            border_width: 5.0,
            border_color: iced::Color::BLACK,
            ..container::Appearance::default()
        }
    }
}
const PLAYER_COLOUR: PieceColour = PieceColour::White;


struct ApplicationStyle;
impl iced::application::StyleSheet for ApplicationStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> iced::application::Appearance {
        iced::application::Appearance {
            background_color: iced::Color::from_rgb8(112,128,144),
            text_color: iced::Color::BLACK,
        }
    }
}

impl Application for ChessBoard {
    type Message = ChessBoardMessage;
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(flags: ()) -> (Self, iced::Command<Self::Message>) {
        let board = board::Board::new(
            Box::new(HumanPlayer { from: None, to: None }),
            Box::new(HumanPlayer { from: None, to: None })
        );
        (
            Self {
                board,
                selected: None,
                move_made: false,
                players_move: true,
                game_state: board::GameState::Active,
                state_idx: 0,
                engine_eval: 0.0,
                scrollable_id: iced::widget::scrollable::Id::new("history"),
            },
            Command::none()
            // Command::perform(
            //     async {},
            //     |()| ChessBoardMessage::EngineMove
            // )
        )
    }

    fn title(&self) -> String {
        String::from("Chess Oxide Engine")
    }
    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        <Self::Theme as iced::application::StyleSheet>::Style::Custom(Box::new(ApplicationStyle))
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            ChessBoardMessage::Select(idx) => {
                if self.players_move {
                    if self.selected.is_none() {
                        self.selected = Some(idx);
                    } else {
                        let from = self.selected.unwrap();
                        let to = idx;
                        self.selected = None;
                        if from == to {
                            return Command::none();
                        }

                        if self.players_move {
                            let mut play_mv = NULL_MOVE;
                            for mv in self.board.current_state.legal_moves.iter() {
                                if mv.from == from && mv.to == to {
                                    play_mv = *mv;
                                    break;
                                }
                            }
                            println!("Player move: {:?}", play_mv);
                            if play_mv != NULL_MOVE {
                                self.board.make_move(&play_mv);
                                self.game_state = self.board.current_state.get_gamestate();
                                self.players_move = false;
                                self.state_idx += 1;
                                return Command::perform(
                                    async {},
                                    |()| ChessBoardMessage::EngineMove
                                );
                            } else {
                                self.selected = Some(idx);
                            }
                        }
                    }
                }
            }
            ChessBoardMessage::EngineMove => {
                let engine_choice = engine::choose_move(
                    &self.board.current_state.position,
                    ENGINE_DEPTH
                );
                let engine_mv = *engine_choice.1;
                self.engine_eval = if PLAYER_COLOUR == PieceColour::White {-engine_choice.0 as f32} else {engine_choice.0 as f32} ;

                println!("Engine move: {:?}", engine_mv);
                if engine_mv != NULL_MOVE {
                    self.board.make_move(&engine_mv);
                    self.players_move = true;
                    self.state_idx += 1;
                }
                self.game_state = self.board.current_state.get_gamestate();
            }
            ChessBoardMessage::Continue => {}
            ChessBoardMessage::Previous => {
                if self.state_idx > 0 {
                    self.selected = None;
                    self.state_idx -= 1;
                    self.board.current_state = self.board.state_history[self.state_idx].clone();
                    self.players_move = self.state_idx % 2 == 0;
                    self.game_state = self.board.current_state.get_gamestate();
                }
            }
            ChessBoardMessage::Next => {
                if self.state_idx < self.board.state_history.len() - 1 {
                    self.selected = None;
                    self.state_idx += 1;
                    self.board.current_state = self.board.state_history[self.state_idx].clone();
                    self.players_move = self.state_idx % 2 == 0;
                    self.game_state = self.board.current_state.get_gamestate();
                }
            }
            ChessBoardMessage::GoStart => {
                self.selected = None;
                self.state_idx = 0;
                self.board.current_state = self.board.state_history[self.state_idx].clone();
                self.players_move = self.state_idx % 2 == 0;
                self.game_state = self.board.current_state.get_gamestate();
            }
            ChessBoardMessage::GoEnd => {
                self.selected = None;
                self.state_idx = self.board.state_history.len() - 1;
                self.board.current_state = self.board.state_history[self.state_idx].clone();
                self.players_move = self.state_idx % 2 == 0;
                self.game_state = self.board.current_state.get_gamestate();
            }
        }
        iced::widget::scrollable::snap_to::<Self::Message>(self.scrollable_id.clone(), 100.0)
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
            let is_selected = if
                let Square::Piece(p) = self.board.current_state.position.position[i]
            {
                if p.pcolour == PLAYER_COLOUR { self.selected.unwrap_or(i + 1) == i } else { false }
            } else {
                false
            };

            let svg = svg::Svg::from_path(
                get_svg_path(&self.board.current_state.position.position[i])
            );
            let mut square = Button::new(
                svg
                    .style(iced::theme::Svg::Default)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
            )
                .width(iced::Length::Units(SQUARE_SIZE))
                .height(iced::Length::Units(SQUARE_SIZE))
                .style(
                    iced::theme::Button::Custom(
                        Box::new(
                            ChessSquare::from(
                                i,
                                is_selected,
                                self.board.current_state.last_move,
                                legal_moves_selected.contains(&i)
                            )
                        )
                    )
                );
                // disable board if browsing history
                if self.state_idx == self.board.state_history.len() - 1 {
                    square = square.on_press(ChessBoardMessage::Select(i));

                }
            row = row.push(
                square
            );
            if (i + 1) % 8 == 0 {
                chess_board = chess_board.push(row);
                row = Row::new().spacing(0).align_items(Alignment::Center);
            }
        }

        Container::new(
            Row::new()
                .push(
                    Column::new()
                        .push(Text::new(self.get_gamestate_text()).size(26))
                        .spacing(10)
                        .push(Text::new(format!("Engine Evaluation: {}{}", if self.engine_eval > 0.0 {"+"} else {""},(self.engine_eval / 100.0))).size(20))
                        .spacing(10)
                        .align_items(iced::Alignment::Center)
                        .push(
                            Container::new(chess_board)
                                .width(iced::Length::Shrink)
                                .height(iced::Length::Shrink)
                                .style(iced::theme::Container::Custom(Box::new(ChessBoardTheme)))
                                .padding(5)
                                .center_x()
                                .center_y()
                        )
                        .align_items(iced::Alignment::Center)
                        .spacing(10)
                        .push(
                            row!(
                                Button::new("<<").on_press(ChessBoardMessage::GoStart),
                                Button::new("<").on_press(ChessBoardMessage::Previous),
                                Button::new(">").on_press(ChessBoardMessage::Next),
                                Button::new(">>").on_press(ChessBoardMessage::GoEnd)
                            ).spacing(5)
                        )
                )
                .spacing(20)
                .push(
                    column!(
                        Text::new("Move History: ").size(40),
                        Scrollable::new(
                            column!(
                                
                                self.board.state_history
                                    .iter()
                                    .fold(Column::new(), |column, state| {
                                        column.push(
                                            Text::new(
                                                if
                                                    state.side_to_move == PieceColour::Black &&
                                                    state.last_move != NULL_MOVE
                                                {
                                                    format!(
                                                        "{}.\n {}",
                                                        state.move_count + 1,
                                                        state.last_move_as_notation()
                                                    )
                                                } else if
                                                    state.side_to_move == PieceColour::White &&
                                                    state.last_move != NULL_MOVE
                                                {
                                                    state.last_move_as_notation()

                                                } else {
                                                    String::from("")
                                                }
                                            ).style(
                                                // highlight current state
                                                if
                                                    Rc::ptr_eq(
                                                        &self.board.state_history[self.state_idx],
                                                        state
                                                    )
                                                {
                                                    iced::theme::Text::Color(
                                                        iced::Color::from_rgb(1.0, 0.0, 0.0)
                                                    )
                                                } else {
                                                    iced::theme::Text::Default
                                                }
                                            ).size(24)
                                        )

                                    })
                            )
                            .padding(20)
                            .width(iced::Length::Units(125))
                            .align_items(iced::Alignment::Center)
                        ).height(iced::Length::Units(SQUARE_SIZE * 8)).id(self.scrollable_id.clone()).scroller_width(1).scrollbar_width(1).scrollbar_margin(0)
                    )
                    .spacing(10)
                    
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
            // icon: Some(iced::window::Icon::from_file(format!("{}/resources/chesslogo.png", env!("CARGO_MANIFEST_DIR"))).unwrap()),
            ..iced::window::Settings::default()
        },
        
        ..Settings::default()
    })
}