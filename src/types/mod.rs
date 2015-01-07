use tokenizer::{PeekableTokens, Token};

pub mod coord;
pub mod point;
pub mod linestring;


pub trait FromTokens: Sized {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => return Err("Missing open parenthesis for type"),
        };
        let result = FromTokens::from_tokens(tokens);
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for type"),
        };
        result
    }
}
