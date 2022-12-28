#[cfg(test)]
use super::*;

#[test]
fn test_perft() {
    // https://www.chessprogramming.org/Perft_Results
    // Assert perft results equal those in chessprogramming.org
    let pos1 = Position::new_starting();
    let pos2 = Position::new_position_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let pos3 = Position::new_position_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let pos4 = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let pos4mirrored = Position::new_position_from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1");
    let pos5 = Position::new_position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let pos6 = Position::new_position_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");

    let pos1_nodes = perft(&pos1, 5);
    assert_eq!(pos1_nodes, 4865609);

    let pos2_nodes = perft(&pos2, 4);
    assert_eq!(pos2_nodes, 4085603);

    let pos3_nodes = perft(&pos3, 5);
    assert_eq!(pos3_nodes, 674624);

    let pos4_nodes = perft(&pos4, 5);
    assert_eq!(pos4_nodes, 15833292);

    let pos4mirrored_nodes = perft(&pos4mirrored, 5);
    assert_eq!(pos4mirrored_nodes, 15833292);

    let pos5_nodes = perft(&pos5, 4);
    assert_eq!(pos5_nodes, 2103487);

    let pos6_nodes = perft(&pos6, 4);
    assert_eq!(pos6_nodes, 3894594);
}