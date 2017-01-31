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
use FromTokens;


#[derive(Default)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub m: Option<f64>,
}

impl FromTokens for Coord {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let x = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected a number for the X coordinate"),
        };
        let y = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected a number for the Y coordinate"),
        };
        Ok(Coord {
            x: x,
            y: y,
            z: None,
            m: None,
        })
    }
}
