//! Rules: legal-move generation, sowing, capture, endelea, nyumba mechanics,
//! phase transitions, win detection. Each branch references a RULES.md
//! section. This module currently implements move generation only; sowing
//! and apply() come in a follow-up step.

use crate::board::{
    next_pit, BoardState, Direction, NyumbaState, Phase, Side, Substate, MBELE_LEN,
    NYUMBA_FUNCTIONAL_THRESHOLD, PITS_PER_SIDE,
};
use crate::events::MoveEvent;
use crate::moves::{KichwaSide, Move};
use crate::variant::Variant;

/// Maximum sow-loop hops; safeguard against any cyclic endelea bug. With at
/// most 64 kete on the board the legitimate maximum is well under this.
const SOW_HOP_LIMIT: u32 = 256;

/// Field where a sow of `count` kete starting at `start` and going in `dir`
/// will drop its last kete. Wraps around the 16-pit ring. See RULES.md §1.3.
pub fn landing(start: u8, dir: Direction, count: u8) -> u8 {
    let mut p = start;
    for _ in 0..count {
        p = next_pit(p, dir);
    }
    p
}

/// All legal moves in the current substate. Empty result implies the active
/// player has lost (handled by win detection in apply()).
pub fn legal_moves(state: &BoardState) -> Vec<Move> {
    match state.phase {
        Phase::Namu(Substate::AwaitMove) => namu_legal_moves(state),
        Phase::Mtaji(Substate::AwaitMove) => mtaji_legal_moves(state),
        Phase::Namu(Substate::AwaitKichwa {
            capture_field,
            prior_dir,
        })
        | Phase::Mtaji(Substate::AwaitKichwa {
            capture_field,
            prior_dir,
        }) => kichwa_legal_moves(capture_field, prior_dir),
        Phase::Namu(Substate::AwaitSafari { .. }) | Phase::Mtaji(Substate::AwaitSafari { .. }) => {
            vec![Move::Safari { go: true }, Move::Safari { go: false }]
        }
    }
}

// ---------- Namu (Kiswahili only) -------------------------------------------------

fn namu_legal_moves(state: &BoardState) -> Vec<Move> {
    debug_assert!(matches!(state.variant, Variant::Kiswahili));
    let own = state.active_side();
    let opp = state.opponent_side();

    // Mandatory-kula: try captures first, only fall back to non-captures.
    let captures = namu_captures(own, opp);
    if !captures.is_empty() {
        return captures;
    }
    namu_non_captures(own, state.variant)
}

/// RULES.md §6.2: namu-kula valid at mbele col `c` when own.mbele[c] >= 1
/// (pre-drop) AND opp.mbele[c] >= 1. Direction is chosen later via kichwa
/// selection; the emitted `dir` is canonical (Cw) and ignored on apply.
fn namu_captures(own: &Side, opp: &Side) -> Vec<Move> {
    let mut out = Vec::new();
    for c in 0..MBELE_LEN as u8 {
        if own.vichwa[c as usize] >= 1 && opp.vichwa[c as usize] >= 1 {
            out.push(Move::Namu {
                col: c,
                dir: Direction::Cw,
            });
        }
    }
    out
}

/// Push both-direction takata moves for column `c`.
fn push_takata(out: &mut Vec<Move>, c: u8) {
    out.push(Move::Namu {
        col: c,
        dir: Direction::Cw,
    });
    out.push(Move::Namu {
        col: c,
        dir: Direction::Ccw,
    });
}

/// RULES.md §8.5: three-branch nyumba-conditional non-capture selection.
fn namu_non_captures(own: &Side, variant: Variant) -> Vec<Move> {
    let mut out = Vec::new();
    let nyumba_state = own.nyumba_state(variant);
    let nyumba_col = own.nyumba_col;

    match nyumba_state {
        NyumbaState::Functional => {
            let mut found_other = false;
            for c in 0..MBELE_LEN as u8 {
                if c == nyumba_col {
                    continue;
                }
                if own.vichwa[c as usize] >= 1 {
                    push_takata(&mut out, c);
                    found_other = true;
                }
            }
            if !found_other && own.vichwa[nyumba_col as usize] >= 1 {
                push_takata(&mut out, nyumba_col);
            }
        }
        NyumbaState::Disabled => {
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 1 {
                    push_takata(&mut out, c);
                }
            }
        }
        NyumbaState::Destroyed => {
            let mut twos = Vec::new();
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 2 {
                    push_takata(&mut twos, c);
                }
            }
            if !twos.is_empty() {
                return twos;
            }
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 1 {
                    push_takata(&mut out, c);
                }
            }
        }
    }
    out
}

// ---------- Mtaji ----------------------------------------------------------------

fn mtaji_legal_moves(state: &BoardState) -> Vec<Move> {
    let own = state.active_side();
    let opp = state.opponent_side();

    let captures = mtaji_captures(own, opp);
    if !captures.is_empty() {
        // TODO RULES.md §11: when active player is the kutakatia-blocker and
        // the blocked field is among capturable fields, restrict to that one.
        return captures;
    }
    mtaji_takata(own)
}

/// RULES.md §6.1 + §4 16-seeds-no-capture: source has 2..=15 kete; landing
/// must be in own mbele (idx 0..7); own.mbele[land] >= 1 (pre-drop) AND
/// opp.mbele[land] >= 1.
fn mtaji_captures(own: &Side, opp: &Side) -> Vec<Move> {
    let mut out = Vec::new();
    for pit in 0..PITS_PER_SIDE as u8 {
        let count = own.vichwa[pit as usize];
        if !(2..=15).contains(&count) {
            continue;
        }
        for &dir in &[Direction::Cw, Direction::Ccw] {
            let land = landing(pit, dir, count);
            if (land as usize) >= MBELE_LEN {
                continue;
            }
            if own.vichwa[land as usize] >= 1 && opp.vichwa[land as usize] >= 1 {
                out.push(Move::Mtaji { pit, dir });
            }
        }
    }
    out
}

