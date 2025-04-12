use crate::{
    Stream,
    Error,
    Rule,
    Declaration
};
use super::at_rule::is_keyword;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// An @supports rule
#[derive(Clone, Debug, PartialEq)]
pub struct Supports<'a> {
    /// The condition
    pub condition: SupportsCondition<'a>,

    /// the rules in the block body
    pub rules: Vec<Rule<'a>>,
}
impl<'a> Supports<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        let condition = SupportsCondition::consume(s)?;
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'{')?;
        
        let mut rules = Vec::new();
        crate::consume_rule_set(s, &mut rules)?;
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'}')?;

        Ok(Self {
            condition,
            rules,
        })
    }
}


/// The type of condition
#[derive(Clone, Debug, PartialEq)]
pub enum SupportsCondition<'a> {
    /// The condition should be negated
    Not(SupportsInParens<'a>),
    /// A (possible) list of conditions
    List {
        /// The first condition in the list
        first: SupportsInParens<'a>,
        /// The rest of the conditions
        list: Vec<SupportsAndOr<'a>>,
    }
}
impl<'a> SupportsCondition<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        if is_keyword(s, "not") {
            s.advance(3);
            Ok(Self::Not(SupportsInParens::consume(s)?))
        } else {
            Ok(Self::List { 
                first: SupportsInParens::consume(s)?, 
                list: SupportsAndOr::consume_many(s)?,
            })
        }
    }

    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        s.skip_spaces_and_comments()?;
        
        if is_keyword(s, "not") || s.curr_byte()? == b'(' {
            Ok(Some(Self::consume(s)?))
        } else {
            Ok(None)
        }
    }
}

/// A support condition prefixed with 'and' or 'or'
#[derive(Clone, Debug, PartialEq)]
pub enum SupportsAndOr<'a> {
    /// A support condition prefixed with 'and'
    And(SupportsInParens<'a>),
    /// A support condition prefixed with 'or'
    Or(SupportsInParens<'a>),
}
impl<'a> SupportsAndOr<'a> {
    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        s.skip_spaces_and_comments()?;

        if is_keyword(s, "and") {
            s.advance(3);
            Ok(Some(Self::And(SupportsInParens::consume(s)?)))
        } else if is_keyword(s, "or") {
            s.advance(2);
            Ok(Some(Self::Or(SupportsInParens::consume(s)?)))
        } else {
            Ok(None)
        }
    }

    fn consume_many(s: &mut Stream<'a>) -> Result<Vec<Self>, Error> {
        let mut list = Vec::new();
        while let Some(i) = Self::try_consume(s)? {
            list.push(i);
        }
        Ok(list)
    }
}

/// An inner @supports condition
#[derive(Clone, Debug, PartialEq)]
pub enum SupportsInParens<'a> {
    /// Another condition
    Condition(Box<SupportsCondition<'a>>),
    /// A declaration
    Feature(Declaration<'a>),
}
impl<'a> SupportsInParens<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'(')?;

        let out = if let Some(cond) = SupportsCondition::try_consume(s)? {
            Self::Condition(Box::new(cond))
        } else {
            Self::Feature(crate::consume_declaration(s)?)
        };

        s.skip_spaces_and_comments()?;
        s.consume_byte(b')')?;
        
        Ok(out)
    }
}
