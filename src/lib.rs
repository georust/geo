use std::ascii::AsciiExt;
use tokenizer::{Token, Tokenizer};

mod tokenizer;


pub struct Point {
    x: f64,
    y: f64,
    z: Option<f64>,
    m: Option<f64>,
}

impl Point {
    fn from_tokens(tokens: &mut Tokenizer) -> Self {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => panic!("FIXME"),
        };
        let x = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => panic!("FIXME"),
        };
        let y = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => panic!("FIXME"),
        };
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => panic!("FIXME"),
        };
        Point {x: x, y: y, z: None, m: None}
    }
}


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

    fn from_reader(reader: &mut Reader) -> Self {
        match reader.read_to_string() {
            Ok(string) => Wkt::from_string(string),
            Err(err) => panic!(err),
        }
    }

    fn from_string(string: String) -> Self {
        let ref mut tokens = tokenizer::tokenize(string.as_slice());
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: &mut Tokenizer) -> Self {
        let mut wkt = Wkt::new();
        match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    panic!("Encountered non-ascii word");
                }
                let uppercased = word.to_ascii_uppercase();
                match uppercased.as_slice() {
                    "POINT" => {
                        wkt.add_point(Point::from_tokens(tokens));
                        wkt
                    },
                    _ => wkt,
                }
            },
            // None
            _ => panic!("Invalid WKT format"),
        }
    }
}


#[test]
fn basic_point() {
    let mut wkt = Wkt::from_string("POINT (10 -20)".to_string());
    assert_eq!(1, wkt.items.len());
    let point = wkt.items.pop().unwrap();
    assert_eq!(10.0, point.x);
    assert_eq!(-20.0, point.y);
    assert_eq!(None, point.z);
    assert_eq!(None, point.m);
}
