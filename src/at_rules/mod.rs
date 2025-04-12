
/// A module with all available @rules
pub mod at_rule;


/// Everything needed for @container rules
/// 
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/@container#formal_syntax>
pub mod container;


/// Everything needed for @import rules
/// 
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/@import>
pub mod import;


/// everything needed for @media rules
/// 
/// FIXME: find where i got the media parsing stuff from lol
pub mod media;


/// Everything needed for an @supports rule
/// 
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/@supports#formal_syntax>
pub mod supports;