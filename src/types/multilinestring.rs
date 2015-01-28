// Copyright 2015 The GeoRust Developers
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

use tokenizer::PeekableTokens;
use types::FromTokens;
use types::linestring::LineString;
use Geometry;


#[derive(Default)]
pub struct MultiLineString {
    pub lines: Vec<LineString>
}

impl MultiLineString {
    pub fn as_item(self) -> Geometry {
        Geometry::MultiLineString(self)
    }
}

impl FromTokens for MultiLineString {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(<LineString as FromTokens>::from_tokens_with_parens, tokens);
        result.map(|vec| MultiLineString {lines: vec})
    }
}


#[cfg(test)]
mod tests {
    use {Wkt, Geometry};

    #[test]
    fn basic_multilinestring() {
        let mut wkt = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let multilinestring = match wkt.items.pop().unwrap() {
            Geometry::MultiLineString(multilinestring) => multilinestring,
            _ => unreachable!(),
        };
        assert_eq!(2, multilinestring.lines.len());
    }
}
