//! Binary shard container for NNUE training data.
//!
//! See `docs/shard_format.md`. All multi-byte fields are little-endian.
//! Layout:
//!
//! ```text
//! [Header: 32 bytes][Record 0][Record 1]...[Record N-1]
//! ```
//!
//! Each record is `record_stride = FEATURE_LEN + 2` bytes (features + i16 label).
//! Readers must validate the magic + version before trusting the rest of the
//! file — silently-wrong reads are the failure mode this format is designed
//! to prevent.

use std::io::{Read, Write};

use crate::features::{encode_features, FEATURE_LEN};

pub const MAGIC: &[u8; 8] = b"BAOSHRD\0";
pub const SHARD_VERSION: u16 = 1;
pub const HEADER_LEN: usize = 32;
pub const LABEL_BYTES: u16 = 2;
pub const RECORD_STRIDE: usize = FEATURE_LEN + LABEL_BYTES as usize;
pub const LABEL_DTYPE_I16: u8 = 1;
pub const LABEL_UNIT_CENTIKETE: u8 = 1;

#[derive(Debug, Clone)]
pub struct ShardHeader {
    pub version: u16,
    pub feature_len: u16,
    pub label_bytes: u16,
    pub record_stride: u16,
    pub n_records: u32,
    pub label_dtype: u8,
    pub label_unit: u8,
    pub engine_major: u8,
    pub engine_minor: u8,
}

impl ShardHeader {
    pub fn new(n_records: u32) -> Self {
        let (major, minor) = parse_engine_version();
        Self {
            version: SHARD_VERSION,
            feature_len: FEATURE_LEN as u16,
            label_bytes: LABEL_BYTES,
            record_stride: RECORD_STRIDE as u16,
            n_records,
            label_dtype: LABEL_DTYPE_I16,
            label_unit: LABEL_UNIT_CENTIKETE,
            engine_major: major,
            engine_minor: minor,
        }
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut buf = [0u8; HEADER_LEN];
        buf[0..8].copy_from_slice(MAGIC);
        buf[8..10].copy_from_slice(&self.version.to_le_bytes());
        buf[10..12].copy_from_slice(&self.feature_len.to_le_bytes());
        buf[12..14].copy_from_slice(&self.label_bytes.to_le_bytes());
        buf[14..16].copy_from_slice(&self.record_stride.to_le_bytes());
        buf[16..20].copy_from_slice(&self.n_records.to_le_bytes());
        buf[20] = self.label_dtype;
        buf[21] = self.label_unit;
        buf[22] = self.engine_major;
        buf[23] = self.engine_minor;
        // buf[24..32] already zero (reserved)
        w.write_all(&buf)
    }

    pub fn read_from<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; HEADER_LEN];
        r.read_exact(&mut buf)?;
        if &buf[0..8] != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad shard magic",
            ));
        }
        let version = u16::from_le_bytes([buf[8], buf[9]]);
        if version > SHARD_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported shard version {} (max {})", version, SHARD_VERSION),
            ));
        }
        let feature_len = u16::from_le_bytes([buf[10], buf[11]]);
        let label_bytes = u16::from_le_bytes([buf[12], buf[13]]);
        let record_stride = u16::from_le_bytes([buf[14], buf[15]]);
        let n_records = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        if feature_len as usize != FEATURE_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("feature_len mismatch: shard has {}, engine expects {}", feature_len, FEATURE_LEN),
            ));
        }
        if record_stride as usize != RECORD_STRIDE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("record_stride mismatch: shard has {}, engine expects {}", record_stride, RECORD_STRIDE),
            ));
        }
        Ok(Self {
            version,
            feature_len,
            label_bytes,
            record_stride,
            n_records,
            label_dtype: buf[20],
            label_unit: buf[21],
            engine_major: buf[22],
            engine_minor: buf[23],
        })
    }
}

fn parse_engine_version() -> (u8, u8) {
    let v = env!("CARGO_PKG_VERSION");
    let mut parts = v.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor)
}

/// Sink that buffers records and emits them with a correct header on close.
pub struct ShardWriter<W: Write> {
    inner: W,
    n_records: u32,
}

