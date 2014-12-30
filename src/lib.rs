mod tokenizer;

pub struct Wkt;

impl Wkt {
}

fn from_reader(reader: &mut Reader) -> Wkt {
    match reader.read_to_string() {
        Ok(string) => from_string(string),
        Err(err) => panic!(err),
    }
}

fn from_string(string: String) -> Wkt {
    Wkt
}

#[test]
fn it_works() {
}