/// RULES.md §12.1 (no-singleton): source >=2. Prefer mbele over nyuma per
/// geziefer's `getMtajiPossibleNonCaptures`. RULES.md §12.2.2 (no-suicide):
/// if the only filled mbele pit is a kichwa, sow must go toward center.
fn mtaji_takata(own: &Side) -> Vec<Move> {
    let mbele_with_two: Vec<u8> = (0..MBELE_LEN as u8)
        .filter(|&c| own.vichwa[c as usize] >= 2)
        .collect();

    let mut out = Vec::new();
    if !mbele_with_two.is_empty() {
        let only_one_filled = mbele_with_two.len() == 1;
        for &pit in &mbele_with_two {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                if only_one_filled && is_suicidal_kichwa(pit, dir) {
                    continue;
                }
                out.push(Move::Mtaji { pit, dir });
            }
        }
        return out;
    }

    // No mbele >=2: nyuma sources only (still no-singleton: >=2).
    for pit in MBELE_LEN as u8..PITS_PER_SIDE as u8 {
        if own.vichwa[pit as usize] >= 2 {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                out.push(Move::Mtaji { pit, dir });
            }
        }
    }
    out
}

/// RULES.md §12.2.2: from kichwa pit 0, sow must go toward center (Cw,
/// step +1 → pit 1). From pit 7, must go Ccw (step -1 → pit 6). The
/// suicidal direction is the one stepping into nyuma (off the front row).
fn is_suicidal_kichwa(pit: u8, dir: Direction) -> bool {
    (pit == 0 && matches!(dir, Direction::Ccw)) || (pit == 7 && matches!(dir, Direction::Cw))
}

// ---------- Kichwa selection ----------------------------------------------------

/// RULES.md §6.3 + geziefer `getPossibleKichwas`. capture_field is the mbele
/// index (0..7) where the capture occurred.
fn kichwa_legal_moves(capture_field: u8, prior_dir: Option<Direction>) -> Vec<Move> {
    let cf = capture_field;
    // Left kimbi (cols 0,1) → LEFT only.
    if cf <= 1 {
        return vec![Move::Kichwa(KichwaSide::Left)];
    }
    // Right kimbi (cols 6,7) → RIGHT only.
    if cf >= 6 {
        return vec![Move::Kichwa(KichwaSide::Right)];
    }
    // Middle (cols 2..=5): direction determines kichwa, OR both if no prior.
    match prior_dir {
        Some(Direction::Cw) => vec![Move::Kichwa(KichwaSide::Left)],
        Some(Direction::Ccw) => vec![Move::Kichwa(KichwaSide::Right)],
        None => vec![
            Move::Kichwa(KichwaSide::Left),
            Move::Kichwa(KichwaSide::Right),
        ],
    }
}

// =================================================================================
// Move execution
// =================================================================================

/// Apply a move and return the resulting state and the event stream. Returns
/// `Err` if the move is illegal in the current substate. Pure: input state is
/// untouched.
pub fn apply(
    state: &BoardState,
    mv: Move,
) -> Result<(BoardState, Vec<MoveEvent>), &'static str> {
    if state.winner.is_some() {
        return Err("game is already over");
    }
    let mut next = *state;
    let mut events = Vec::new();
    apply_in_place(&mut next, &mut events, mv)?;
    Ok((next, events))
}

fn apply_in_place(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    mv: Move,
) -> Result<(), &'static str> {
    match (state.phase, mv) {
        (Phase::Namu(Substate::AwaitMove), Move::Namu { col, dir }) => {
            apply_namu(state, events, col, dir)
        }
        (Phase::Mtaji(Substate::AwaitMove), Move::Mtaji { pit, dir }) => {
            apply_mtaji(state, events, pit, dir)
        }
        (
            Phase::Namu(Substate::AwaitKichwa {
                capture_field,
                prior_dir,
            }),
            Move::Kichwa(side),
        )
        | (
            Phase::Mtaji(Substate::AwaitKichwa {
                capture_field,
                prior_dir,
            }),
            Move::Kichwa(side),
        ) => apply_kichwa(state, events, capture_field, prior_dir, side),
        (
            Phase::Namu(Substate::AwaitSafari { sow_dir }),
            Move::Safari { go },
        )
        | (
            Phase::Mtaji(Substate::AwaitSafari { sow_dir }),
            Move::Safari { go },
        ) => apply_safari(state, events, sow_dir, go),
        _ => Err("illegal move for current phase/substate"),
    }
}

// ---------- Namu apply ----------

fn apply_namu(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    col: u8,
    dir: Direction,
) -> Result<(), &'static str> {
    if (col as usize) >= MBELE_LEN {
        return Err("namu col out of mbele");
    }
    let active = state.active as usize;
    let opp = state.opponent(state.active) as usize;
    if state.sides[active].ghala == 0 {
        return Err("ghala empty in namu");
    }

    let is_kula =
        state.sides[active].vichwa[col as usize] >= 1 && state.sides[opp].vichwa[col as usize] >= 1;

    // Place 1 kete from ghala into chosen mbele pit.
    state.sides[active].ghala -= 1;
    state.sides[active].vichwa[col as usize] += 1;
    events.push(MoveEvent::NamuPlace {
        player: state.active,
        pit: col,
    });

    if is_kula {
        // Stop after placement; player must select kichwa (RULES.md §6.3).
        events.push(MoveEvent::KichwaSelectionRequired {
            player: state.active,
            capture_field: col,
        });
        state.phase = Phase::Namu(Substate::AwaitKichwa {
            capture_field: col,
            prior_dir: None,
        });
        return Ok(());
    }

    // Non-capture (takata). Apply tax-rule if source is own functional nyumba
    // (RULES.md §8.4): take only 2 kete instead of full count.
    let nyumba_col = state.sides[active].nyumba_col;
    let post_count = state.sides[active].vichwa[col as usize];
    let was_functional_nyumba = col == nyumba_col
        && state.sides[active].nyumba_owned
        && post_count >= NYUMBA_FUNCTIONAL_THRESHOLD;

    let pickup = if was_functional_nyumba {
        events.push(MoveEvent::Tax {
            player: state.active,
            pit: col,
            taken: 2,
        });
        state.sides[active].vichwa[col as usize] -= 2;
        2
    } else {
        let c = state.sides[active].vichwa[col as usize];
        state.sides[active].vichwa[col as usize] = 0;
        events.push(MoveEvent::Pickup {
            player: state.active,
            pit: col,
            count: c,
        });
        maybe_destroy_own_nyumba(state, active, col, events);
        c
    };

    do_takata_sow(state, events, col, dir, pickup)?;
    end_turn(state, events);
    Ok(())
}

