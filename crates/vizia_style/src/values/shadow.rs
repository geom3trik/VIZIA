use crate::{Color, CustomParseError, Length, Parse};
use cssparser::*;

/// A box shadow adding a shadow effect around an element's frame.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Shadow {
    /// The horizontal offset of the box shadow.
    pub x_offset: Length,
    /// The vertical offset of the box shadow.
    pub y_offset: Length,
    /// The blur radius used for making the box shadow bigger and lighter.
    pub blur_radius: Option<Length>,
    /// The spread radius used for expanding and growing the box shadow.
    pub spread_radius: Option<Length>,
    /// The color of the box shadow.
    pub color: Option<Color>,
    /// Determines if the box shadow should be an outer shadow (outset) or an inner shadow (inset).
    pub inset: bool,
}

impl Shadow {
    /// Creates a new shadow.
    pub fn new(
        x_offset: impl Into<Length>,
        y_offset: impl Into<Length>,
        blur_radius: Option<Length>,
        spread_radius: Option<Length>,
        color: Option<Color>,
        inset: bool,
    ) -> Self {
        Self {
            x_offset: x_offset.into(),
            y_offset: y_offset.into(),
            blur_radius,
            spread_radius,
            color,
            inset,
        }
    }
}

impl<'i> Parse<'i> for Shadow {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        let mut color = None;
        let mut lengths = None;
        let mut inset = false;

        loop {
            if !inset && input.try_parse(|input| input.expect_ident_matching("inset")).is_ok() {
                inset = true;
                continue;
            }

            if lengths.is_none() {
                let value = input.try_parse::<_, _, ParseError<CustomParseError<'i>>>(|input| {
                    let horizontal = Length::parse(input)?;
                    let vertical = Length::parse(input)?;
                    let blur = input.try_parse(Length::parse).ok();
                    let spread = input.try_parse(Length::parse).ok();
                    Ok((horizontal, vertical, blur, spread))
                });

                if let Ok(value) = value {
                    lengths = Some(value);
                    continue;
                }
            }

            if color.is_none() {
                if let Ok(value) = input.try_parse(Color::parse) {
                    color = Some(value);
                    continue;
                }
            }

            break;
        }

        let lengths = lengths.ok_or(input.new_error(BasicParseErrorKind::QualifiedRuleInvalid))?;
        Ok(Shadow {
            color,
            x_offset: lengths.0,
            y_offset: lengths.1,
            blur_radius: lengths.2,
            spread_radius: lengths.3,
            inset,
        })
    }
}

impl<'i> Parse<'i> for Vec<Shadow> {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        input.parse_comma_separated(Shadow::parse)
    }
}

impl From<&str> for Shadow {
    fn from(s: &str) -> Self {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        Shadow::parse(&mut parser).unwrap_or_default()
    }
}

/// A text shadow.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextShadow {
    /// The horizontal offset of the box shadow.
    pub x_offset: Length,
    /// The vertical offset of the box shadow.
    pub y_offset: Length,
    /// The blur radius used for making the box shadow bigger and lighter.
    pub blur_radius: Option<Length>,
    /// The color of the box shadow.
    pub color: Option<Color>,
}

impl TextShadow {
    /// Creates a new shadow.
    pub fn new(
        x_offset: impl Into<Length>,
        y_offset: impl Into<Length>,
        blur_radius: Option<Length>,
        color: Option<Color>,
    ) -> Self {
        Self { x_offset: x_offset.into(), y_offset: y_offset.into(), blur_radius, color }
    }
}

impl<'i> Parse<'i> for TextShadow {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        let mut color = None;
        let mut lengths = None;

        loop {
            if lengths.is_none() {
                let value = input.try_parse::<_, _, ParseError<CustomParseError<'i>>>(|input| {
                    let horizontal = Length::parse(input)?;
                    let vertical = Length::parse(input)?;
                    let blur = input.try_parse(Length::parse).ok();
                    Ok((horizontal, vertical, blur))
                });

                if let Ok(value) = value {
                    lengths = Some(value);
                    continue;
                }
            }

            if color.is_none() {
                if let Ok(value) = input.try_parse(Color::parse) {
                    color = Some(value);
                    continue;
                }
            }

            break;
        }

        let lengths = lengths.ok_or(input.new_error(BasicParseErrorKind::QualifiedRuleInvalid))?;
        Ok(TextShadow { color, x_offset: lengths.0, y_offset: lengths.1, blur_radius: lengths.2 })
    }
}

impl<'i> Parse<'i> for Vec<TextShadow> {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        input.parse_comma_separated(TextShadow::parse)
    }
}

impl From<&str> for TextShadow {
    fn from(s: &str) -> Self {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        TextShadow::parse(&mut parser).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::assert_parse;

    assert_parse! {
        Shadow, parse_shadow,

        custom {
            success {
                "10px 20px" => Shadow::new(
                    Length::px(10.0),
                    Length::px(20.0),
                    None,
                    None,
                    None,
                    false,
                ),
                "10px 20px 30px 40px red inset" => Shadow::new(
                    Length::px(10.0),
                    Length::px(20.0),
                    Some(Length::px(30.0)),
                    Some(Length::px(40.0)),
                    Some(Color::rgb(255, 0, 0)),
                    true,
                ),
            }

            failure {
                "test",
                "123",
            }
        }
    }

    assert_parse! {
        Vec<Shadow>, parse_vec_shadow,

        custom {
            success {
                "10px 20px, 10px 20px 30px 40px red inset" => vec![
                    Shadow::new(
                        Length::px(10.0),
                        Length::px(20.0),
                        None,
                        None,
                        None,
                        false,
                    ),
                    Shadow::new(
                        Length::px(10.0),
                        Length::px(20.0),
                        Some(Length::px(30.0)),
                        Some(Length::px(40.0)),
                        Some(Color::rgb(255, 0, 0)),
                        true,
                    ),
                ],
            }

            failure {
                "10px, 10px, 10px",
                "test",
                "123",
            }
        }
    }
}
