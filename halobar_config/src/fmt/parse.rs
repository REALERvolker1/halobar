use super::*;

/// Parse a variable string
///
/// Refer to the crate documentation for more information, including format syntax.
///
/// ```
/// use halobar_fmt::HaloFormatter;
/// pub struct State {
///     is_enabled: bool,
///     current_state: i32,
/// }
/// impl HaloFormatter for State {
///     fn variable_map<'a, 'b: 'a>(
///         &'a self,
///         segments: halobar_fmt::FmtSegments<'b>,
///     ) -> Result<ahash::AHashMap<&'a str, bool>, halobar_fmt::FormatStrError> {
///         halobar_fmt::variable_map(&["is_enabled", "current_state"], segments)
///     }
///     fn format(&self, segments: halobar_fmt::FmtSegments) -> Result<String, halobar_fmt::FormatStrError> {
///         let mut out = String::new();
///         for segment in segments {
///             match segment {
///                 halobar_fmt::Segment::Literal(s) => out.push_str(&s),
///                 halobar_fmt::Segment::Variable(v) => match v.ident.as_str() {
///                     "is_enabled" => {
///                         if self.is_enabled {
///                             let truthy = v.truthy.join("true");
///                             out.push_str(&truthy);
///                         } else {
///                             out.push_str(&v.falsy);
///                         }
///                     }
///                     "current_state" => {
///                         if self.current_state != 0 {
///                             out.push_str(&v.truthy(&self.current_state.to_string()))
///                         } else {
///                             out.push_str(&v.falsy)
///                         }
///                     }
///                     _ => return Err(halobar_fmt::FormatStrError::InvalidVariable(v.ident.clone())),
///                 },
///             }
///         }
///         Ok(out)
///     }
/// }
///
/// fn main() {
///     let string = "This is {is_enabled?An enabled struct:disabled} and the current state {current_state?is $:isn't valid}";
///     let formatted = halobar_fmt::parse(string).unwrap();
///
///     let true_state = State {
///         is_enabled: true,
///         current_state: 8,
///     };
///
///     let false_state = State {
///         is_enabled: false,
///         current_state: 0,
///     };
///
///     assert_eq!(
///         true_state.format(formatted.segments()).unwrap(),
///         String::from("This is An enabled struct and the current state is 8")
///     );
///     assert_eq!(
///         false_state.format(formatted.segments()).unwrap(),
///         String::from("This is disabled and the current state isn't valid")
///     );
///
///     println!(
///         "{}\n{}",
///         true_state.format(formatted.segments()).unwrap(),
///         false_state.format(formatted.segments()).unwrap()
///     );
/// }
/// ```
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
