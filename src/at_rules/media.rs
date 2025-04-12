use crate::{
    Stream,
    Error,
    Rule,
};
use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

use super::at_rule::{is_keyword, Comparison};


/// an @media rule
#[derive(Clone, Debug, PartialEq)]
pub struct Media<'a> {
    /// the query params
    pub query: Vec<MediaQuery<'a>>,
    /// the rules inside the body
    pub rules: Vec<Rule<'a>>
}
impl<'a> Media<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        let first = MediaQuery::consume(s)?;
        let mut list = vec![first];

        s.skip_spaces_and_comments()?;
        while s.curr_byte()? != b'{' {
            s.advance(1);
            s.skip_spaces_and_comments()?;
            list.push(MediaQuery::consume(s)?);
            s.skip_spaces_and_comments()?;
        }

        s.skip_spaces_and_comments()?;
        s.consume_byte(b'{')?;
        let mut rules = Vec::new();
        crate::consume_rule_set(s, &mut rules)?;
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'}')?;

        Ok(Self {
            query: list,
            rules,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// `not` or `only`
pub enum MediaNotOnly {
    /// not
    Not,
    /// only
    Only,
}

#[derive(Clone, Debug, PartialEq)]
/// An @media query
pub enum MediaQuery<'a> {
    Condition(MediaCondition<'a>),
    OtherThing {
        not_only: Option<MediaNotOnly>, 
        media_type: &'a str,
        conditions: Vec<MediaConditionWithoutOr<'a>>,
    }
}
impl<'a> MediaQuery<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        if is_keyword(s, "not") {
            s.advance(3);
            s.skip_spaces_and_comments()?;

            if s.curr_byte()? == b'(' {
                // media-not
                let a = MediaCondition::Not(Box::new(MediaInParens::consume(s)?));
                Ok(Self::Condition(a))
            } else {
                // media-type
                let media_type = s.consume_ident()?;
                s.skip_spaces_and_comments()?;

                let mut conditions = Vec::new();
                while is_keyword(s, "and") {
                    s.advance(3);
                    s.skip_spaces_and_comments()?;
                    conditions.push(MediaConditionWithoutOr::consume(s)?);
                    s.skip_spaces_and_comments()?;
                }

                Ok(Self::OtherThing { 
                    not_only: Some(MediaNotOnly::Not), 
                    media_type, 
                    conditions,
                })
            }
        } else if is_keyword(s, "only") {
            // media-type
            s.advance(4);
            s.skip_spaces_and_comments()?;
            let media_type = s.consume_ident()?;
            s.skip_spaces_and_comments()?;

            let mut conditions = Vec::new();
            while is_keyword(s, "and") {
                s.advance(3);
                s.skip_spaces_and_comments()?;
                conditions.push(MediaConditionWithoutOr::consume(s)?);
                s.skip_spaces_and_comments()?;
            }

            Ok(Self::OtherThing { 
                not_only: Some(MediaNotOnly::Only), 
                media_type, 
                conditions,
            })
        } else if s.curr_byte()? == b'(' {
            let media = MediaInParens::consume(s)?;
            s.skip_spaces_and_comments()?;
            let conditions = MediaAndOr::consume_many(s)?;
            Ok(Self::Condition(MediaCondition::List { 
                first: Box::new(media), 
                conditions,
            }))
        } else {
            // media-type
            s.skip_spaces_and_comments()?;
            let media_type = s.consume_ident()?;
            s.skip_spaces_and_comments()?;

            let mut conditions = Vec::new();
            while is_keyword(s, "and") {
                s.advance(3);
                s.skip_spaces_and_comments()?;
                conditions.push(MediaConditionWithoutOr::consume(s)?);
                s.skip_spaces_and_comments()?;
            }

            Ok(Self::OtherThing { 
                not_only: None, 
                media_type, 
                conditions,
            })
        }

    }
}

