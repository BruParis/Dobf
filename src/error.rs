#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissClosePar(String),
    MissOpenPar(String),
    SuccessiveOp(String),
    WrongChar(String),
    NegSignOp(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;
