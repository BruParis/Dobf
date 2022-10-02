use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead};
use std::mem;
use std::path::Path;

use crate::error::ParseError;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug, PartialEq)]
enum Assoc {
    Right,
    Left,
    Both,
}

// Shunting yard algorithm
pub fn parse_rpn(mut line: String) -> Result<VecDeque<String>, ParseError> {
    println!("expr: {}", line);
    let mut res_rpn: VecDeque<String> = VecDeque::new();
    let mut op_stack: Vec<String> = Vec::new();
    let mut curr_int = String::new();

    // remove whitespace
    line = line.replace(" ", "");
    line.push(' ');
    line = " ".to_string() + &line;

    let inter = line.chars().collect::<Vec<char>>();
    for w in inter.windows(2) {
        match w[0] {
            '(' => {
                if "+.^&|".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong seq of char: {}/{}",
                        w[0], w[1]
                    )));
                }

                op_stack.push("(".to_owned());
            }
            ')' => {
                // while there is no openning parenthesis at top of stack...
                while op_stack.last() != Some(&"(".to_owned()) {
                    // ... check stack is not empty (or else it means a mismatch in parenthesis)
                    if op_stack.len() == 0 {
                        return Err(ParseError::MissOpenPar("Missing (".to_string()));
                    }

                    if let Some(op) = op_stack.pop() {
                        res_rpn.push_back(op.to_string());
                    }
                }

                // ... check there is indeed an openning parenthesis at top of stack
                // ... and discard it
                if op_stack.last() == Some(&"(".to_owned()) {
                    op_stack.pop();
                } else {
                    return Err(ParseError::MissOpenPar("Missing (".to_string()));
                }

                // if there is a negative sign, pop and push
                if let Some(op) = op_stack.last() {
                    if op.len() > 1 {
                        op_stack.pop();
                        res_rpn.push_back('~'.to_string());
                    }
                }
            }
            '-' | '~' => {
                if !w[1].is_alphanumeric() && !"()-~".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong seq of char: {}/{}",
                        w[0], w[1]
                    )));
                }

                op_stack.push(w[0].to_string());
            }
            '+' | '.' | '^' | '&' | '|' => {
                if "+.^&|".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong seq of char: {}/{}",
                        w[0], w[1]
                    )));
                }

                while let Some(op) = op_stack.last() {
                    if op == "(" || op_stack.len() == 0 {
                        break;
                    }

                    let (op_prec, _) = preced_assoc(&op)?;
                    let (w_prec, w_assoc) = preced_assoc(&w[0].to_string())?;
                    if op_prec > w_prec || (op_prec == w_prec && w_assoc != Assoc::Right) {
                        res_rpn.push_back(op.to_string());
                        op_stack.pop();
                    } else {
                        break;
                    }
                }

                op_stack.push(w[0].to_string());
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                curr_int.push(w[0]);

                // if next char is not numeric, current int parsing is done
                if !w[1].is_numeric() {
                    res_rpn.push_back(curr_int);
                    curr_int = String::new();
                }
            }
            _ => {
                if w[0].is_alphabetic() {
                    if w[1].is_alphabetic() || "(".contains(w[1]) {
                        return Err(ParseError::WrongSeqChar(format!(
                            "wrong seq of char: {}/{}",
                            w[0], w[1]
                        )));
                    }

                    res_rpn.push_back(w[0].to_string());
                } else if w[0] != ' ' {
                    return Err(ParseError::WrongChar("Wrong char".to_string()));
                }
            }
        }
    }

    while let Some(op) = op_stack.pop() {
        if op == "(" {
            return Err(ParseError::MissClosePar("Missing )".to_string()));
        } else if op == ")" {
            return Err(ParseError::MissOpenPar("Missing (".to_string()));
        }
        res_rpn.push_back(op.to_string());
    }

    // '-' is can be binary or unary operator
    // if binary operator, convert to unary with use of add ('-' -> '-', '+')
    let mut num_term = 0;
    let mut aux_rpn: VecDeque<String> = VecDeque::new();
    while let Some(e) = res_rpn.pop_front() {
        if e.chars().all(|c| c.is_alphanumeric()) {
            num_term += 1;
        }

        // '-' could be a binary op.
        if e.chars().all(|c| "+-.&^|".contains(c)) {
            num_term = 0;
        }

        // not necessary
        if num_term < 0 {
            panic!("wrong number of terms");
        }

        aux_rpn.push_back(e.clone());
        if e == "-" {
            if num_term % 2 == 0 {
                aux_rpn.push_back("+".to_string());
            }
        }
    }

    Ok(aux_rpn)
}

fn preced_assoc(op: &str) -> Result<(i8, Assoc), ParseError> {
    match op {
        "+" => Ok((2, Assoc::Both)),
        "-" => Ok((2, Assoc::Both)),
        "~" => Ok((2, Assoc::Both)),
        "^" => Ok((3, Assoc::Both)),
        "&" => Ok((4, Assoc::Both)),
        "|" => Ok((4, Assoc::Both)),
        "." => Ok((5, Assoc::Both)),
        _ => Err(ParseError::NotOp()),
    }
}
