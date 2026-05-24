//! Loader for the ``.nnue`` binary format. See ``docs/nnue_format.md``.
//!
//! Layout (little-endian throughout):
//! - 8 bytes magic ``BAONNUE\0``
//! - u16 version (= 1)
//! - u16 n_features (= 280)
//! - [u16; 4] hidden_sizes (= [512, 32, 32, 1])
//! - f32 quant_scale (= OUTPUT_SCALE)
//! - i16 [n_features × accumulator_dim] L0 weights, row-major
//! - i16 [accumulator_dim] L0 bias
//! - i8  [accumulator_dim × hidden_dim] L1 weights (in, out) row-major
//! - i32 [hidden_dim] L1 bias
//! - i8  [hidden_dim × hidden_dim] L2 weights (in, out)
//! - i32 [hidden_dim] L2 bias
//! - i8  [hidden_dim × 1] L3 weights (in, out)
//! - i32 [1] L3 bias

use std::io::{Cursor, Read};

use super::transformer::N_FEATURES;

pub const MAGIC: &[u8; 8] = b"BAONNUE\0";
pub const VERSION: u16 = 2;
pub const ACCUMULATOR_DIM: usize = 512;
pub const HIDDEN_DIM: usize = 32;
pub const WEIGHT_SCALE_L0: i32 = 64;
pub const WEIGHT_SCALE_HIDDEN: i32 = 64;
pub const ACTIVATION_CLIP: i32 = 127;
pub const OUTPUT_SCALE: i32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad magic: {0:?}")]
    BadMagic([u8; 8]),
    #[error("unsupported version {0} (max {VERSION})")]
    BadVersion(u16),
    #[error("n_features mismatch: file={file}, expected={expected}")]
    NFeaturesMismatch { file: u16, expected: u16 },
    #[error("hidden_sizes mismatch: file={file:?}, expected={expected:?}")]
    HiddenSizesMismatch { file: [u16; 4], expected: [u16; 4] },
}

pub struct NnueModel {
    pub n_features: usize,
    pub l0_weight: Vec<i16>, // (n_features × accumulator_dim)
    pub l0_bias: Vec<i16>,   // (accumulator_dim)
    pub l1_weight: Vec<i16>, // (accumulator_dim × hidden_dim)
    pub l1_bias: Vec<i32>,
    pub l2_weight: Vec<i16>,
    pub l2_bias: Vec<i32>,
    pub l3_weight: Vec<i16>,
    pub l3_bias: Vec<i32>,
}

fn read_u16<R: Read>(r: &mut R) -> std::io::Result<u16> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn read_f32<R: Read>(r: &mut R) -> std::io::Result<f32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(f32::from_le_bytes(b))
}

fn read_i16_vec<R: Read>(r: &mut R, n: usize) -> std::io::Result<Vec<i16>> {
    let mut buf = vec![0u8; n * 2];
    r.read_exact(&mut buf)?;
    Ok(buf.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect())
}

