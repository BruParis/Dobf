#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissClosePar(String),
    MissOpenPar(String),
    WrongSeqChar(String),
    WrongChar(String),
    DanglingNegSign(),
    NotOp(),
}

#[derive(Debug, PartialEq)]
pub enum DAGError {
    RPNEmpty(),
    RPNSyntaxError(),
    SingleTerm(),
    VarError(),
}

//pub type Result<T> = std::result::Result<T, ParseError>;
//pub type Result<T> = std::result::Result<T, DAGError>;
