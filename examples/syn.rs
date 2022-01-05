use rocdoc::extract::extract_items;
use std::fs::read_to_string;

const FNAME: &str = "src/extract.rs";

fn main() {
    let contents = read_to_string(FNAME).expect("wrong file path");
    let items = extract_items("extract", &contents).unwrap();

    println!("{}", items.render_all());
}
