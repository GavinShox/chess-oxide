use std::cmp;

use crate::board::*;
use crate::movegen::*;
use crate::transposition::*;
use crate::util;

// avoid int overflows when operating on these values i.e. negating, +/- checkmate depth etc.
const MIN: i32 = i32::MIN + 1000;
const MAX: i32 = i32::MAX - 1000;
const CHECKMATE_VALUE: i32 = 100_000_000;
const CHECKMATE_THRESHOLD: i32 = CHECKMATE_VALUE - 1000;
const DRAW_VALUE: i32 = 0;
// max depth for quiescence search, best case it should be unlimited (only stopping when there are no more captures), but in practice it takes too long
const QUIECENCE_DEPTH: u8 = 4;

// TODO for tt, to make sure checkmate eval is relative to the ply it was found at, maybe have a checkmate flag in the tt entry or an enum here for evals i dont know
#[inline(always)]
pub const fn is_eval_checkmate(eval: i32) -> bool {
    eval.abs() >= CHECKMATE_THRESHOLD
}

// amount of plys until checkmate
#[inline(always)]
pub const fn get_checkmate_ply(eval: i32) -> u8 {
    (CHECKMATE_VALUE - eval.abs()).unsigned_abs() as u8
}

struct Nodes {
    negamax_nodes: u64,
    negamax_prunes: u64,
    quiescence_nodes: u64,
    quiescence_prunes: u64,
    transposition_table_hits: u64,
}
impl Nodes {
    fn new() -> Self {
        Nodes {
            negamax_nodes: 0,
            negamax_prunes: 0,
            quiescence_nodes: 0,
            quiescence_prunes: 0,
            transposition_table_hits: 0,
        }
    }

    fn total_nodes(&self) -> u64 {
        self.negamax_nodes + self.quiescence_nodes
    }

    fn total_prunes(&self) -> u64 {
        self.negamax_prunes + self.quiescence_prunes
    }
}

pub fn choose_move<'a>(
    bs: &'a BoardState,
    depth: u8,
    tt: &mut TranspositionTable,
) -> (i32, &'a Move) {
    let mut nodes = Nodes::new();
    // TODO add check if position is in endgame, for different evaluation
    let (eval, mv) = negamax_root(bs, depth, tt, &mut nodes);

    if cfg!(feature = "debug_engine_logging") {
        log::info!("Nodes searched: {}", nodes.total_nodes());
        log::info!("Branches pruned: {}", nodes.total_prunes());
        log::info!("Negamax nodes: {}", nodes.negamax_nodes);
        log::info!("Negamax prunes: {}", nodes.negamax_prunes);
        log::info!("Quiescence nodes: {}", nodes.quiescence_nodes);
        log::info!("Quiescence prunes: {}", nodes.quiescence_prunes);
        log::info!(
            "Transposition table hits: {}",
            nodes.transposition_table_hits
        );
    }
    log::debug!(
        "Transposition table: Entries -> {}/{}, Size on heap -> {}",
        tt.len(),
        tt.size(),
        util::bytes_to_str(tt.heap_alloc_size())
    );
    if is_eval_checkmate(eval) {
        log::info!(
            "Engine chose move: {:?} with eval: Mate in {} ply @ depth {}",
            mv,
            get_checkmate_ply(eval),
            depth
        );
    } else {
        log::info!(
            "Engine chose move: {:?} with eval: {} @ depth {}",
            mv,
            eval,
            depth
        );
    }
    (eval, mv)
}

