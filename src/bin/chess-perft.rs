use env_logger::{Builder, Target};

fn main() {

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let pos = chess::Position::new_starting();
    let board = chess::board::Board::new();
    //let board = chess::board::Board::from_fen("r1b2rk1/1pp3pp/p2bpq2/3p4/8/2P3P1/PP1BQPBP/3R1RK1 b - - 1 15").unwrap();
    //let mut pos = Position::from_fen_partial_impl("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap().0;
    //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    //r1b2rk1/1pp3pp/p2bpq2/3p4/8/2P3P1/PP1BQPBP/3R1RK b - - 1 15

    chess::perft(&pos, 5);
    chess::engine_perft(&board.current_state, 7);

    // let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    // assert_eq!(board.to_fen(), "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_owned());
}
