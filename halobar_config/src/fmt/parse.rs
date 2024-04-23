use super::*;

/// Parse a variable string
///
/// Refer to [`HaloFormatter`] for more information, including format syntax.
pub fn parse<S: AsRef<str>>(interpolated_string: S) -> Result<FmtSegmentVec, FormatStrError> {
    let input_str = interpolated_string.as_ref();

    let mut segments = Vec::new();

    let mut current_literal = String::new();
    let mut current_variable = Variable::default();
    let mut current_truthy = String::new();
    let mut min_length = 0usize;

    let mut current_state = ParserState::Literal;
    let mut is_escaped = false;

    macro_rules! push_char {
        ($character:expr) => {
            min_length = min_length.saturating_add(1);
            is_escaped = false;
            match current_state {
                ParserState::Literal => current_literal.push($character),
                ParserState::VarIdent => current_variable.ident.push($character),
                ParserState::VarTruthy => current_truthy.push($character),
                ParserState::VarFalsy => current_variable.falsy.push($character),
            }
        };
        ($character:expr => $collect:expr) => {
            min_length = min_length.saturating_add(1);
            is_escaped = false;
            $collect.push($character)
        };
    }

    for (idx, character) in input_str.chars().enumerate() {
        if is_escaped {
            push_char!(character);
            continue;
        }

        match character {
            '{' => {
                if current_state == ParserState::Literal {
                    segments.push(Segment::Literal(take(&mut current_literal)));
                    current_state = ParserState::VarIdent;
                } else {
                    return Err(FormatStrError::InvalidSymbol(current_state, idx, character));
                }
            }
            '}' => match current_state {
                ParserState::VarIdent | ParserState::VarFalsy => {
                    segments.push(Segment::Variable(take(&mut current_variable)));
                    current_state = ParserState::Literal;
                }
                ParserState::Literal => {
                    return Err(FormatStrError::InvalidSymbol(current_state, idx, character));
                }
                // allow for {variable}
                ParserState::VarTruthy => {
                    current_variable.truthy.push(take(&mut current_truthy));
                    segments.push(Segment::Variable(take(&mut current_variable)));
                    current_state = ParserState::Literal;
                }
            },
            '\\' => is_escaped = true,
            '?' => {
                if current_state == ParserState::VarIdent {
                    current_state = ParserState::VarTruthy;
                } else {
                    push_char!(character);
                }
            }
            ':' => {
                if current_state == ParserState::VarTruthy {
                    current_variable.truthy.push(take(&mut current_truthy));
                    current_state = ParserState::VarFalsy;
                } else {
                    push_char!(character);
                }
            }
            '$' => {
                if current_state == ParserState::VarTruthy {
                    current_variable.truthy.push(take(&mut current_truthy));
                } else {
                    push_char!(character);
                }
            }
            _ => {
                push_char!(character);
            }
        }
    }

    Ok(FmtSegmentVec {
        min_length,
        inner: segments,
    })
}
