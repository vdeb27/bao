use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Variant {
    Kiswahili,
    Kujifunza,
}

impl Variant {
    pub fn has_namu(self) -> bool {
        matches!(self, Variant::Kiswahili)
    }

    pub fn has_nyumba(self) -> bool {
        matches!(self, Variant::Kiswahili)
    }

    pub fn has_kutakatia(self) -> bool {
        matches!(self, Variant::Kiswahili)
    }

    pub fn has_tax(self) -> bool {
        matches!(self, Variant::Kiswahili)
    }
}
