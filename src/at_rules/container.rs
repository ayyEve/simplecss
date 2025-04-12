use crate::{StyleSheet, Stream, Error};
use super::at_rule::{Comparison, is_keyword};
use alloc::vec::Vec;
use alloc::boxed::Box;


#[derive(Clone, Debug, PartialEq)]
/// An @container rule
pub struct Container<'a> {
    /// the rule conditions
    pub conditions: Vec<ContainerCondition<'a>>,
    /// the contents of the rule's block
    pub contents: StyleSheet<'a>,
}
impl<'a> Container<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;

        let mut conditions = Vec::new();
        loop {
            conditions.push(ContainerCondition::consume(s)?);
            s.skip_spaces_and_comments()?;
            if s.curr_byte()? != b',' { break }
            s.advance(1);
            s.skip_spaces_and_comments()?;
        }

        s.skip_spaces_and_comments()?;
        s.consume_byte(b'{')?;

        let start = s.pos();
        crate::consume_until_block_end(s);

        let sheet = StyleSheet::parse(s.slice_range(start, s.pos()));

        Ok(Self {
            conditions,
            contents: sheet
        })
    }
}


/// an @container rule condition
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerCondition<'a> {
    /// the condition is just the name
    NameOnly(&'a str),
    /// the condition is just the query
    QueryOnly(ContainerQuery<'a>),
    /// the condition has a name and a query
    NameAndQuery {
        /// the name of the container
        name: &'a str,
        /// the query
        query: ContainerQuery<'a>,
    }
}
impl<'a> ContainerCondition<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        let text_pos = s.gen_text_pos();
        let mut name = None;
        let mut query = None;

        if let Some(q) = ContainerQuery::try_consume(s)? {
            query = Some(q);
        } else {
            name = Some(s.consume_ident()?);
            if let Some(q) = ContainerQuery::try_consume(s)? {
                query = Some(q);
            }
        }

        match (name, query) {
            (Some(name), None) => Ok(Self::NameOnly(name)),
            (None, Some(query)) => Ok(Self::QueryOnly(query)),
            (Some(name), Some(query)) => Ok(Self::NameAndQuery { name, query }),
            (None, None) => Err(Error::InvalidValue(text_pos))
        }
    }
}


/// a container query
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerQuery<'a> {
    /// the preceeding query should be negated
    Not(ContainerQueryInParens<'a>),
    /// a list of queries
    List {
        /// the first query
        first: ContainerQueryInParens<'a>,
        /// any following queryies 
        rest: Vec<ContainerQueryAndOr<'a>>
    }
}
impl<'a> ContainerQuery<'a> {
    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        s.skip_spaces_and_comments()?;

        let pos = s.pos();
        if s.curr_byte()? == b'(' || is_keyword(s, "not") {
            Ok(Some(Self::consume(s)?))
        } else {
            // check for "function"
            // TODO: there's probably a better way to do this check
            let _ident = s.consume_ident_special()?;
            if s.curr_byte()? == b'(' {
                s.reset_pos(pos);
                Ok(Some(Self::consume(s)?))
            } else {
                s.reset_pos(pos);
                Ok(None)
            }
        }
    }

    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        if is_keyword(s, "not") {
            Ok(Self::Not(ContainerQueryInParens::consume(s)?))
        } else {
            Ok(Self::List {
                first: ContainerQueryInParens::consume(s)?,
                rest: ContainerQueryAndOr::consume_many(s)?,
            })
        }
    }
}

/// a container query prefixed with either 'and' or 'or'
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerQueryAndOr<'a> {
    /// a query prefixed with 'and'
    And(ContainerQueryInParens<'a>),

    /// a query prefixed with 'or'
    Or(ContainerQueryInParens<'a>)
}
impl<'a> ContainerQueryAndOr<'a> {
    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        s.skip_spaces_and_comments()?;
        if is_keyword(s, "and") {
            s.advance(3);
            Ok(Some(Self::And(ContainerQueryInParens::consume(s)?)))
        } else if is_keyword(s, "or") {
            s.advance(2);
            Ok(Some(Self::Or(ContainerQueryInParens::consume(s)?)))
        } else {
            Ok(None)
        }
    }

    fn consume_many(s: &mut Stream<'a>) -> Result<Vec<Self>, Error> {
        let mut list = Vec::new();
        while let Some(a) = Self::try_consume(s)? {
            list.push(a);
        }
        Ok(list)
    }
}

