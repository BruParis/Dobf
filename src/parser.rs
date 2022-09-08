use std::fs::File;
use std::io::{self, BufRead};
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
pub fn parse_rpn(mut line: String) -> Result<Vec<String>, ParseError> {
    println!("expr: {}", line);
    let mut neg_sign = false;
    let mut res_rpn: Vec<String> = Vec::new();
    let mut sign_count = 0;
    let mut op_stack: Vec<char> = Vec::new();
    let mut curr_int = String::new();

    // remove whitespace
    line = line.replace(" ", "");
    line.push(' ');
    line = " ".to_string() + &line;

    let inter = line.chars().collect::<Vec<char>>();
    for w in inter.windows(2) {
        if neg_sign && w[0] != '~' {
            op_stack.push('~');
            neg_sign = false;
        }

        match w[0] {
            '(' => {
                if "+-.^&|".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong sequence of char: {}{}",
                        w[0], w[1]
                    )));
                }

                op_stack.push('(');
            }
            ')' => {
                // while there is no openning parenthesis at top of stack...
                while op_stack.last() != Some(&'(') {
                    // ... check stack is not empty (or else it means a mismatch in parenthesis)
                    if op_stack.len() == 0 {
                        return Err(ParseError::MissOpenPar("Missing (".to_string()));
                    }

                    if let Some(op) = op_stack.pop() {
                        res_rpn.push(op.to_string());
                    }
                }

                // ... check there is indeed an openning parenthesis at top of stack
                // ... and discard it
                if op_stack.last() == Some(&'(') {
                    op_stack.pop();
                } else {
                    return Err(ParseError::MissOpenPar("Missing (".to_string()));
                }

                // if there is a negative sign, pop and push
                if op_stack.last() == Some(&'~') {
                    op_stack.pop();
                    res_rpn.push('~'.to_string());
                }
            }
            '~' => {
                if w[1] != '~' && !w[1].is_alphanumeric() && !"()".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong sequence of char: {}{}",
                        w[0], w[1]
                    )));
                }

                neg_sign = !neg_sign;
            }
            '+' | '-' | '.' | '^' | '&' | '|' => {
                if "+-.^&|".contains(w[1]) {
                    return Err(ParseError::WrongSeqChar(format!(
                        "wrong sequence of char: {}{}",
                        w[0], w[1]
                    )));
                }

                while let Some(op) = op_stack.last() {
                    if op == &'(' || op_stack.len() == 0 {
                        break;
                    }

                    let (op_prec, _) = preced_assoc(&op)?;
                    let (w_prec, w_assoc) = preced_assoc(&w[0])?;
                    if op_prec > w_prec || (op_prec == w_prec && w_assoc != Assoc::Right) {
                        res_rpn.push(op.to_string());
                        op_stack.pop();
                    } else {
                        break;
                    }
                }

                op_stack.push(w[0]);
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                curr_int.push(w[0]);

                // if next char is not numeric, current int parsing is done
                if !w[1].is_numeric() {
                    res_rpn.push(curr_int);
                    curr_int = String::new();
                }
            }
            _ => {
                if w[0].is_alphabetic() {
                    res_rpn.push(w[0].to_string());
                } else if w[0] != ' ' {
                    return Err(ParseError::WrongChar("Wrong char".to_string()));
                }
            }
        }
    }

    if neg_sign {
        return Err(ParseError::DanglingNegSign());
    }

    while sign_count > 0 {
        res_rpn.push("~".to_string());
        sign_count -= 1;
    }

    while let Some(op) = op_stack.pop() {
        if op == '(' {
            return Err(ParseError::MissClosePar("Missing )".to_string()));
        } else if op == ')' {
            return Err(ParseError::MissOpenPar("Missing (".to_string()));
        }
        res_rpn.push(op.to_string());
    }

    Ok(res_rpn)
}

fn preced_assoc(op: &char) -> Result<(i8, Assoc), ParseError> {
    match op {
        '+' => Ok((2, Assoc::Both)),
        '-' => Ok((2, Assoc::Both)),
        '^' => Ok((3, Assoc::Both)),
        '&' => Ok((4, Assoc::Both)),
        '|' => Ok((4, Assoc::Both)),
        '.' => Ok((5, Assoc::Both)),
        '~' => Ok((6, Assoc::Right)),
        _ => Err(ParseError::NotOp()),
    }
}