/// An @media condition
#[derive(Clone, Debug, PartialEq)]
pub enum MediaCondition<'a> {
    /// The condition should be negated
    Not(Box<MediaInParens<'a>>),
    /// A (potential) list of conditions
    List {
        /// The first condition in the list
        first: Box<MediaInParens<'a>>,
        /// All subsequent conditions in the list
        conditions: Vec<MediaAndOr<'a>>
    },
}
impl<'a> MediaCondition<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        if is_keyword(s, "not") {
            s.advance(3);
            let media = MediaInParens::consume(s)?;
            Ok(Self::Not(Box::new(media)))
        } else {
            let media = MediaInParens::consume(s)?;
            let conditions = MediaAndOr::consume_many(s)?;
            Ok(Self::List {
                first: Box::new(media),
                conditions,
            })
        }
    }
}

/// A media condition without the ability to 'or'
#[derive(Clone, Debug, PartialEq)]
pub enum MediaConditionWithoutOr<'a> {
    /// The media query should be negated
    Not(MediaInParens<'a>),
    /// The list should evaluate to true
    Media {
        /// The first in the list
        media: MediaInParens<'a>,
        /// Any following conditions
        conditions: Vec<MediaAnd<'a>>
    },
}
impl<'a> MediaConditionWithoutOr<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        if is_keyword(s, "not") {
            s.advance(3);
            Ok(Self::Not(MediaInParens::consume(s)?))
        } else {
            let media = MediaInParens::consume(s)?;
            s.skip_spaces_and_comments()?;

            let mut conditions = Vec::new();
            while is_keyword(s, "and") {
                s.advance(3);
                conditions.push(MediaAnd(MediaInParens::consume(s)?));
                s.skip_spaces_and_comments()?;
            }

            Ok(Self::Media { media, conditions })
        }
    }
}

/// Wrapper for when `MediaInParens` can only be preceeded by an "and"
#[derive(Clone, Debug, PartialEq)]
pub struct MediaAnd<'a>(pub MediaInParens<'a>);

/// Wrapper for when `MediaInParens` can be preceeded by an "and" or an "or"
#[derive(Clone, Debug, PartialEq)]
pub enum MediaAndOr<'a> {
    /// Prefixed with 'and'
    And(MediaInParens<'a>),
    /// Prefixed with 'or'
    Or(MediaInParens<'a>),
}
impl<'a> MediaAndOr<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        
        if is_keyword(s, "and") {
            s.advance(3);
            s.skip_spaces_and_comments()?;

            Ok(Self::And(MediaInParens::consume(s)?))
        } else if is_keyword(s, "or") {
            s.advance(2);
            s.skip_spaces_and_comments()?;

            Ok(Self::And(MediaInParens::consume(s)?))
        } else {
            Err(Error::InvalidIdent(s.gen_text_pos()))
        }
    }

    fn consume_many(s: &mut Stream<'a>) -> Result<Vec<Self>, Error> {
        let mut list = Vec::new();
        s.skip_spaces_and_comments()?;

        while is_keyword(s, "and") || is_keyword(s, "or") {
            list.push(Self::consume(s)?);
            s.skip_spaces_and_comments()?;
        }

        Ok(list)
    }
}


/// a media condition inside of parenthesis
#[derive(Clone, Debug, PartialEq)]
pub enum MediaInParens<'a> {
    /// a condition
    Condition(MediaCondition<'a>),
    /// a feature
    Feature(MediaFeature<'a>),
}
impl<'a> MediaInParens<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        let start = s.pos();
        if let Ok(feature) = MediaFeature::consume(s) {
            Ok(Self::Feature(feature))
        } else {
            s.reset_pos(start);
            s.consume_byte(b'(')?;
            s.skip_spaces_and_comments()?;
            let cond = MediaCondition::consume(s)?;
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Ok(Self::Condition(cond))
        }
    }
}

