use crate::{
    Declaration, Error,
    Rule, Stream,
};
use alloc::vec::Vec;
use super::{
    container::Container,
    import::Import,
    media::Media,
    supports::Supports,
};

/// An @ rule
#[derive(Clone, Debug, PartialEq)]
pub enum AtRule<'a> {
    /// An @container rule
    Container(Container<'a>),

    /// An @font-face rule
    FontFace(Vec<Declaration<'a>>),

    /// An @import rule.
    Import(Import<'a>),

    /// An @keyframes rule.
    Keyframes {
        /// Animation name
        name: &'a str,

        /// Frames in the animation
        frames: Vec<KeyFrame<'a>>,
    },

    /// An @layer rule
    Layer(LayerType<'a>),

    /// An @media rule
    Media(Media<'a>),

    /// An @namespace rule
    Namespace {
        /// The name of the namespace, None if default
        name: Option<&'a str>,

        /// The source of the namespace
        value: &'a str,
    },

    /// An @supports rule
    Supports(Supports<'a>),

    /// Some other, unparsed at rule
    Other {
        /// The identity of the @rule
        /// 
        /// ie. "keyframes" in "@keyframes anim-name {...}"
        ident: &'a str,

        /// Any data before a block begins
        /// 
        /// ie the "anim-name" in "@keyframes anim-name {...}"
        pre_block: &'a str,

        /// The body of the @ rule if it was a block
        /// 
        /// ie. "..." in "@keyframes anim-name {...}"
        block: &'a str,
    },
}
impl<'a> AtRule<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        let ident = s.consume_ident()?;

        match ident {
            "container" => Ok(Self::Container(Container::consume(s)?)),

            "font-face" => {
                s.skip_spaces();
                s.try_consume_byte(b'{');
                let declarations = crate::consume_declarations(s)?;
                s.skip_spaces();
                s.try_consume_byte(b'}');
                Ok(Self::FontFace(declarations))
            }

            "import" => Ok(Self::Import(Import::consume(s)?)),

            "keyframes" => {
                s.skip_spaces_and_comments()?;
                let name = s.consume_ident()?;

                s.skip_spaces_and_comments()?;
                s.consume_byte(b'{')?;

                let mut frames = Vec::new();
                loop {
                    // read the keyframe ident
                    s.skip_spaces_and_comments()?;
                    let key = s.consume_ident_special()?.trim();

                    // read the declarations
                    s.skip_spaces_and_comments()?;
                    s.consume_byte(b'{')?;
                    let declarations = crate::consume_declarations(s)?;
                    frames.push(KeyFrame { key, declarations });
                    s.try_consume_byte(b'}');
                    s.skip_spaces_and_comments()?;

                    // if done with block, exit
                    if s.curr_byte()? == b'}' {
                        s.advance(1);
                        break;
                    }
                }

                Ok(Self::Keyframes { name, frames })
            }

            "layer" => {
                s.skip_spaces_and_comments()?;
                let value = s.consume_ident()?;
                s.skip_spaces_and_comments()?;

                match s.curr_byte()? {
                    // layer def
                    b'{' => {
                        s.advance(1);
                        s.skip_spaces_and_comments()?;
                        
                        let name = value.trim();
                        let mut rules = Vec::new();
                        crate::consume_rule_set(s, &mut rules)?;
                        s.skip_spaces_and_comments()?;
                        s.try_consume_byte(b'}');
                        s.skip_spaces_and_comments()?;

                        let name = Some(name).filter(|n| !n.is_empty());
                        Ok(Self::Layer(LayerType::Block { name, rules }))
                    }
                    // list
                    b',' => {
                        let mut names = alloc::vec![value.trim()];
                        s.skip_spaces_and_comments()?;

                        // layer list (probably)
                        while s.curr_byte()? == b',' {
                            s.advance(1);
                            s.skip_spaces_and_comments()?;
                            names.push(s.consume_ident()?.trim());
                            s.skip_spaces_and_comments()?;
                        }
                        s.consume_byte(b';')?;

                        Ok(Self::Layer(LayerType::Statement(names)))
                    }

                    // single item list
                    b';' => {
                        Ok(Self::Layer(LayerType::Statement(alloc::vec![value])))
                    }

                    _ => Err(Error::InvalidIdent(s.gen_text_pos())),
                }
            }

            "media" => {
                Ok(Self::Media(Media::consume(s)?))
            }

            // TODO: this is inconsistent
            "namespace" => {
                s.skip_spaces_and_comments()?;
                if s.curr_byte()? == b'"' {
                    let value = s.consume_string()?;
                    s.skip_spaces_and_comments()?;
                    s.consume_byte(b';')?;

                    Ok(Self::Namespace { name: None, value })
                } else {
                    let start = s.pos();
                    let ident2 = s.consume_ident()?;

                    s.skip_spaces_and_comments()?;
                    if s.curr_byte()? == b'(' {
                        s.advance(1);
                        s.skip_spaces_and_comments()?;
                        
                        if s.curr_byte()? == b'"' {
                            s.skip_spaces_and_comments()?;
                            s.consume_byte(b';')?;
                        } else {
                            s.consume_ident()?;
                        }
                        s.skip_spaces_and_comments()?;
                        s.consume_byte(b')')?;

                        let value = s.slice_range(start, s.pos());
                        s.skip_spaces_and_comments()?;
                        s.consume_byte(b';')?;
                        s.skip_spaces_and_comments()?;
                        
                        Ok(Self::Namespace { name: None, value })
                    } else {
                        s.skip_spaces_and_comments()?;
                        let value = crate::consume_value(s)?;
                        s.skip_spaces_and_comments()?;
                        s.consume_byte(b';')?;

                        Ok(Self::Namespace { name: Some(ident2), value })
                    }
                }
            }

            "supports" => Ok(Self::Supports(Supports::consume(s)?)),

            _ => {
                let pre_block = s.consume_bytes(|c| c != b';' && c != b'{').trim();
                s.skip_spaces_and_comments()?;

                let block = if s.curr_byte()? == b'{' {
                    crate::read_block(s, false).trim()
                } else {
                    ""
                };

                Ok(Self::Other {
                    ident,
                    pre_block,
                    block,
                })
            }
        }
    }
}


/// An @keyframes entry
#[derive(Clone, Debug, PartialEq)]
pub struct KeyFrame<'a> {
    /// The key in the keyframe, ie "100%"
    pub key: &'a str,

    /// the list of declarations inside the body
    pub declarations: Vec<Declaration<'a>>,
}

/// What type of layer entry is this?
#[derive(Clone, Debug, PartialEq)]
pub enum LayerType<'a> {
    /// @layer layer1, layer2
    Statement(Vec<&'a str>),

    /// @layer name { ..rules.. }
    Block {
        /// The name of the layer.
        /// 
        /// None if anonymous
        name: Option<&'a str>,

        /// The list of rules in the @layer body
        rules: Vec<Rule<'a>>,
    },
}

/// A comparison operator
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Comparison<'a> {
    /// == 
    Equal,

    /// !=
    NotEqual,

    /// <
    Less,

    /// >
    Greater,

    /// <=
    LessEq,

    /// =>
    GreaterEq,

    /// Some other, likely custom operator
    Other(&'a str),
}
impl<'a> Comparison<'a> {
    pub(crate) fn consume(s: &mut Stream<'a>) -> Result<Self, Error> {
        s.skip_spaces_and_comments()?;
        let bytes = s.consume_bytes(|b| b != b' ').trim();
        s.advance(1);

        Ok(match bytes {
            "=" | "==" => Self::Equal,
            ">" => Self::Greater,
            "<" => Self::Less,
            ">=" => Self::GreaterEq,
            "<=" => Self::LessEq,
            "!=" => Self::NotEqual,
            _ => Self::Other(bytes),
        })

    }
}


/// A combinator/modifier
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Combinator<'a> {
    /// The next thing should not be true
    Not,

    /// The previous or the next should be true
    Or,

    /// The previous and the next should be true
    And,

    /// Some other, likely custom operator
    Other(&'a str),
}

pub(crate) fn is_keyword(s: &mut Stream<'_>, keyword: &str) -> bool {
    let tail = s.slice_tail();
    // if the tail doesnt start with our keyword then its obviously not the keyword 
    if !tail.starts_with(keyword) { 
        return false 
    }

    // if the tail and the keyword are the same length, then all thats left in the stream is the keyword
    if tail.len() == keyword.len() {
        return true;
    }

    let c = tail.chars().nth(keyword.len()).unwrap();
    match c as u8 {
        // if c is a whitespace character then the keyword exists
        b' ' | b'\t' | b'\n' | b'\r' | b'\x0C' => true,
        // otherwise the keyword is part of a larger term (ie if keyword is "not", the term might be "nothing")
        _ => false,
    }
}