// ---------- Mtaji apply ----------

fn apply_mtaji(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    pit: u8,
    dir: Direction,
) -> Result<(), &'static str> {
    if (pit as usize) >= PITS_PER_SIDE {
        return Err("mtaji pit out of range");
    }
    let active = state.active as usize;
    let opp = state.opponent(state.active) as usize;
    let count = state.sides[active].vichwa[pit as usize];
    if count < 2 {
        return Err("mtaji source < 2 (no-singleton, RULES.md §12.1)");
    }

    let is_capture = (2..=15).contains(&count) && {
        let land = landing(pit, dir, count);
        (land as usize) < MBELE_LEN
            && state.sides[active].vichwa[land as usize] >= 1
            && state.sides[opp].vichwa[land as usize] >= 1
    };

    // Pick up source.
    state.sides[active].vichwa[pit as usize] = 0;
    events.push(MoveEvent::Pickup {
        player: state.active,
        pit,
        count,
    });
    maybe_destroy_own_nyumba(state, active, pit, events);

    if is_capture {
        // Sow `count` kete from `pit` in `dir`, then await kichwa selection.
        let mut hand = count;
        let mut pos = pit;
        let mut hops = 0u32;
        while hand > 0 {
            pos = next_pit(pos, dir);
            state.sides[active].vichwa[pos as usize] += 1;
            events.push(MoveEvent::Sow {
                player: state.active,
                pit: pos,
            });
            hand -= 1;
            hops += 1;
            if hops > SOW_HOP_LIMIT {
                return Err("mtaji-capture sow exceeded hop limit");
            }
        }
        events.push(MoveEvent::KichwaSelectionRequired {
            player: state.active,
            capture_field: pos,
        });
        state.phase = Phase::Mtaji(Substate::AwaitKichwa {
            capture_field: pos,
            prior_dir: Some(dir),
        });
        return Ok(());
    }

    // Mtaji-takata.
    do_takata_sow(state, events, pit, dir, count)?;
    end_turn(state, events);
    Ok(())
}

// ---------- Kichwa apply (capture-sow start) ----------

fn apply_kichwa(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    capture_field: u8,
    prior_dir: Option<Direction>,
    side: KichwaSide,
) -> Result<(), &'static str> {
    let legal = kichwa_legal_moves(capture_field, prior_dir);
    if !legal.contains(&Move::Kichwa(side)) {
        return Err("illegal kichwa selection");
    }

    let opp = state.opponent(state.active) as usize;

    // Capture: empty opponent's mbele[capture_field].
    let captured = state.sides[opp].vichwa[capture_field as usize];
    if captured == 0 {
        return Err("kichwa: opponent capture-field is empty");
    }
    state.sides[opp].vichwa[capture_field as usize] = 0;
    events.push(MoveEvent::Capture {
        from_player: state.opponent(state.active),
        from_pit: capture_field,
        count: captured,
    });
    maybe_destroy_opp_nyumba(state, opp, capture_field, events);

    // Sow from kichwa in the implied direction. Per geziefer, the start
    // position is one step before kichwa so the first loop iteration drops
    // into kichwa itself.
    let sow_dir = side.sow_direction();
    let kichwa = side.pit();
    let start = next_pit(kichwa, sow_dir.reverse());

    do_capture_sow(state, events, start, sow_dir, captured)
}

// ---------- Safari apply ----------

fn apply_safari(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    sow_dir: Direction,
    go: bool,
) -> Result<(), &'static str> {
    if !go {
        end_turn(state, events);
        return Ok(());
    }

    let active = state.active as usize;
    let nyumba = state.sides[active].nyumba_col;
    let count = state.sides[active].vichwa[nyumba as usize];
    state.sides[active].vichwa[nyumba as usize] = 0;
    state.sides[active].nyumba_owned = false;
    events.push(MoveEvent::Pickup {
        player: state.active,
        pit: nyumba,
        count,
    });
    events.push(MoveEvent::NyumbaDestroyed {
        player: state.active,
    });

    do_capture_sow(state, events, nyumba, sow_dir, count)
}

// ---------- Sow loops ----------

/// Takata-sow with endelea (RULES.md §5, §7). No capture detection: per §4
/// "first-lap-determines-captures" a sow that started without a capture
/// trigger never captures. Stops on empty landing or when reaching own
/// functional nyumba (which would otherwise endelea).
fn do_takata_sow(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    source: u8,
    dir: Direction,
    initial_hand: u8,
) -> Result<(), &'static str> {
    let active = state.active as usize;
    let mut pos = source;
    let mut hand = initial_hand;
    let mut hops = 0u32;

    loop {
        while hand > 0 {
            pos = next_pit(pos, dir);
            state.sides[active].vichwa[pos as usize] += 1;
            events.push(MoveEvent::Sow {
                player: state.active,
                pit: pos,
            });
            hand -= 1;
            hops += 1;
            if hops > SOW_HOP_LIMIT {
                return Err("takata-sow exceeded hop limit");
            }
        }

        let landed = state.sides[active].vichwa[pos as usize];
        if landed <= 1 {
            // Empty landing → end of takata.
            return Ok(());
        }

        // Endelea triggers. RULES.md §5.2: stop on own functional nyumba.
        let is_own_func_nyumba = pos == state.sides[active].nyumba_col
            && state.sides[active].nyumba_owned
            && landed >= NYUMBA_FUNCTIONAL_THRESHOLD;
        if is_own_func_nyumba {
            return Ok(());
        }

        // Pickup and continue (RULES.md §7).
        hand = landed;
        state.sides[active].vichwa[pos as usize] = 0;
        events.push(MoveEvent::Pickup {
            player: state.active,
            pit: pos,
            count: hand,
        });
        maybe_destroy_own_nyumba(state, active, pos, events);
    }
}

