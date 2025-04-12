use alloc::vec::Vec;
use crate::{
    Stream,
    Error,
};
use super::media::MediaQuery;
use super::at_rule::is_keyword;

/// An @import rule
#[derive(Clone, Debug, PartialEq)]
pub struct Import<'a> {
    /// The url to import from
    pub url: ImportUrl<'a>,

    /// The layer if specified
    pub layer: Option<ImportLayer<'a>>,

    /// The supports condition if specified
    pub supports: Option<ImportConditionSupports<'a>>,

    /// A list of media queries
    pub media_queries: Vec<MediaQuery<'a>>,
}
impl<'a> Import<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        // read url
        let url = ImportUrl::consume(s)?;

        // read layer maybe
        let mut layer = None;
        s.skip_spaces_and_comments()?;
        if s.slice_tail().starts_with("layer") {
            s.advance(5);
            if s.curr_byte()? == b'(' {
                s.advance(1);
                s.skip_spaces_and_comments()?;
                let name = s.consume_ident_special()?;
                s.skip_spaces_and_comments()?;
                s.consume_byte(b')')?;
                layer = Some(ImportLayer::Named(name));
            } else {
                layer = Some(ImportLayer::Layer);
            }
        }

        // supports
        let mut supports = None;
        s.skip_spaces_and_comments()?;
        if s.slice_tail().starts_with("supports") {
            s.advance(8);
            s.consume_byte(b'(')?;
            s.skip_spaces_and_comments()?;

            if is_keyword(s, "not") {
                supports = Some(ImportConditionSupports::SupportsCondition(
                    super::supports::SupportsCondition::consume(s)?
                ));
            } else {
                // try declaration first
                // TODO: i got a bit lazy here, is there a more proper way to do this?
                let pos = s.pos();
                match s.consume_ident() {
                    Ok(name) => {
                        s.skip_spaces_and_comments()?;
                        if s.curr_byte()? == b':' {
                            s.advance(1);
                            s.skip_spaces_and_comments()?;
                            let value = crate::consume_value(s)?;

                            supports = Some(ImportConditionSupports::Declaration(
                                crate::Declaration { name, value, important: false }
                            ));
                        } else {
                            s.reset_pos(pos);
                            supports = Some(ImportConditionSupports::SupportsCondition(
                                super::supports::SupportsCondition::consume(s)?
                            ));
                        }
                    }

                    Err(_) => {
                        s.reset_pos(pos);
                        supports = Some(ImportConditionSupports::SupportsCondition(
                            super::supports::SupportsCondition::consume(s)?
                        ));
                    }
                }
            }

            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
        }

        // media queries
        let mut media_queries = Vec::new();
        s.skip_spaces_and_comments()?;
        while s.curr_byte()? != b';' {
            s.skip_spaces_and_comments()?;
            media_queries.push(MediaQuery::consume(s)?);
            s.skip_spaces_and_comments()?;
            s.try_consume_byte(b',');
            s.skip_spaces_and_comments()?;
        }
        s.consume_byte(b';')?;

        Ok(Self {
            url,
            layer,
            supports,
            media_queries
        })
    }   
}

/// The url for an @import rule
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImportUrl<'a> {
    /// A url(...)
    Url(&'a str),
    /// An src(...)
    Src(&'a str),
    /// A raw string 
    String(&'a str),
}
impl<'a> ImportUrl<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        if s.slice_tail().starts_with("url") {
            // url
            s.advance(3);
            s.skip_spaces_and_comments()?;
            s.consume_byte(b'(')?;
            s.skip_spaces_and_comments()?;
            let url = s.consume_string()?;
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Ok(Self::Url(url))
        } else if s.slice_tail().starts_with("src") {
            // src
            s.advance(3);
            s.skip_spaces_and_comments()?;
            s.consume_byte(b'(')?;
            s.skip_spaces_and_comments()?;
            let src = s.consume_string()?;
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Ok(Self::Src(src))
        } else {
            // string
            Ok(Self::String(s.consume_string()?))
        }
    }
}


/// An @import supports condition
#[derive(Clone, Debug, PartialEq)]
pub enum ImportConditionSupports<'a> {
    /// An @supports condition
    SupportsCondition(super::supports::SupportsCondition<'a>),

    /// A key: value declaration
    Declaration(crate::Declaration<'a>),
}

/// Layer info in an @import rule
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImportLayer<'a> {
    /// The `layer` keyword was specified
    Layer,

    /// A layer was named
    Named(&'a str),
} 
