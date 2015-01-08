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
use types::FromTokens;
use types::coord::Coord;
use WktItem;


pub struct LineString {
    pub coords: Vec<Coord>
}

impl LineString {
    pub fn as_item(self) -> WktItem {
        WktItem::LineString(self)
    }
}

impl FromTokens for LineString {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let mut coords = Vec::new();

        coords.push(match Coord::from_tokens(tokens) {
            Ok(c) => c,
            Err(s) => return Err(s),
        });

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next();  // throw away comma

            coords.push(match Coord::from_tokens(tokens) {
                Ok(c) => c,
                Err(s) => return Err(s),
            });
        }

        Ok(LineString {coords: coords})
    }
}