// TODO add checks (and maybe promotions) to quiescence search
fn quiescence(
    bs: &BoardState,
    depth: u8,
    ply: u8,
    mut alpha: i32,
    beta: i32,
    nodes: &mut Nodes,
) -> i32 {
    let pseudo_legal_moves = bs.get_pseudo_legal_moves();
    // check game over conditions returning immediately, or begin quiescence search
    match bs.get_gamestate() {
        GameState::Checkmate => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.quiescence_nodes += 1;
            }
            return -CHECKMATE_VALUE + ply as i32;
        }
        // draw states
        GameState::Stalemate
        | GameState::Repetition
        | GameState::FiftyMove
        | GameState::InsufficientMaterial => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.quiescence_nodes += 1;
            }
            return DRAW_VALUE;
        }
        _ => {}
    }

    let mut max_eval = evaluate(bs);
    if max_eval >= beta || depth == 0 {
        return max_eval;
    }
    alpha = cmp::max(alpha, max_eval);

    for i in sorted_move_indexes(pseudo_legal_moves, true, NULL_SHORT_MOVE, &bs.last_move) {
        let mv = &pseudo_legal_moves[i];
        if !bs.is_move_legal_position(mv) {
            continue; // skip illegal moves
        }
        let child_bs = bs.next_state_unchecked(mv);
        let eval = -quiescence(&child_bs, depth - 1, ply + 1, -beta, -alpha, nodes);
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
    depth: u8,
    tt: &mut TranspositionTable,
    nodes: &mut Nodes,
) -> (i32, &'a Move) {
    let pseudo_legal_moves = bs.get_pseudo_legal_moves();
    // check game over conditions returning immediately, or begin quiescence search
    match bs.get_gamestate() {
        GameState::Checkmate => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_nodes += 1;
            }
            return (-CHECKMATE_VALUE, &NULL_MOVE);
        }
        // draw states
        GameState::Stalemate
        | GameState::Repetition
        | GameState::FiftyMove
        | GameState::InsufficientMaterial => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_nodes += 1;
            }
            return (DRAW_VALUE, &NULL_MOVE); // stalemate
        }
        _ => {}
    }
    let mut alpha = MIN;
    let beta = MAX;
    let mut best_move = &NULL_MOVE;
    let mut max_eval = MIN;
    for i in sorted_move_indexes(pseudo_legal_moves, false, NULL_SHORT_MOVE, &bs.last_move) {
        let mv = &pseudo_legal_moves[i];
        if !bs.is_move_legal_position(mv) {
            continue; // skip illegal moves
        }
        let child_bs = bs.next_state_unchecked(mv);
        let eval = -negamax(&child_bs, depth - 1, 1, -beta, -alpha, tt, nodes);

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

    (max_eval, best_move)
}

