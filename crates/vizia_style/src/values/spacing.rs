use cssparser::*;

use crate::{CustomParseError, Parse};

use super::Length;

#[derive(Debug, Clone, PartialEq)]
pub struct Spacing(pub Length);

impl<'i> Parse<'i> for Spacing {
    fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, CustomParseError<'i>>> {
        if input.try_parse(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(Spacing(Length::zero()));
        }

        if let Ok(lp) = input.try_parse(|input| Length::parse(input)) {
            return Ok(Spacing(lp));
        }

        let location = input.current_source_location();

        Err(location.new_custom_error(CustomParseError::InvalidValue))
    }
}
