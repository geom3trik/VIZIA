use std::str::FromStr;

use cssparser::*;
use morphorm::Units;

use crate::{CustomParseError, Parse};

use super::{Length, LengthOrPercentage};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LineHeight {
    /// The line height is based on the font.
    #[default]
    Normal,
    /// A multiple of the view's font size.
    Number(f32),
    /// An explicit height.
    Length(LengthOrPercentage),
}

impl<'i> Parse<'i> for LineHeight {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        if input.try_parse(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(LineHeight::Normal);
        }

        if let Ok(val) = input.try_parse(|input| f32::parse(input)) {
            return Ok(LineHeight::Number(val));
        }

        if let Ok(val) = input.try_parse(|input| LengthOrPercentage::parse(input)) {
            return Ok(LineHeight::Length(val));
        }

        let location = input.current_source_location();

        Err(location.new_custom_error(CustomParseError::InvalidValue))
    }
}

impl From<f32> for LineHeight {
    fn from(value: f32) -> Self {
        LineHeight::Number(value)
    }
}

impl From<f64> for LineHeight {
    fn from(value: f64) -> Self {
        LineHeight::Number(value as f32)
    }
}

impl From<Units> for LineHeight {
    fn from(value: Units) -> Self {
        match value {
            Units::Pixels(val) => LineHeight::Length(Length::px(val).into()),
            Units::Percentage(val) => LineHeight::Length(LengthOrPercentage::Percentage(val)),
            _ => LineHeight::Normal,
        }
    }
}

impl From<Length> for LineHeight {
    fn from(value: Length) -> Self {
        LineHeight::Length(value.into())
    }
}

impl From<LengthOrPercentage> for LineHeight {
    fn from(value: LengthOrPercentage) -> Self {
        LineHeight::Length(value)
    }
}

impl From<&str> for LineHeight {
    fn from(s: &str) -> Self {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        LineHeight::parse(&mut parser).unwrap_or_default()
    }
}

impl std::fmt::Display for LineHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineHeight::Normal => write!(f, "normal"),
            LineHeight::Number(num) => write!(f, "{:.2}", *num),
            LineHeight::Length(l) => match l {
                LengthOrPercentage::Percentage(p) => write!(f, "{:.0}%", *p),
                LengthOrPercentage::Length(ll) => write!(f, "{:.1}px", ll.to_px().unwrap()),
            },
        }
    }
}

impl FromStr for LineHeight {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        Ok(LineHeight::parse(&mut parser).unwrap_or_default())
    }
}
