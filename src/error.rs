#[derive(Debug)]
pub enum ParseError {
    MissClosePar(String),
    MissOpenPar(String),
    SuccessiveOp(String),
    WrongChar(String),
}
