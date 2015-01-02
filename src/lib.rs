use std::ascii::AsciiExt;

use tokenizer::{Token, Tokenizer};
use types::point::Point;

mod tokenizer;
mod types;


pub struct Wkt {
    items: Vec<Point>
}

impl Wkt {
    fn new() -> Self {
        Wkt {items: vec![]}
    }

    fn add_point(&mut self, point: Point) {
        self.items.push(point);
    }

    fn from_reader(reader: &mut Reader) -> Result<Self, &'static str> {
        match reader.read_to_string() {
            Ok(string) => Wkt::from_str(string.as_slice()),
            Err(err) => Err(err.desc),
        }
    }

    fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = tokenizer::tokenize(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokenizer) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        let mut tokens = tokens.peekable();
        match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                let uppercased = word.to_ascii_uppercase();
                let constructor = match uppercased.as_slice() {
                    "POINT" => Point::from_tokens,
                    _ => return Err("Invalid type encountered"),
                };
                match tokens.next() {
                    Some(Token::ParenOpen) => (),
                    _ => return Err("Missing open parenthesis for type"),
                };
                match constructor(&mut tokens) {
                    Ok(point) => wkt.add_point(point),
                    Err(s) => return Err(s),
                }
                match tokens.next() {
                    Some(Token::ParenClose) => (),
                    _ => return Err("Missing closing parenthesis for type"),
                };
                Ok(wkt)
            },
            None => Ok(wkt),
            _ => Err("Invalid WKT format"),
        }
    }
}


#[test]
fn empty_string() {
    let wkt = Wkt::from_str("").ok().unwrap();
    assert_eq!(0, wkt.items.len());
}


#[test]
fn basic_point() {
    let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
    assert_eq!(1, wkt.items.len());
    let point = wkt.items.pop().unwrap();
    assert_eq!(10.0, point.coord.x);
    assert_eq!(-20.0, point.coord.y);
    assert_eq!(None, point.coord.z);
    assert_eq!(None, point.coord.m);
}


#[test]
fn invalid_points() {
    Wkt::from_str("POINT ()").err().unwrap();
    Wkt::from_str("POINT (10)").err().unwrap();
    Wkt::from_str("POINT 10").err().unwrap();
    Wkt::from_str("POINT (10 -20 40)").err().unwrap();
}
