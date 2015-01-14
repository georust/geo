// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use tokenizer::{PeekableTokens, Token};

pub mod coord;
pub mod point;
pub mod polygon;
pub mod linestring;
pub mod multipoint;


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

    fn comma_many<F>(f: F, tokens: &mut PeekableTokens) -> Result<Vec<Self>, &'static str>
            where F: Fn(&mut PeekableTokens) -> Result<Self, &'static str> {
        let mut items = Vec::new();

        match f(tokens) {
            Ok(i) => items.push(i),
            Err(s) => return Err(s),
        };

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next();  // throw away comma

            match f(tokens) {
                Ok(i) => items.push(i),
                Err(s) => return Err(s),
            };
        }

        Ok(items)
    }
}