/// An @media feature
#[derive(Clone, Debug, PartialEq)]
pub enum MediaFeature<'a> {
    /// A key should equal a value
    KeyVal {
        /// The property
        key: &'a str,
        /// The value
        val: &'a str,
    },
    
    /// An ident
    Name(&'a str),

    /// A range
    Range(MediaRange<'a>)
}
impl<'a> MediaFeature<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'(')?;
        s.skip_spaces_and_comments()?;

        let first = consume_value2(s);
        s.skip_spaces_and_comments()?;

        if s.curr_byte()? == b':' {
            s.advance(1);
            s.skip_spaces_and_comments()?;
            let second = consume_value2(s);
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            s.skip_spaces_and_comments()?;
            Ok(Self::KeyVal { key: first, val: second })
        } else if let Some(range) = MediaRange::try_consume(s)? {
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Ok(Self::Range(range))
        } else {
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Ok(Self::Name(first))
        }
    }
}

/// A range comparison
#[derive(Clone, Debug, PartialEq)]
pub enum MediaRange<'a> {
    /// A typical comparison
    /// 
    /// ie lhs > rhs
    Regular {
        /// Left hand side
        lhs: &'a str,
        /// Comparison to perform
        comp: Comparison<'a>,
        /// Right hand side
        rhs: &'a str,
    },

    /// lhs < ident < rhs
    LessThanChain {
        /// Left hand side
        lhs: &'a str,
        /// Middle 
        ident: &'a str,
        /// Right hand side
        rhs: &'a str,
    },

    /// lhs <= ident <= rhs
    LessEqChain {
        /// Left hand side
        lhs: &'a str,
        /// Middle 
        ident: &'a str,
        /// Right hand side
        rhs: &'a str,
    },

    /// lhs > ident > rhs
    GreaterThanChain {
        /// Left hand side
        lhs: &'a str,
        /// Middle 
        ident: &'a str,
        /// Right hand side
        rhs: &'a str,
    },

    /// lhs >= ident >= rhs
    GreaterEqChain {
        /// Left hand side
        lhs: &'a str,
        /// Middle 
        ident: &'a str,
        /// Right hand side
        rhs: &'a str,
    },
}
impl<'a> MediaRange<'a> {
    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        let start_pos = s.pos();

        let lhs = consume_value2(s);

        s.skip_spaces_and_comments()?;
        let tail = s.slice_tail();

        let comp_str;
        let comp  = if tail.starts_with("<") {
            comp_str = "<";
            Comparison::Less
        } else if tail.starts_with("<=") {
            comp_str = "<=";
            Comparison::LessEq
        } else if tail.starts_with(">") {
            comp_str = ">";
            Comparison::Greater
        } else if tail.starts_with(">=") {
            comp_str = ">=";
            Comparison::GreaterEq
        } else if tail.starts_with("=") {
            comp_str = "=";
            Comparison::Equal
        } else {
            s.reset_pos(start_pos);
            return Ok(None)
        };

        s.advance(comp_str.len());
        s.skip_spaces_and_comments()?;

        let value = consume_value2(s);
        s.skip_spaces_and_comments()?;

        if s.slice_tail().starts_with(comp_str) {
            s.advance(comp_str.len());
            let rhs = consume_value2(s);
            let a = match comp {
                Comparison::Greater => {
                    Self::GreaterThanChain { lhs, ident: value, rhs }
                }
                Comparison::GreaterEq => {
                    Self::GreaterEqChain { lhs, ident: value, rhs }
                }
                Comparison::Less => {
                    Self::LessThanChain { lhs, ident: value, rhs }
                }
                Comparison::LessEq => {
                    Self::LessEqChain { lhs, ident: value, rhs }
                }
                // TODO: proper error for this?
                _ => return Err(Error::UnexpectedCombinator),
            };

            Ok(Some(a))
        } else {
            Ok(Some(Self::Regular { lhs, comp, rhs: value }))
        }
    }
}


fn consume_value2<'a>(s: &mut Stream<'a>) -> &'a str {
    s.consume_bytes(|b| ![b'>', b'<', b'=', b')', b':'].contains(&b))
}