fn read_i32_vec<R: Read>(r: &mut R, n: usize) -> std::io::Result<Vec<i32>> {
    let mut buf = vec![0u8; n * 4];
    r.read_exact(&mut buf)?;
    Ok(buf
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

pub fn load_from_bytes(bytes: &[u8]) -> Result<NnueModel, LoadError> {
    let mut cur = Cursor::new(bytes);
    let mut magic = [0u8; 8];
    cur.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(LoadError::BadMagic(magic));
    }
    let version = read_u16(&mut cur)?;
    if version > VERSION {
        return Err(LoadError::BadVersion(version));
    }
    let n_features = read_u16(&mut cur)?;
    if n_features as usize != N_FEATURES {
        return Err(LoadError::NFeaturesMismatch {
            file: n_features,
            expected: N_FEATURES as u16,
        });
    }
    let mut hidden = [0u16; 4];
    for slot in &mut hidden {
        *slot = read_u16(&mut cur)?;
    }
    let expected = [ACCUMULATOR_DIM as u16, HIDDEN_DIM as u16, HIDDEN_DIM as u16, 1];
    if hidden != expected {
        return Err(LoadError::HiddenSizesMismatch { file: hidden, expected });
    }
    let _quant_scale = read_f32(&mut cur)?;

    let l0_weight = read_i16_vec(&mut cur, N_FEATURES * ACCUMULATOR_DIM)?;
    let l0_bias = read_i16_vec(&mut cur, ACCUMULATOR_DIM)?;
    let l1_weight = read_i16_vec(&mut cur, ACCUMULATOR_DIM * HIDDEN_DIM)?;
    let l1_bias = read_i32_vec(&mut cur, HIDDEN_DIM)?;
    let l2_weight = read_i16_vec(&mut cur, HIDDEN_DIM * HIDDEN_DIM)?;
    let l2_bias = read_i32_vec(&mut cur, HIDDEN_DIM)?;
    let l3_weight = read_i16_vec(&mut cur, HIDDEN_DIM)?;
    let l3_bias = read_i32_vec(&mut cur, 1)?;

    Ok(NnueModel {
        n_features: N_FEATURES,
        l0_weight,
        l0_bias,
        l1_weight,
        l1_bias,
        l2_weight,
        l2_bias,
        l3_weight,
        l3_bias,
    })
}

#[inline]
fn clipped_relu(x: i32) -> i32 {
    x.clamp(0, ACTIVATION_CLIP)
}

impl NnueModel {
    /// Integer forward pass on a set of sparse active indices. Returns the
    /// raw network output (in `OUTPUT_SCALE × centi-kete` units); divide by
    /// `OUTPUT_SCALE` for centi-kete.
    pub fn forward_raw(&self, indices: &[u16]) -> i32 {
        // Accumulator: sum selected rows of L0 + bias, kept at scale L0.
        let mut acc = [0i32; ACCUMULATOR_DIM];
        for j in 0..ACCUMULATOR_DIM {
            acc[j] = self.l0_bias[j] as i32;
        }
        for &idx in indices {
            let row_start = (idx as usize) * ACCUMULATOR_DIM;
            let row = &self.l0_weight[row_start..row_start + ACCUMULATOR_DIM];
            for (a, w) in acc.iter_mut().zip(row.iter()) {
                *a += *w as i32;
            }
        }

        // ClippedReLU on accumulator (rescale from L0-scale to activation-scale).
        let mut h1_in = [0i32; ACCUMULATOR_DIM];
        for (i, &a) in acc.iter().enumerate() {
            h1_in[i] = clipped_relu(a / WEIGHT_SCALE_L0);
        }

        // L1: h2 = clipped_relu( (h1 · W1 + bias) / W_HIDDEN )
        let mut h2 = [0i32; HIDDEN_DIM];
        for j in 0..HIDDEN_DIM {
            let mut sum = self.l1_bias[j];
            for i in 0..ACCUMULATOR_DIM {
                sum += h1_in[i] * (self.l1_weight[i * HIDDEN_DIM + j] as i32);
            }
            h2[j] = clipped_relu(sum / WEIGHT_SCALE_HIDDEN);
        }

        // L2
        let mut h3 = [0i32; HIDDEN_DIM];
        for j in 0..HIDDEN_DIM {
            let mut sum = self.l2_bias[j];
            for i in 0..HIDDEN_DIM {
                sum += h2[i] * (self.l2_weight[i * HIDDEN_DIM + j] as i32);
            }
            h3[j] = clipped_relu(sum / WEIGHT_SCALE_HIDDEN);
        }

        // L3 → scalar, no activation
        let mut out = self.l3_bias[0];
        for i in 0..HIDDEN_DIM {
            out += h3[i] * (self.l3_weight[i] as i32);
        }
        out / WEIGHT_SCALE_HIDDEN
    }

    /// Evaluate a position: returns centi-kete score from the active
    /// player's perspective.
    pub fn evaluate(&self, state: &crate::board::BoardState) -> i32 {
        let idx = super::transformer::indices(state);
        self.forward_raw(&idx) / OUTPUT_SCALE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_zero_model() -> Vec<u8> {
        // Hand-build a BAONNUE file with all weights = 0, all biases = 0.
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(MAGIC);
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&(N_FEATURES as u16).to_le_bytes());
        for &s in &[ACCUMULATOR_DIM as u16, HIDDEN_DIM as u16, HIDDEN_DIM as u16, 1u16] {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(OUTPUT_SCALE as f32).to_le_bytes());
        let l0_w = vec![0u8; N_FEATURES * ACCUMULATOR_DIM * 2];
        let l0_b = vec![0u8; ACCUMULATOR_DIM * 2];
        let l1_w = vec![0u8; ACCUMULATOR_DIM * HIDDEN_DIM * 2];
        let l1_b = vec![0u8; HIDDEN_DIM * 4];
        let l2_w = vec![0u8; HIDDEN_DIM * HIDDEN_DIM * 2];
        let l2_b = vec![0u8; HIDDEN_DIM * 4];
        let l3_w = vec![0u8; HIDDEN_DIM * 2];
        let l3_b = vec![0u8; 4];
        buf.extend(l0_w); buf.extend(l0_b);
        buf.extend(l1_w); buf.extend(l1_b);
        buf.extend(l2_w); buf.extend(l2_b);
        buf.extend(l3_w); buf.extend(l3_b);
        buf
    }

    #[test]
    fn load_zero_model_evaluates_to_zero() {
        let bytes = synth_zero_model();
        let model = load_from_bytes(&bytes).expect("load");
        let s = crate::board::BoardState::new(crate::variant::Variant::Kiswahili);
        assert_eq!(model.evaluate(&s), 0);
    }

    #[test]
    fn bad_magic_rejected() {
        let mut bytes = synth_zero_model();
        bytes[0] = b'X';
        assert!(matches!(load_from_bytes(&bytes), Err(LoadError::BadMagic(_))));
    }

    #[test]
    fn bad_version_rejected() {
        let mut bytes = synth_zero_model();
        bytes[8..10].copy_from_slice(&999u16.to_le_bytes());
        assert!(matches!(load_from_bytes(&bytes), Err(LoadError::BadVersion(_))));
    }

    #[test]
    fn forward_with_bias_only() {
        // L3 bias = 1600 (in combined scale: WEIGHT_SCALE_HIDDEN²), all else 0.
        // forward_raw = out / WEIGHT_SCALE_HIDDEN = 1600/64 = 25.
        // evaluate = forward_raw / OUTPUT_SCALE = 25/1 = 25.
        let mut bytes = synth_zero_model();
        let l3_bias_offset = bytes.len() - 4;
        bytes[l3_bias_offset..].copy_from_slice(&1600i32.to_le_bytes());
        let model = load_from_bytes(&bytes).expect("load");
        let s = crate::board::BoardState::new(crate::variant::Variant::Kiswahili);
        assert_eq!(model.forward_raw(&[]), 25);
        assert_eq!(model.evaluate(&s), 25);
    }
}
