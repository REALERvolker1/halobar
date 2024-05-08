use super::*;

/// An enum used internally. It is marked as public because it could be part of an error message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// Currently parsing a Literal
    Literal,
    /// Parsing the variable name segment
    VarIdent,
    /// Parsing the truthy segment
    VarTruthy,
    /// Parsing the falsy segment
    VarFalsy,
}

/// Parse a variable string
///
/// Refer to [`HaloFormatter`] for more information, including format syntax.
#[tracing::instrument(level = "trace", skip_all)]
pub fn parse(input_str: &str) -> Result<FmtSegmentVec, FormatStrError> {
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
                ParserState::VarIdent => {
                    // This case is only if the variable format is {var}. Thus current_truthy should always be empty
                    debug_assert!(current_variable.truthy.is_empty());
                    current_variable.truthy.push(VarContentType::Value);

                    segments.push(Segment::Variable(take(&mut current_variable)));
                    current_state = ParserState::Literal;
                }
                ParserState::VarFalsy => {
                    segments.push(Segment::Variable(take(&mut current_variable)));
                    current_state = ParserState::Literal;
                }
                ParserState::Literal => {
                    return Err(FormatStrError::InvalidSymbol(current_state, idx, character));
                }
                ParserState::VarTruthy => {
                    let item = if current_truthy.is_empty() {
                        VarContentType::Value
                    } else {
                        take(&mut current_truthy).into()
                    };
                    current_variable.truthy.push(item);
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
                    current_variable
                        .truthy
                        .push(take(&mut current_truthy).into());
                    current_state = ParserState::VarFalsy;
                } else {
                    push_char!(character);
                }
            }
            '$' => {
                if current_state == ParserState::VarTruthy {
                    // push the stuff we have accumulated so far so we maintain order
                    current_variable
                        .truthy
                        .push(take(&mut current_truthy).into());

                    current_variable.truthy.push(VarContentType::Value);
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
