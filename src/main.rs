use std::path::Path;

use parser::CellParser;
use raw_spreadsheet::RawSpreadSheet;
mod raw_spreadsheet;
mod parser;

fn main() {
    let input = Path::new("csv").join("sum.csv");
    let raw_cells = RawSpreadSheet::new(input);
    println!("{}", &raw_cells);
    let parsed_cells = CellParser::parse_raw(raw_cells);
    println!("{:?}", parsed_cells);
}

// TODO represent cells in a single data structure where each cell can have 3 states : raw, parsed, computed