/// Capture-sow with endelea (RULES.md §5, §6, §7). Mid-sow may trigger
/// another capture (transition to AwaitKichwa) or safari (AwaitSafari).
fn do_capture_sow(
    state: &mut BoardState,
    events: &mut Vec<MoveEvent>,
    start: u8,
    dir: Direction,
    initial_hand: u8,
) -> Result<(), &'static str> {
    let active = state.active as usize;
    let opp = state.opponent(state.active) as usize;
    let mut pos = start;
    let mut hand = initial_hand;
    let mut hops = 0u32;

    loop {
        while hand > 0 {
            pos = next_pit(pos, dir);
            state.sides[active].vichwa[pos as usize] += 1;
            events.push(MoveEvent::Sow {
                player: state.active,
                pit: pos,
            });
            hand -= 1;
            hops += 1;
            if hops > SOW_HOP_LIMIT {
                return Err("capture-sow exceeded hop limit");
            }
        }

        let landed = state.sides[active].vichwa[pos as usize];

        if landed <= 1 {
            // Empty landing → done.
            end_turn(state, events);
            return Ok(());
        }

        // Mid-sow capture trigger (RULES.md §6.1 applied to endelea-laps).
        if (pos as usize) < MBELE_LEN
            && landed >= 2
            && state.sides[opp].vichwa[pos as usize] >= 1
        {
            events.push(MoveEvent::KichwaSelectionRequired {
                player: state.active,
                capture_field: pos,
            });
            state.phase = match state.phase {
                Phase::Namu(_) => Phase::Namu(Substate::AwaitKichwa {
                    capture_field: pos,
                    prior_dir: Some(dir),
                }),
                Phase::Mtaji(_) => Phase::Mtaji(Substate::AwaitKichwa {
                    capture_field: pos,
                    prior_dir: Some(dir),
                }),
            };
            return Ok(());
        }

        // Safari trigger (RULES.md §6.4).
        let is_own_func_nyumba = pos == state.sides[active].nyumba_col
            && state.sides[active].nyumba_owned
            && landed >= NYUMBA_FUNCTIONAL_THRESHOLD;
        if is_own_func_nyumba {
            events.push(MoveEvent::SafariTriggered {
                player: state.active,
            });
            state.phase = match state.phase {
                Phase::Namu(_) => Phase::Namu(Substate::AwaitSafari { sow_dir: dir }),
                Phase::Mtaji(_) => Phase::Mtaji(Substate::AwaitSafari { sow_dir: dir }),
            };
            return Ok(());
        }

        // Endelea: pickup and continue.
        hand = landed;
        state.sides[active].vichwa[pos as usize] = 0;
        events.push(MoveEvent::Pickup {
            player: state.active,
            pit: pos,
            count: hand,
        });
        maybe_destroy_own_nyumba(state, active, pos, events);
    }
}

// ---------- Turn / phase administration ----------

fn end_turn(state: &mut BoardState, events: &mut Vec<MoveEvent>) {
    state.active = 1 - state.active;
    state.ply += 1;

    // TODO RULES.md §11: decrement kutakatia counter and clear if 0.

    // Phase shift namu → mtaji (Kiswahili) when both ghalas are empty
    // (RULES.md §3.3).
    if matches!(state.phase, Phase::Namu(_))
        && state.variant == Variant::Kiswahili
        && state.sides[0].ghala == 0
        && state.sides[1].ghala == 0
    {
        state.phase = Phase::Mtaji(Substate::AwaitMove);
        events.push(MoveEvent::PhaseShift);
        return;
    }

    state.phase = match state.phase {
        Phase::Namu(_) => Phase::Namu(Substate::AwaitMove),
        Phase::Mtaji(_) => Phase::Mtaji(Substate::AwaitMove),
    };

    // Win/loss detection (RULES.md §9). Only applied at the start of the
    // new active player's turn — substate transitions don't end the game.
    if let Some(winner) = check_terminal(state) {
        state.winner = Some(winner);
        events.push(MoveEvent::GameOver { winner });
    }
}

/// RULES.md §9: opponent's mbele empty after a capture is hamna; active's
/// mbele empty at the start of their turn is hamna/mkononi; active without
/// any legal move is a stalemate-loss (also §9.3).
fn check_terminal(state: &BoardState) -> Option<u8> {
    if state.sides[0].mbele_total() == 0 {
        return Some(1);
    }
    if state.sides[1].mbele_total() == 0 {
        return Some(0);
    }
    if legal_moves(state).is_empty() {
        return Some(state.opponent(state.active));
    }
    None
}

/// Set nyumba_owned to false if active player's own sow emptied it
/// (RULES.md §8.3 trigger (a)/(c)).
fn maybe_destroy_own_nyumba(
    state: &mut BoardState,
    side_idx: usize,
    pit: u8,
    events: &mut Vec<MoveEvent>,
) {
    if pit == state.sides[side_idx].nyumba_col
        && state.sides[side_idx].nyumba_owned
        && state.sides[side_idx].vichwa[pit as usize] == 0
    {
        state.sides[side_idx].nyumba_owned = false;
        events.push(MoveEvent::NyumbaDestroyed {
            player: side_idx as u8,
        });
    }
}

