use tokenizer::PeekableTokens;

pub mod coord;
pub mod point;
pub mod linestring;


pub trait FromTokens {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str>;
}
