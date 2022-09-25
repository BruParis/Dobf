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

#[derive(Debug, PartialEq)]
pub enum ExprError {
    RPNEmpty(),
    RPNSyntaxError(),
    SingleTerm(),
    VarError(),
}

#[derive(Debug, PartialEq)]
pub enum ArenaError {
    NotFound(),
    NotANode(),
    ParentIsLeaf(),
    ElemIsLeaf(),
    ElemIsNode(),
    ElemIsFree(),
}

//pub type Result<T> = std::result::Result<T, ParseError>;
//pub type Result<T> = std::result::Result<T, DAGError>;
