//! NNUE evaluation: sparse feature transformer + (later) integer forward
//! pass + loader. See `docs/nnue_format.md`.

pub mod loader;
pub mod transformer;

pub use loader::{load_from_bytes, LoadError, NnueModel, OUTPUT_SCALE};
pub use transformer::{
    indices, indices_from_features, N_FEATURES, MAX_ACTIVE,
    PIT_BUCKETS_BASE, NYUMBA_STATE_BASE, PHASE_SUBSTATE_BASE, KUTAKATIA_BASE,
};

use crate::board::BoardState;
use crate::eval::Evaluator;

/// NNUE-backed `Evaluator` impl. Wraps an `NnueModel` for use in search.
pub struct NnueEval {
    pub model: NnueModel,
}

impl NnueEval {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LoadError> {
        Ok(Self { model: load_from_bytes(bytes)? })
    }
}

impl Evaluator for NnueEval {
    fn eval(&self, state: &BoardState) -> i32 {
        self.model.evaluate(state)
    }
}