fn negamax(
    bs: &BoardState,
    depth: u8,
    ply: u8,
    mut alpha: i32,
    mut beta: i32,
    tt: &mut TranspositionTable,
    nodes: &mut Nodes,
) -> i32 {
    // transposition table lookup
    let alpha_orig = alpha;
    let mut best_move = NULL_SHORT_MOVE; // will be set on tt hit
    if let Some(entry) = tt.get(bs.board_hash) {
        //TODO does adding halfmove count to the hash make sense? test performance
        if cfg!(feature = "debug_engine_logging") {
            nodes.transposition_table_hits += 1;
        }
        if entry.depth >= depth {
            match entry.bound_type {
                BoundType::Exact => {
                    return entry.eval;
                }
                BoundType::Lower => {
                    alpha = cmp::max(alpha, entry.eval);
                }
                BoundType::Upper => {
                    beta = cmp::min(beta, entry.eval);
                }
                BoundType::Invalid => {
                    unreachable!("Invalid bound type returned in transposition table entry");
                }
            }
            if alpha >= beta {
                return entry.eval;
            }
        }
        best_move = entry.mv;
    }
    // TODO checkmate stored in tt will have wrong eval, so the root depth should be recalculated, might need a checkmate flag in tt entry

    let pseudo_legal_moves = bs.get_pseudo_legal_moves();
    // check game over conditions returning immediately, or begin quiescence search
    match bs.get_gamestate() {
        GameState::Checkmate => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_nodes += 1;
            }
            return -CHECKMATE_VALUE + ply as i32;
        }
        // draw states
        GameState::Stalemate
        | GameState::Repetition
        | GameState::FiftyMove
        | GameState::InsufficientMaterial => {
            if cfg!(feature = "debug_engine_logging") {
                nodes.negamax_nodes += 1;
            }
            return DRAW_VALUE; // stalemate
        }
        _ => {}
    }

    if depth == 0 {
        return quiescence(bs, QUIECENCE_DEPTH, ply + 1, alpha, beta, nodes);
    }

    let mut max_eval = MIN;
    let moves = sorted_move_indexes(pseudo_legal_moves, false, best_move, &bs.last_move); // sort pseudo legal moves instead of consuming the lazy iterator
    for i in moves {
        let mv = &pseudo_legal_moves[i];
        if !bs.is_move_legal_position(mv) {
            continue; // skip illegal moves
        }

        let child_bs = bs.next_state_unchecked(mv);
        let eval = -negamax(&child_bs, depth - 1, ply + 1, -beta, -alpha, tt, nodes);
        if eval > max_eval {
            max_eval = eval;
            best_move = mv.short_move();
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

    // Insert new entry in transposition table
    let mut entry = TableEntry {
        bound_type: BoundType::Exact, // set to exact, and change to another bound below if needed
        depth,
        ply,
        eval: max_eval,
        mv: best_move,
    };
    // set bound type to Upper or Lower, otherwise it stays Exact
    if entry.eval <= alpha_orig {
        entry.bound_type = BoundType::Upper;
    } else if entry.eval >= beta {
        entry.bound_type = BoundType::Lower;
    }
    tt.insert(bs.board_hash, entry);

    max_eval
}

fn sorted_move_indexes(
    moves: &[Move],
    captures_only: bool,
    tt_mv: ShortMove,
    last_mv: &Option<Move>,
) -> Vec<usize> {
    let mut move_scores: Vec<(usize, i32)> = Vec::with_capacity(moves.len());

    for (index, mv) in moves.iter().enumerate() {
        if captures_only && !matches!(mv.move_type, MoveType::Capture(_)) {
            continue;
        }
        if mv == &tt_mv {
            move_scores.push((index, MAX)); // tt move should be searched first
            continue;
        }

        let mv_score = match mv.move_type {
            MoveType::Capture(capture_type) => {
                let mv_ptype_value = get_piece_value(&mv.piece.ptype);
                // prioritise captures, even when capturing with a more valuable piece. After trades it could still be good, so min 1
                cmp::max(get_piece_value(&capture_type) - mv_ptype_value, 1)
                    + if let Some(last_mv) = last_mv {
                        // prioritize recaptures, with least valuable piece
                        if mv.to == last_mv.to {
                            10000 - mv_ptype_value
                        } else {
                            0
                        }
                    } else {
                        0
                    }
            }
            MoveType::Promotion(promotion_type, _) => get_piece_value(&promotion_type), // TODO maybe potential capture should be taken into account
            _ => 0,
        };

        move_scores.push((index, mv_score));
    }

    move_scores.sort_by(|a, b| b.1.cmp(&a.1));

    move_scores
        .into_iter()
        .unzip::<_, _, Vec<usize>, Vec<i32>>()
        .0
}

// values in centipawns
#[inline(always)]
const fn get_piece_value(ptype: &PieceType) -> i32 {
    match ptype {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
}

#[inline(always)]
const fn get_piece_pos_value(i: usize, piece: &Piece, is_endgame: bool) -> i32 {
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

    let side_adjusted_idx = match piece.pcolour {
        PieceColour::White => i,
        PieceColour::Black => 63 - i,
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
    }
}

// adapted piece eval scores from here -> https://www.chessprogramming.org/Simplified_Evaluation_Function
fn evaluate(bs: &BoardState) -> i32 {
    let maxi_colour = bs.side_to_move;
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
                    w_eval += val;
                } else {
                    b_eval += val;
                }
            }
        }
    }
    let eval = w_eval - b_eval;
    if maxi_colour == PieceColour::White {
        eval
    } else {
        -eval
    }
}
