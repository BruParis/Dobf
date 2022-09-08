#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissClosePar(String),
    MissOpenPar(String),
    WrongSeqChar(String),
    WrongChar(String),
    DanglingNegSign(),
    NotOp(),
}

pub type Result<T> = std::result::Result<T, ParseError>;