/// Set opponent's nyumba_owned to false if a capture emptied it
/// (RULES.md §8.3 trigger (b)).
fn maybe_destroy_opp_nyumba(
    state: &mut BoardState,
    opp_idx: usize,
    pit: u8,
    events: &mut Vec<MoveEvent>,
) {
    if pit == state.sides[opp_idx].nyumba_col
        && state.sides[opp_idx].nyumba_owned
        && state.sides[opp_idx].vichwa[pit as usize] == 0
    {
        state.sides[opp_idx].nyumba_owned = false;
        events.push(MoveEvent::NyumbaDestroyed {
            player: opp_idx as u8,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BoardState, Side, NYUMBA_COL, NYUMBA_COL_NORTH, PITS_PER_SIDE};

    fn empty_kiswahili_state() -> BoardState {
        let south = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 32,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 32,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL_NORTH as u8,
        };
        BoardState {
            sides: [south, north],
            phase: Phase::Namu(Substate::AwaitMove),
            active: 0,
            ply: 0,
            variant: Variant::Kiswahili,
            kutakatia: None,
            winner: None,
        }
    }

    fn empty_mtaji_state() -> BoardState {
        let south = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL_NORTH as u8,
        };
        BoardState {
            sides: [south, north],
            phase: Phase::Mtaji(Substate::AwaitMove),
            active: 0,
            ply: 0,
            variant: Variant::Kiswahili,
            kutakatia: None,
            winner: None,
        }
    }

    // ---------- landing helper ----------

    #[test]
    fn landing_one_step_cw() {
        assert_eq!(landing(0, Direction::Cw, 1), 1);
        assert_eq!(landing(7, Direction::Cw, 1), 8);
        assert_eq!(landing(15, Direction::Cw, 1), 0);
    }

    #[test]
    fn landing_wraps() {
        // 5 kete from pit 14 cw: 15, 0, 1, 2, 3
        assert_eq!(landing(14, Direction::Cw, 5), 3);
        // 5 kete from pit 2 ccw: 1, 0, 15, 14, 13
        assert_eq!(landing(2, Direction::Ccw, 5), 13);
    }

    // ---------- Namu legal moves ----------

    #[test]
    fn namu_initial_position_no_captures_falls_to_takata() {
        let state = BoardState::new(Variant::Kiswahili);
        let moves = legal_moves(&state);
        // Initial Kiswahili: own mbele has cols 4,5,6 filled but opp.mbele
        // mirror has cols 1,2,3 filled (from opp perspective those are their
        // own initial positions). So columns 4,5,6 vs opponent 4,5,6 — opp's
        // 4,5,6 are all 0. No captures.
        for m in &moves {
            assert!(matches!(m, Move::Namu { .. }), "got {:?}", m);
        }
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 5, .. })));
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 6, .. })));
        assert!(!moves.iter().any(|m| matches!(m, Move::Namu { col: 4, .. })));
    }

    #[test]
    fn namu_capture_when_aligned() {
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[3] = 1; // own mbele
        state.sides[1].vichwa[3] = 1; // opp mbele same col → capture available
        state.sides[0].vichwa[5] = 4; // some non-capture filler
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            Move::Namu {
                col: 3,
                dir: Direction::Cw,
            }
        );
    }

    #[test]
    fn namu_takata_disabled_nyumba_no_restriction() {
        let mut state = empty_kiswahili_state();
        // Owned but <6 → Disabled.
        state.sides[0].vichwa[NYUMBA_COL] = 3;
        state.sides[0].vichwa[1] = 1;
        // No opp kete → no captures.
        let moves = legal_moves(&state);
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 1, .. })));
        assert!(moves
            .iter()
            .any(|m| matches!(m, Move::Namu { col: c, .. } if *c == NYUMBA_COL as u8)));
    }

    #[test]
    fn namu_takata_destroyed_nyumba_prefers_two() {
        let mut state = empty_kiswahili_state();
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[1] = 2;
        state.sides[0].vichwa[3] = 1;
        let moves = legal_moves(&state);
        // Two directions per col, only col 1 (>=2) allowed.
        assert_eq!(moves.len(), 2);
        assert!(moves.iter().all(|m| matches!(m, Move::Namu { col: 1, .. })));
    }

    #[test]
    fn namu_takata_destroyed_nyumba_falls_back_to_one() {
        let mut state = empty_kiswahili_state();
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[2] = 1;
        state.sides[0].vichwa[5] = 1;
        let moves = legal_moves(&state);
        // 2 cols × 2 dirs = 4.
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn namu_takata_functional_falls_back_to_nyumba_when_no_other() {
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[NYUMBA_COL] = 6; // functional, only filled mbele
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
        assert!(moves
            .iter()
            .all(|m| matches!(m, Move::Namu { col, .. } if *col == NYUMBA_COL as u8)));
    }

    // ---------- Mtaji legal moves ----------

    #[test]
    fn mtaji_capture_basic() {
        let mut state = empty_mtaji_state();
        // Own pit 5 has 3 kete; sowing Cw → pits 6, 7, 0 (wraps). Land=0.
        // Wait: 3 from pit 5 Cw lands at (5+3)%16 = 8 (nyuma). Not mbele,
        // so no capture there. Use Ccw: (5-3) mod 16 = 2 (mbele).
        state.sides[0].vichwa[5] = 3;
        state.sides[0].vichwa[2] = 1; // landing has >=1 pre-drop
        state.sides[1].vichwa[2] = 1; // opponent same col has stones
        let moves = legal_moves(&state);
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 5,
                dir: Direction::Ccw,
            }]
        );
    }

    #[test]
    fn mtaji_no_capture_with_16_seeds() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 16; // exceeds 15 → no capture eligibility
        state.sides[0].vichwa[3] = 2; // alternative source
        state.sides[1].vichwa[3] = 1;
        let moves = legal_moves(&state);
        // From pit 3 Cw lands at 5 (own=0, no cap). Ccw lands at 1 (own=0, no cap).
        // From pit 0 with 16 kete → ineligible for capture.
        // So no captures → fall through to takata.
        assert!(!moves.is_empty());
        assert!(moves.iter().all(|m| matches!(m, Move::Mtaji { .. })));
    }

    #[test]
    fn mtaji_takata_prefers_mbele() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[2] = 3; // mbele >=2
        state.sides[0].vichwa[10] = 5; // nyuma >=2
        let moves = legal_moves(&state);
        // Must come from mbele (pit 2) only.
        for m in &moves {
            match m {
                Move::Mtaji { pit, .. } => assert!((*pit as usize) < MBELE_LEN),
                _ => panic!("unexpected move {:?}", m),
            }
        }
    }

    #[test]
    fn mtaji_takata_falls_back_to_nyuma() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[10] = 4;
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
        for m in &moves {
            assert!(matches!(m, Move::Mtaji { pit: 10, .. }));
        }
    }

    #[test]
    fn mtaji_no_singleton_source() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[3] = 1; // singleton — must not appear as source
        let moves = legal_moves(&state);
        assert!(moves.is_empty());
    }

    #[test]
    fn mtaji_no_suicide_kichwa_left() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 3; // only filled mbele = left kichwa
        let moves = legal_moves(&state);
        // Only Cw (toward center) allowed; Ccw would step into nyuma.
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 0,
                dir: Direction::Cw,
            }]
        );
    }

    #[test]
    fn mtaji_no_suicide_kichwa_right() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[7] = 3;
        let moves = legal_moves(&state);
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 7,
                dir: Direction::Ccw,
            }]
        );
    }

    #[test]
    fn mtaji_kichwa_with_other_filled_allows_both_dirs() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 3;
        state.sides[0].vichwa[3] = 2; // additional filled mbele → no-suicide doesn't apply
        let moves = legal_moves(&state);
        // Pit 0 Cw, Pit 0 Ccw, Pit 3 Cw, Pit 3 Ccw → 4 moves.
        assert_eq!(moves.len(), 4);
    }

    // ---------- Kujifunza ----------

    #[test]
    fn kujifunza_initial_state_has_legal_moves() {
        let state = BoardState::new(Variant::Kujifunza);
        let moves = legal_moves(&state);
        // All 16 pits have 2 kete; each opponent col has 2 → all mtaji
        // captures from mbele (pits 0..7). From mbele, both directions: do
        // any land in own mbele with opp non-empty?
        // From pit i with 2 kete Cw lands at i+2; Ccw lands at i-2 mod 16.
        // For pits 0..5 Cw → 2..7 (mbele) ✓ capture eligible.
        // For pit 6 Cw → 8 (nyuma) — no.
        // For pit 7 Cw → 9 (nyuma) — no.
        // For pit 0 Ccw → 14 (nyuma) — no.
        // For pit 1 Ccw → 15 (nyuma) — no.
        // For pits 2..7 Ccw → 0..5 (mbele) ✓.
        // From nyuma pits 8..15 with 2 Cw/Ccw — many land in mbele too.
        // Just assert non-empty and all captures.
        assert!(!moves.is_empty());
        for m in &moves {
            assert!(matches!(m, Move::Mtaji { .. }));
        }
    }

    // ---------- Substates ----------

    #[test]
    fn await_kichwa_left_kimbi() {
        let moves = kichwa_legal_moves(0, None);
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Left)]);
        let moves = kichwa_legal_moves(1, Some(Direction::Cw));
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Left)]);
    }

    #[test]
    fn await_kichwa_right_kimbi() {
        let moves = kichwa_legal_moves(7, None);
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Right)]);
        let moves = kichwa_legal_moves(6, Some(Direction::Ccw));
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Right)]);
    }

    #[test]
    fn await_kichwa_middle_with_dir_is_deterministic() {
        assert_eq!(
            kichwa_legal_moves(4, Some(Direction::Cw)),
            vec![Move::Kichwa(KichwaSide::Left)]
        );
        assert_eq!(
            kichwa_legal_moves(4, Some(Direction::Ccw)),
            vec![Move::Kichwa(KichwaSide::Right)]
        );
    }

    #[test]
    fn await_kichwa_middle_no_dir_offers_both() {
        let moves = kichwa_legal_moves(4, None);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Kichwa(KichwaSide::Left)));
        assert!(moves.contains(&Move::Kichwa(KichwaSide::Right)));
    }

    // =================================================================
    // apply() tests
    // =================================================================

    #[test]
    fn apply_namu_takata_basic_sow() {
        let mut state = empty_kiswahili_state();
        // South: only col 2 has 1 kete in mbele; nyumba destroyed → branch 1
        // (>=1 fallback).
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[2] = 1;
        // After namu place: vichwa[2]=2. Pickup all → sow Cw: drop at 3, 4.
        let mv = Move::Namu {
            col: 2,
            dir: Direction::Cw,
        };
        let (after, _events) = apply(&state, mv).unwrap();
        assert_eq!(after.sides[0].vichwa[2], 0);
        assert_eq!(after.sides[0].vichwa[3], 1);
        assert_eq!(after.sides[0].vichwa[4], 1);
        assert_eq!(after.sides[0].ghala, 31);
        // End-turn flipped active.
        assert_eq!(after.active, 1);
        assert!(matches!(after.phase, Phase::Namu(Substate::AwaitMove)));
    }

    #[test]
    fn apply_namu_kula_awaits_kichwa() {
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[3] = 1;
        state.sides[1].vichwa[3] = 1;
        let mv = Move::Namu {
            col: 3,
            dir: Direction::Cw,
        };
        let (after, _) = apply(&state, mv).unwrap();
        // Placement happened.
        assert_eq!(after.sides[0].vichwa[3], 2);
        assert_eq!(after.sides[0].ghala, 31);
        // Opponent untouched.
        assert_eq!(after.sides[1].vichwa[3], 1);
        // Awaiting kichwa selection; active player still 0.
        assert_eq!(after.active, 0);
        assert!(matches!(
            after.phase,
            Phase::Namu(Substate::AwaitKichwa {
                capture_field: 3,
                prior_dir: None,
            })
        ));
    }

    #[test]
    fn apply_kichwa_executes_capture_and_sow() {
        // Reproduce a namu-kula then kichwa: own col 3 has 1 (post-place 2),
        // opp col 3 has 4 → capture 4. Kichwa Left (cw from pit 0).
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[3] = 2; // post-place state simulated
        state.sides[1].vichwa[3] = 4;
        state.phase = Phase::Namu(Substate::AwaitKichwa {
            capture_field: 3,
            prior_dir: None,
        });
        let mv = Move::Kichwa(KichwaSide::Left);
        let (after, _) = apply(&state, mv).unwrap();
        // Opponent's col 3 emptied.
        assert_eq!(after.sides[1].vichwa[3], 0);
        // 4 kete sown clockwise from kichwa pit 0: drops at 0, 1, 2, 3.
        // Pre-sow our pits: 0=0,1=0,2=0,3=2. After 4 drops: 0=1,1=1,2=1,3=3.
        // Then post-final-drop at pit 3, count=3 → endelea! Pickup 3, sow
        // from 3 cw: drops at 4,5,6. Pre 4=0,5=0,6=0 → after 4=1,5=1,6=1.
        // Final landing at pit 6, count=1 → empty landing → end.
        assert_eq!(after.sides[0].vichwa[0], 1);
        assert_eq!(after.sides[0].vichwa[1], 1);
        assert_eq!(after.sides[0].vichwa[2], 1);
        assert_eq!(after.sides[0].vichwa[3], 0);
        assert_eq!(after.sides[0].vichwa[4], 1);
        assert_eq!(after.sides[0].vichwa[5], 1);
        assert_eq!(after.sides[0].vichwa[6], 1);
        // Turn flipped.
        assert_eq!(after.active, 1);
    }

    #[test]
    fn apply_mtaji_takata_sow() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[2] = 3;
        // Sow cw from 2: drops at 3,4,5. No capture (opp empty), no endelea.
        let (after, _) = apply(
            &state,
            Move::Mtaji {
                pit: 2,
                dir: Direction::Cw,
            },
        )
        .unwrap();
        assert_eq!(after.sides[0].vichwa[2], 0);
        assert_eq!(after.sides[0].vichwa[3], 1);
        assert_eq!(after.sides[0].vichwa[4], 1);
        assert_eq!(after.sides[0].vichwa[5], 1);
        assert_eq!(after.active, 1);
    }

    #[test]
    fn apply_mtaji_capture_awaits_kichwa() {
        let mut state = empty_mtaji_state();
        // Pit 5 has 3 kete; sow ccw: 4,3,2. Land=2, own=1, opp=1 → capture.
        state.sides[0].vichwa[5] = 3;
        state.sides[0].vichwa[2] = 1;
        state.sides[1].vichwa[2] = 1;
        let (after, _) = apply(
            &state,
            Move::Mtaji {
                pit: 5,
                dir: Direction::Ccw,
            },
        )
        .unwrap();
        assert!(matches!(
            after.phase,
            Phase::Mtaji(Substate::AwaitKichwa {
                capture_field: 2,
                prior_dir: Some(Direction::Ccw),
            })
        ));
        // Sow happened: own pit 5=0, pits 4,3,2 each +1.
        assert_eq!(after.sides[0].vichwa[5], 0);
        assert_eq!(after.sides[0].vichwa[4], 1);
        assert_eq!(after.sides[0].vichwa[3], 1);
        assert_eq!(after.sides[0].vichwa[2], 2);
        // Opponent untouched until kichwa applied.
        assert_eq!(after.sides[1].vichwa[2], 1);
        assert_eq!(after.active, 0);
    }

    #[test]
    fn apply_namu_tax_from_functional_nyumba() {
        let mut state = empty_kiswahili_state();
        // South nyumba (idx 4) has 6 kete (functional). All other mbele empty
        // → §8.5 branch 2 fallback: takata from nyumba is the only legal move.
        state.sides[0].vichwa[4] = 6;
        // Apply namu place at nyumba_col 4 with dir Cw. Post-place=7.
        // Tax: take 2 (not 7). Sow cw 2 from idx 4 → drops at 5, 6.
        let (after, _) = apply(
            &state,
            Move::Namu {
                col: 4,
                dir: Direction::Cw,
            },
        )
        .unwrap();
        // Nyumba had 6, +1 placement = 7, -2 tax = 5.
        assert_eq!(after.sides[0].vichwa[4], 5);
        assert_eq!(after.sides[0].vichwa[5], 1);
        assert_eq!(after.sides[0].vichwa[6], 1);
        // Nyumba still owned (tax never destroys).
        assert!(after.sides[0].nyumba_owned);
    }

    #[test]
    fn apply_phase_shift_when_both_ghalas_empty() {
        let mut state = empty_kiswahili_state();
        state.sides[0].ghala = 1;
        state.sides[1].ghala = 0;
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[2] = 1;
        // Kunamua: place from ghala (1→0). Both ghalas now 0 → phase shift.
        let (after, events) = apply(
            &state,
            Move::Namu {
                col: 2,
                dir: Direction::Cw,
            },
        )
        .unwrap();
        assert_eq!(after.sides[0].ghala, 0);
        assert!(matches!(after.phase, Phase::Mtaji(Substate::AwaitMove)));
        assert!(events.iter().any(|e| matches!(e, MoveEvent::PhaseShift)));
    }

    #[test]
    fn apply_mtaji_takata_destroys_own_nyumba() {
        let mut state = empty_mtaji_state();
        // Set up an owned nyumba at South.nyumba_col=4, with 2 kete.
        state.sides[0].nyumba_owned = true;
        state.sides[0].vichwa[NYUMBA_COL] = 2;
        // Source = nyumba; takata-sow empties it → destroyed.
        let (after, events) = apply(
            &state,
            Move::Mtaji {
                pit: NYUMBA_COL as u8,
                dir: Direction::Cw,
            },
        )
        .unwrap();
        assert!(!after.sides[0].nyumba_owned);
        assert!(events
            .iter()
            .any(|e| matches!(e, MoveEvent::NyumbaDestroyed { .. })));
    }

    #[test]
    fn apply_safari_stop_ends_turn() {
        let mut state = empty_mtaji_state();
        state.sides[0].nyumba_owned = true;
        state.sides[0].vichwa[NYUMBA_COL] = 6;
        state.phase = Phase::Mtaji(Substate::AwaitSafari {
            sow_dir: Direction::Cw,
        });
        let (after, _) = apply(&state, Move::Safari { go: false }).unwrap();
        assert_eq!(after.active, 1);
        assert!(after.sides[0].nyumba_owned);
        assert_eq!(after.sides[0].vichwa[NYUMBA_COL], 6);
        assert!(matches!(after.phase, Phase::Mtaji(Substate::AwaitMove)));
    }

    #[test]
    fn apply_safari_go_destroys_nyumba_and_continues() {
        let mut state = empty_mtaji_state();
        state.sides[0].nyumba_owned = true;
        state.sides[0].vichwa[NYUMBA_COL] = 6;
        state.phase = Phase::Mtaji(Substate::AwaitSafari {
            sow_dir: Direction::Cw,
        });
        let (after, events) = apply(&state, Move::Safari { go: true }).unwrap();
        assert!(!after.sides[0].nyumba_owned);
        assert_eq!(after.sides[0].vichwa[NYUMBA_COL], 0);
        // 6 kete sown cw from idx 4: drops at 5,6,7,8,9,10.
        assert_eq!(after.sides[0].vichwa[5], 1);
        assert_eq!(after.sides[0].vichwa[10], 1);
        // Nyumba destroyed event emitted.
        assert!(events
            .iter()
            .any(|e| matches!(e, MoveEvent::NyumbaDestroyed { .. })));
    }

    #[test]
    fn apply_total_kete_invariant() {
        // Run a few moves on a fresh Kiswahili board; check 64-kete invariant.
        let mut state = BoardState::new(Variant::Kiswahili);
        for _ in 0..10 {
            let moves = legal_moves(&state);
            if moves.is_empty() {
                break;
            }
            let m = moves[0];
            // Skip if it triggers a substate (we'd need to follow up); just
            // pick a move type that always completes in one apply.
            match m {
                Move::Namu { .. } | Move::Mtaji { .. } => {
                    if let Ok((next, _)) = apply(&state, m) {
                        // Substates (kichwa/safari) leave active player same;
                        // we only count atomic transitions here.
                        if !matches!(
                            next.phase,
                            Phase::Namu(Substate::AwaitKichwa { .. })
                                | Phase::Mtaji(Substate::AwaitKichwa { .. })
                                | Phase::Namu(Substate::AwaitSafari { .. })
                                | Phase::Mtaji(Substate::AwaitSafari { .. })
                        ) {
                            assert_eq!(next.total_kete(), 64);
                            state = next;
                            continue;
                        }
                    }
                    break;
                }
                _ => break,
            }
        }
    }

    #[test]
    fn apply_terminal_hamna_when_opp_mbele_emptied() {
        // South captures opp's only mbele kete via namu-kula → opp mbele 0 → S wins.
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[3] = 1; // own pre-place
        state.sides[1].vichwa[3] = 1; // opp's only mbele kete
        // Trigger AwaitKichwa
        let (s1, _) = apply(
            &state,
            Move::Namu {
                col: 3,
                dir: Direction::Cw,
            },
        )
        .unwrap();
        assert!(s1.winner.is_none());
        // Pick kichwa Left (deterministic since cf=3 is in middle but
        // prior_dir=None → both legal; either works).
        let (s2, events) = apply(&s1, Move::Kichwa(KichwaSide::Left)).unwrap();
        assert_eq!(s2.winner, Some(0));
        assert!(events
            .iter()
            .any(|e| matches!(e, MoveEvent::GameOver { winner: 0 })));
    }

    #[test]
    fn apply_rejects_after_game_over() {
        let mut state = empty_kiswahili_state();
        state.winner = Some(0);
        let res = apply(
            &state,
            Move::Namu {
                col: 0,
                dir: Direction::Cw,
            },
        );
        assert!(res.is_err());
    }

    #[test]
    fn apply_terminal_stalemate_no_legal_moves() {
        // Mtaji state where active has no vichwa with >=2 anywhere → loss.
        let mut state = empty_mtaji_state();
        // Opponent has at least 1 mbele kete to avoid hamna firing first.
        state.sides[1].vichwa[3] = 5;
        // Active player after end_turn will be 1 after we apply something.
        // Easier: set state directly with active=0 having no >=2 anywhere.
        // mbele_total non-zero (avoid hamna): vichwa[0]=1 singleton.
        state.sides[0].vichwa[0] = 1;
        // Run a fake "move complete" by directly invoking end_turn... not
        // accessible. Instead: set up a position and check legal_moves +
        // terminal manually.
        assert!(legal_moves(&state).is_empty());
        assert_eq!(check_terminal(&state), Some(1));
    }

    #[test]
    fn apply_rejects_move_in_wrong_substate() {
        let state = BoardState::new(Variant::Kiswahili);
        // Phase = Namu(AwaitMove). A Move::Mtaji is illegal here.
        let err = apply(
            &state,
            Move::Mtaji {
                pit: 5,
                dir: Direction::Cw,
            },
        );
        assert!(err.is_err());
    }

    #[test]
    fn await_safari_offers_both() {
        let mut state = empty_mtaji_state();
        state.phase = Phase::Mtaji(Substate::AwaitSafari {
            sow_dir: Direction::Cw,
        });
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Safari { go: true }));
        assert!(moves.contains(&Move::Safari { go: false }));
    }
}
