use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

use crate::{common_types::Index, spreadsheet::SpreadSheet};

// Window configuration
const INITIAL_WINDOW_WIDTH: f32 = 1200.0;
const INITIAL_WINDOW_HEIGHT: f32 = 900.0;

// Grid configuration
const GRID_ROWS: usize = 10;
const GRID_COLS: usize = 10;

// Editor configuration
const EDITOR_HEIGHT: f32 = 80.0;
const EDITOR_TOP_MARGIN: f32 = 0.0;

// Cell styling
const CELL_FONT_SIZE: u16 = 12;
const SELECTED_CELL_BORDER_WIDTH: f32 = 3.0;
const NORMAL_CELL_BORDER_WIDTH: f32 = 1.0;

// Colors
const BACKGROUND_COLOR: Color = BLACK;
const GRID_BACKGROUND_COLOR: Color = WHITE;
const SELECTED_CELL_BORDER_COLOR: Color = ORANGE;
const NORMAL_CELL_BORDER_COLOR: Color = BLACK;
const CELL_TEXT_COLOR: Color = BLACK;

pub struct GUI {
    selected_cell: Option<Index>,
    editor_content: String,
    font: Font,
    spread_sheet: SpreadSheet,
}

impl GUI {
    pub async fn new(spread_sheet: SpreadSheet) -> Self {
        let font = load_ttf_font("fonts/jetbrains-mono-font/JetbrainsMonoRegular-RpvmM.ttf")
            .await
            .unwrap();
        Self {
            selected_cell: None,
            font,
            editor_content: String::new(),
            spread_sheet,
        }
    }

    pub async fn start(&mut self) {
        request_new_screen_size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);

        loop {
            clear_background(BACKGROUND_COLOR);

            // Handle egui input
            self.draw_editor();
            self.draw_cells(
                (0.0, EDITOR_HEIGHT + EDITOR_TOP_MARGIN),
                (screen_width(), screen_height()),
            );

            next_frame().await
        }
    }

    fn draw_editor(&mut self) {
        // Create an egui text input widget

        root_ui().window(
            hash!(),
            vec2(0.0, EDITOR_TOP_MARGIN),
            vec2(screen_width(), EDITOR_HEIGHT),
            |ui| {
                ui.input_text(hash!(), "", &mut self.editor_content);

                // Add a submit button
                if ui.button(None, "Submit") || is_key_pressed(KeyCode::Enter) {
                    self.commit_editor();
                    self.selected_cell = None;
                    self.editor_content.clear();
                }
            },
        );
    }

    fn draw_cells(&mut self, start: (f32, f32), end: (f32, f32)) {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        let cell_height = (end_y - start_y) / GRID_ROWS as f32;
        let cell_width = (end_x - start_x) / GRID_COLS as f32;

        // Handle if mouse clicked
        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            if is_point_in_rect((x, y), start, end) {
                let col = ((x - start_x) / cell_width) as i32;
                let row = ((y - start_y) / cell_height) as i32;
                self.change_selected_cell(Index {
                    x: col.try_into().expect("Got negative idx from click"),
                    y: row.try_into().expect("Got negative idx from click"),
                });
            }
        }

        // Draw background
        draw_rectangle(
            start_x,
            start_y,
            end_x - start_x,
            end_y - start_y,
            GRID_BACKGROUND_COLOR,
        );

        // Draw all cells in the grid
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let cell_start_x = start_x + col as f32 * cell_width;
                let cell_start_y = start_y + row as f32 * cell_height;

                self.draw_cell(
                    Index { x: col, y: row },
                    (cell_start_x, cell_start_y),
                    (cell_width, cell_height),
                );
            }
        }
    }

    fn draw_cell(&self, index: Index, start: (f32, f32), dimensions: (f32, f32)) {
        let (start_x, start_y) = start;
        let (width, height) = dimensions;

        let center_x = start_x + width / 2.0;
        let center_y = start_y + height / 2.0;

        let (border_width, border_color) = if Some(index) == self.selected_cell {
            (SELECTED_CELL_BORDER_WIDTH, SELECTED_CELL_BORDER_COLOR)
        } else {
            (NORMAL_CELL_BORDER_WIDTH, NORMAL_CELL_BORDER_COLOR)
        };

        draw_rectangle_lines(
            start_x,
            start_y,
            width,
            height,
            border_width,
            border_color,
        );

        let text = if Some(index) == self.selected_cell {
            &self.editor_content
        } else {
            &self.spread_sheet.get_text(index)
        };

        if !text.is_empty() {
            draw_text_ex(
                text,
                center_x,
                center_y,
                TextParams {
                    font: Some(&self.font),
                    font_size: CELL_FONT_SIZE,
                    font_scale: 1.0,
                    font_scale_aspect: 1.0,
                    rotation: 0.0,
                    color: CELL_TEXT_COLOR,
                },
            );
        }
    }

    fn commit_editor(&mut self) {
        if let Some(idx) = self.selected_cell {
            let previous_content = self.spread_sheet.get_raw(&idx).unwrap_or_default();
            let new_content = self.editor_content.trim().to_string();

            match (previous_content, new_content.as_str()) {
                (prev, new) if prev == new => (),
                ("", "") => (),
                ("", _added_content) => self.spread_sheet.add_cell_and_compute(idx, new_content),
                (_deleted_content, "") => self.spread_sheet.remove_cell(idx),
                (_mutated_from, _mutated_to) => self.spread_sheet.mutate_cell(idx, new_content),
            }
        }
    }

    fn change_selected_cell(&mut self, idx: Index) {
        if self.selected_cell == Some(idx) {
            return;
        }

        self.commit_editor();
        self.editor_content = self
            .spread_sheet
            .get_raw(&idx)
            .unwrap_or_default()
            .to_owned();
        self.selected_cell = Some(idx);
    }
}

fn is_point_in_rect<T: std::cmp::PartialOrd>(
    point: (T, T),
    rect_start: (T, T),
    rect_end: (T, T),
) -> bool {
    rect_start.0 <= point.0
        && point.0 <= rect_end.0
        && rect_start.1 <= point.1
        && point.1 <= rect_end.1
}
