use std::cmp;

use crate::board::*;
use crate::movegen::*;
use crate::PositionHash;

// avoid int overflows when operating on these values i.e. negating, +/- checkmate depth etc.
const MIN: i32 = i32::MIN + 1000;
const MAX: i32 = i32::MAX - 1000;
const QUIECENCE_DEPTH: i32 = 4;

// #[cfg(feature = "debug_engine_logging")]
struct Nodes {
    nodes_searched: u64,
    branches_pruned: u64,
    negamax_nodes: u64,
    negamax_prunes: u64,
    quiescence_nodes: u64,
    quiescence_prunes: u64,
}

#[derive(Debug)]
enum BoundType {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug)]
pub struct TranspositionTable {
    table: ahash::AHashMap<PositionHash, (BoundType, i32, i32)>,
}
impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: ahash::AHashMap::default(),
        }
    }

    fn insert(&mut self, hash: PositionHash, bound_type: BoundType, depth: i32, eval: i32) {
        self.table.insert(hash, (bound_type, depth, eval));
    }

    fn get(&self, hash: PositionHash) -> Option<&(BoundType, i32, i32)> {
        self.table.get(&hash)
    }
}

pub fn choose_move<'a>(
    bs: &'a BoardState,
    depth: i32,
    tt: &'a mut TranspositionTable,
) -> (i32, &'a Move) {
    let mut nodes = Nodes {
        nodes_searched: 0,
        branches_pruned: 0,
        negamax_nodes: 0,
        negamax_prunes: 0,
        quiescence_nodes: 0,
        quiescence_prunes: 0,
    };
    // TODO add check if position is in endgame, for different evaluation
    let (eval, mv) = negamax_root(bs, depth, bs.side_to_move, tt, &mut nodes);

    // total up nodes and prunes
    nodes.nodes_searched = nodes.negamax_nodes + nodes.quiescence_nodes;
    nodes.branches_pruned = nodes.negamax_prunes + nodes.quiescence_prunes;

    if cfg!(feature = "debug_engine_logging") {
        log::info!("Nodes searched: {}", nodes.nodes_searched);
        log::info!("Branches pruned: {}", nodes.branches_pruned);
        log::info!("Negamax nodes: {}", nodes.negamax_nodes);
        log::info!("Negamax prunes: {}", nodes.negamax_prunes);
        log::info!("Quiescence nodes: {}", nodes.quiescence_nodes);
        log::info!("Quiescence prunes: {}", nodes.quiescence_prunes);
    }
    log::info!(
        "Engine chose move: {:?} with eval: {} @ depth {}",
        mv,
        eval,
        depth
    );

    (eval, mv)
}

fn quiescence(
    bs: &BoardState,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    maxi_colour: PieceColour,
    nodes: &mut Nodes,
) -> i32 {
    let mut max_eval = evaluate(bs, maxi_colour);
    if max_eval >= beta || depth == 0 {
        return max_eval;
    }
    alpha = cmp::max(alpha, max_eval);
    let moves = &bs.legal_moves;
    for i in sorted_move_indexes(moves, true) {
        let mv = moves[i];
        let child_bs = bs.next_state(&mv).unwrap();
        let eval = -quiescence(&child_bs, depth - 1, -beta, -alpha, !maxi_colour, nodes);
        max_eval = cmp::max(max_eval, eval);
        alpha = cmp::max(alpha, max_eval);

        if cfg!(feature = "debug_engine_logging") {
            nodes.quiescence_nodes += 1;
        }

        if beta <= alpha {
            if cfg!(feature = "debug_engine_logging") {
                nodes.quiescence_prunes += 1;
            }
            break;
        }
    }
    max_eval
}

fn negamax_root<'a>(
    bs: &'a BoardState,
    depth: i32,
    maxi_colour: PieceColour,
    tt: &'a mut TranspositionTable,
    nodes: &mut Nodes,
) -> (i32, &'a Move) {
    if bs.is_checkmate() {
        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        return (
            if bs.side_to_move == maxi_colour {
                MIN
            } else {
                MAX
            },
            &NULL_MOVE,
        );
    } else if bs.is_draw() {
        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        // stalemate
        return (0, &NULL_MOVE);
    }
    let mut alpha = MIN;
    let beta = MAX;

    let mut best_move = &bs.legal_moves[0];
    let mut max_eval = MIN;
    for i in sorted_move_indexes(&bs.legal_moves, false) {
        let mv = &bs.legal_moves[i];
        // println!("evaluating move: {:?}", mv);
        let child_bs = bs.next_state(mv).unwrap();
        let eval = -negamax(
            &child_bs,
            depth - 1,
            -beta,
            -alpha,
            !maxi_colour,
            1,
            tt,
            nodes,
        );

        if eval > max_eval {
            max_eval = eval;
            best_move = mv;
        }
        alpha = cmp::max(alpha, max_eval);

        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        if beta <= alpha {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_prunes += 1;
            }
            break;
        }
    }
    // println!("root");
    // println!("root eval: {}", max_eval);
    // println!("best move: {:?}", best_move);
    //println!("pos occurences: {}", bs.get_occurences_of_current_position());
    (max_eval, best_move)
}