/// A container query inner value
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerQueryInParens<'a> {
    /// Another query
    Query(Box<ContainerQuery<'a>>),
    /// A feature
    Feature(Feature<'a>),
    /// A "function"
    Function(ContainerFunction<'a>),
}
impl<'a> ContainerQueryInParens<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        if s.curr_byte()? == b'(' {
            // query or feature
            s.advance(1);
            s.skip_spaces_and_comments()?;

            if is_keyword(s, "not") || s.curr_byte()? == b'(' {
                let query = ContainerQuery::consume(s)?;
                s.skip_spaces_and_comments()?;
                s.consume_byte(b')')?;
                Ok(Self::Query(Box::new(query)))
            } else {
                let feature = Feature::consume(s)?;
                s.skip_spaces_and_comments()?;
                s.consume_byte(b')')?;
                Ok(Self::Feature(feature))
            }
        } else {
            // function
            Ok(Self::Function(ContainerFunction::consume(s)?))
        }
    }
}

/// A field should be compared to a value
#[derive(Clone, Debug, PartialEq)]
pub struct Feature<'a> {
    /// The field to be compared
    pub key: &'a str,
    /// What comparison should be made
    pub comparison: Comparison<'a>,

    /// The value the field should be compared against
    pub value: &'a str,
}
impl<'a> Feature<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        let key = s.consume_ident()?.trim();
        s.skip_spaces_and_comments()?;
        let comparison = Comparison::consume(s)?;
        s.skip_spaces_and_comments()?;
        let value = crate::consume_value(s)?;
        Ok(Self {
            key, 
            comparison,
            value
        })
    }
}


/// TODO: rename? again, not really a "function"
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerFunction<'a> {
    /// the name of the "function", ie "style"
    pub name: &'a str,

    /// the query
    pub query: FunctionQuery<'a>
}
impl<'a> ContainerFunction<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        Ok(Self {
            name: s.consume_ident_special()?.trim(),
            query: FunctionQuery::consume(s)?,
        })
    }
}


/// A function query
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionQuery<'a> {
    /// The following should be negated
    Not(FunctionInParens<'a>),
    /// A list of functions
    List {
        /// The first in the list
        first: FunctionInParens<'a>,
        /// The following to be compared as well
        rest: Vec<FunctionAndOr<'a>>
    }
}
impl<'a> FunctionQuery<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        if is_keyword(s, "not") {
            Ok(Self::Not(FunctionInParens::consume(s)?))
        } else {
            Ok(Self::List {
                first: FunctionInParens::consume(s)?,
                rest: FunctionAndOr::consume_many(s)?,
            })
        }
    }
}


/// An inner function query
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionInParens<'a> {
    /// Another query
    Query(Box<ContainerFunction<'a>>),
    /// A feature
    Feature(Feature<'a>),
}
impl<'a> FunctionInParens<'a> {
    fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        s.consume_byte(b'(')?;

        let ident = s.consume_ident_special()?;
        s.skip_spaces_and_comments()?;
        let r = if s.curr_byte()? == b':' {
            s.advance(1);
            let value = s.consume_bytes(|b| b != b')').trim();
            Self::Feature(Feature { key: ident, comparison: Comparison::Equal, value })
        } else {
            s.consume_byte(b'(')?;
            let query = FunctionQuery::consume(s)?;
            s.skip_spaces_and_comments()?;
            s.consume_byte(b')')?;
            Self::Query(Box::new(ContainerFunction {
                name: ident,
                query,
            }))
        };
        
        s.skip_spaces_and_comments()?;
        s.consume_byte(b')')?;

        Ok(r)
    }
}

/// A function preceeded by 'and' or 'or'
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionAndOr<'a> {
    /// A function preceeded by 'and'
    And(FunctionInParens<'a>),
    /// A function preceeded by 'or'
    Or(FunctionInParens<'a>),
}
impl<'a> FunctionAndOr<'a> {
    fn try_consume(s: &mut Stream<'a>) -> Result<Option<Self>, Error> {
        s.skip_spaces_and_comments()?;
        if is_keyword(s, "and") {
            s.advance(3);
            Ok(Some(Self::And(FunctionInParens::consume(s)?)))
        } else if is_keyword(s, "or") {
            s.advance(2);
            Ok(Some(Self::Or(FunctionInParens::consume(s)?)))
        } else {
            Ok(None)
        }
    }

    fn consume_many(s: &mut Stream<'a>) -> Result<Vec<Self>, Error> {
        let mut list = Vec::new();
        while let Some(a) = Self::try_consume(s)? {
            list.push(a);
        }
        Ok(list)
    }
}