impl<W: Write> ShardWriter<W> {
    /// `expected_n` is written to the header up-front. The actual record
    /// count must match by the time `finish` is called.
    pub fn new(mut inner: W, expected_n: u32) -> std::io::Result<Self> {
        ShardHeader::new(expected_n).write_to(&mut inner)?;
        Ok(Self {
            inner,
            n_records: 0,
        })
    }

    pub fn write_record(&mut self, features: &[u8; FEATURE_LEN], label: i16) -> std::io::Result<()> {
        self.inner.write_all(features)?;
        self.inner.write_all(&label.to_le_bytes())?;
        self.n_records += 1;
        Ok(())
    }

    pub fn write_from_state(
        &mut self,
        state: &crate::board::BoardState,
        label: i16,
    ) -> std::io::Result<()> {
        let f = encode_features(state);
        self.write_record(&f, label)
    }

    /// Returns the number of records written.
    pub fn finish(self) -> std::io::Result<u32> {
        Ok(self.n_records)
    }
}

/// Read every record from a shard. Returns `(header, records)` where each
/// record is `(features, label)`. For training we'll usually memory-map and
/// avoid the per-record allocation, but this loader is the canonical
/// roundtrip-test reference and the Python loader's behaviour mirror.
pub fn read_shard<R: Read>(
    r: &mut R,
) -> std::io::Result<(ShardHeader, Vec<([u8; FEATURE_LEN], i16)>)> {
    let header = ShardHeader::read_from(r)?;
    let mut out = Vec::with_capacity(header.n_records as usize);
    let mut buf = [0u8; RECORD_STRIDE];
    for _ in 0..header.n_records {
        r.read_exact(&mut buf)?;
        let mut feat = [0u8; FEATURE_LEN];
        feat.copy_from_slice(&buf[..FEATURE_LEN]);
        let label = i16::from_le_bytes([buf[FEATURE_LEN], buf[FEATURE_LEN + 1]]);
        out.push((feat, label));
    }
    Ok((header, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardState;
    use crate::variant::Variant;
    use std::io::Cursor;

    #[test]
    fn header_roundtrip() {
        let h = ShardHeader::new(123);
        let mut buf = Vec::new();
        h.write_to(&mut buf).unwrap();
        assert_eq!(buf.len(), HEADER_LEN);
        let h2 = ShardHeader::read_from(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(h2.version, SHARD_VERSION);
        assert_eq!(h2.feature_len as usize, FEATURE_LEN);
        assert_eq!(h2.record_stride as usize, RECORD_STRIDE);
        assert_eq!(h2.n_records, 123);
        assert_eq!(h2.label_dtype, LABEL_DTYPE_I16);
    }

    #[test]
    fn bad_magic_rejected() {
        let mut buf = vec![0u8; HEADER_LEN];
        buf[0..8].copy_from_slice(b"NOTBAOSH");
        let err = ShardHeader::read_from(&mut Cursor::new(&buf)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn full_shard_roundtrip() {
        let s = BoardState::new(Variant::Kiswahili);
        let mut buf = Vec::new();
        {
            let mut w = ShardWriter::new(&mut buf, 3).unwrap();
            w.write_from_state(&s, 100).unwrap();
            w.write_from_state(&s, -50).unwrap();
            w.write_from_state(&s, 0).unwrap();
            let n = w.finish().unwrap();
            assert_eq!(n, 3);
        }
        let (h, recs) = read_shard(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(h.n_records, 3);
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].1, 100);
        assert_eq!(recs[1].1, -50);
        assert_eq!(recs[2].1, 0);
        let expected = encode_features(&s);
        for (feat, _) in &recs {
            assert_eq!(feat, &expected);
        }
    }

    #[test]
    fn unsupported_version_rejected() {
        let mut buf = vec![0u8; HEADER_LEN];
        buf[0..8].copy_from_slice(MAGIC);
        buf[8..10].copy_from_slice(&999u16.to_le_bytes());
        buf[10..12].copy_from_slice(&(FEATURE_LEN as u16).to_le_bytes());
        let err = ShardHeader::read_from(&mut Cursor::new(&buf)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