//todo maybe no need for BoardState here, only for root negamax?
fn negamax(
    bs: &BoardState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maxi_colour: PieceColour,
    root_depth: i32,
    tt: &mut TranspositionTable,
    nodes: &mut Nodes,
) -> i32 {
    // TODO ADD MOVE ORDERING WITH BEST MOVE IN TRANSPOSITION TABLE?
    // transposition table lookup
    let alpha_orig = alpha;
    if let Some((bound_type, tt_depth, tt_eval)) = tt.get(bs.board_hash) {
        let tt_eval = *tt_eval;
        if *tt_depth >= depth {
            match bound_type {
                BoundType::Exact => return tt_eval,
                BoundType::Lower => alpha = cmp::max(alpha, tt_eval),
                BoundType::Upper => beta = cmp::min(beta, tt_eval),
            }
            if alpha >= beta {
                return tt_eval;
            }
        }
    }
    if bs.is_checkmate() {
        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        return if bs.side_to_move == maxi_colour {
            MIN + root_depth
        } else {
            MAX - root_depth
        };
    } else if bs.is_draw() {
        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        return 0; // stalemate
    } else if depth == 0 {
        return quiescence(bs, QUIECENCE_DEPTH, alpha, beta, maxi_colour, nodes);
    }

    let mut max_eval = MIN;
    for i in sorted_move_indexes(&bs.legal_moves, false) {
        let mv = &bs.legal_moves[i];
        let child_bs = bs.next_state(mv).unwrap();
        let eval = -negamax(
            &child_bs,
            depth - 1,
            -beta,
            -alpha,
            !maxi_colour,
            root_depth + 1,
            tt,
            nodes,
        );
        if eval > max_eval {
            max_eval = eval;
        }
        alpha = cmp::max(alpha, max_eval);

        if cfg!(feature = "debug_engine_logging") {
            nodes.negamax_nodes += 1;
        }
        if beta <= alpha {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_prunes += 1;
            }
            break;
        }
    }

    let tt_eval = max_eval;
    if tt_eval <= alpha_orig {
        tt.insert(bs.board_hash, BoundType::Upper, depth, tt_eval);
    } else if tt_eval >= beta {
        tt.insert(bs.board_hash, BoundType::Lower, depth, tt_eval);
    } else {
        tt.insert(bs.board_hash, BoundType::Exact, depth, tt_eval);
    }

    // println!("max_eval: {}", max_eval);

    max_eval
}

fn sorted_move_indexes(moves: &[Move], captures_only: bool) -> Vec<usize> {
    let mut move_scores: Vec<(usize, i32)> = Vec::with_capacity(moves.len());

    for (index, mv) in moves.iter().enumerate() {
        if captures_only && !matches!(mv.move_type, MoveType::Capture(_)) {
            continue;
        }

        let mv_score = match mv.move_type {
            MoveType::Capture(capture_type) => {
                // prioritise captures, even when capturing with a more valuable piece. After trades it could still be good, so min 1
                cmp::max(
                    get_piece_value(&capture_type) - get_piece_value(&mv.piece.ptype),
                    1,
                )
            }
            MoveType::Promotion(promotion_type) => get_piece_value(&promotion_type),
            _ => 0,
        };

        move_scores.push((index, mv_score));
    }

    move_scores.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    move_scores
        .into_iter()
        .unzip::<_, _, Vec<usize>, Vec<i32>>()
        .0
}

// values in centipawns
#[inline(always)]
fn get_piece_value(ptype: &PieceType) -> i32 {
    match ptype {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
        PieceType::None => unreachable!(),
    }
}

#[inline(always)]
fn get_piece_pos_value(i: usize, piece: &Piece, is_endgame: bool) -> i32 {
    // all pos values are from whites perspective (a8 = index 0, h1 = index 63)
    const PAWN_POS_VALUES: [i32; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const KNIGHT_POS_VALUES: [i32; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ];
    const BISHOP_POS_VALUES: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const ROOK_POS_VALUES: [i32; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ];
    const QUEEN_POS_VALUES: [i32; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];
    const KING_MIDDLE_POS_VALUES: [i32; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ];
    const KING_END_POS_VALUES: [i32; 64] = [
        -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
        30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
        -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
        -30, -30, -30, -30, -50,
    ];

    let side_adjusted_idx = if piece.pcolour == PieceColour::White {
        i
    } else {
        63 - i
    };
    match piece.ptype {
        PieceType::Pawn => PAWN_POS_VALUES[side_adjusted_idx],
        PieceType::Knight => KNIGHT_POS_VALUES[side_adjusted_idx],
        PieceType::Bishop => BISHOP_POS_VALUES[side_adjusted_idx],
        PieceType::Rook => ROOK_POS_VALUES[side_adjusted_idx],
        PieceType::Queen => QUEEN_POS_VALUES[side_adjusted_idx],
        PieceType::King => {
            if is_endgame {
                KING_END_POS_VALUES[side_adjusted_idx]
            } else {
                KING_MIDDLE_POS_VALUES[side_adjusted_idx]
            }
        }
        PieceType::None => unreachable!(),
    }
}

// adapted piece eval scores from here -> https://www.chessprogramming.org/Simplified_Evaluation_Function
fn evaluate(bs: &BoardState, maxi_colour: PieceColour) -> i32 {
    let mut w_eval: i32 = 0;
    let mut b_eval: i32 = 0;
    for (i, s) in bs.get_pos64().iter().enumerate() {
        match s {
            Square::Empty => {
                continue;
            }
            Square::Piece(p) => {
                let val = get_piece_value(&p.ptype) + get_piece_pos_value(i, p, false);
                if p.pcolour == PieceColour::White {
                    w_eval += val
                } else {
                    b_eval += val
                };
            }
        }
    }
    let eval = w_eval - b_eval;
    eval * (if maxi_colour == PieceColour::White {
        1
    } else {
        -1
    })
}
