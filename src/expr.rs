use std::fmt::{self, Debug};

pub struct VarTerm {
    pub val: char,
    pub sign: String,
}

impl Debug for VarTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.sign, self.val)
    }
}
