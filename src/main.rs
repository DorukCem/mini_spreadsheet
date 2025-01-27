pub mod common_types;
pub mod common_functions;

use gui::GUI;
use spreadsheet::SpreadSheet;

mod gui;
mod renderer;
mod spreadsheet;

#[macroquad::main("Spredsheet")]
async fn main() {
    let spread_sheet = SpreadSheet::default();
    let mut gui = GUI::new(spread_sheet).await;
    gui.start().await;
}