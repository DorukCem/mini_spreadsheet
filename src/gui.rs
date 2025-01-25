use macroquad::ui::widgets::InputText;
use macroquad::ui::{hash, root_ui, Skin};
use macroquad::{prelude::*, text};

use crate::common_types::{ComputeError, Value};
use crate::{common_types::Index, spreadsheet::SpreadSheet};

// Window configuration
const INITIAL_WINDOW_WIDTH: f32 = 1200.0;
const INITIAL_WINDOW_HEIGHT: f32 = 900.0;

// Grid configuration
const GRID_ROWS: usize = 20;
const GRID_COLS: usize = 6;

// Editor configuration
const EDITOR_HEIGHT: f32 = 24.0;
const EDITOR_TOP_MARGIN: f32 = 0.0;
const EDITOR_PADDING: f32 = 20.0;
const EDITOR_WINDOW_HEIGHT: f32 = EDITOR_HEIGHT + EDITOR_PADDING * 2.0;

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

// Labels
const ROW_LABEL_WIDTH: f32 = 40.0;
const COL_LABEL_HEIGHT: f32 = 30.0;
const LABEL_FONT_SIZE: u16 = 10;
const LABEL_TEXT_COLOR: Color = DARKGRAY;
const LABEL_BORDER_COLOR: Color = DARKGRAY;
const SELECTED_LABEL_BACKGROUND: Color = SKYBLUE;

pub struct GUI {
    selected_cell: Option<Index>,
    editor_content: String,
    regular_font: Font,
    bold_font: Font,
    spread_sheet: SpreadSheet,
    editor_skin: Skin,
}

impl GUI {
    pub async fn new(spread_sheet: SpreadSheet) -> Self {
        let regular_font = load_ttf_font("fonts/ttf/Hack-Regular.ttf").await.unwrap();

        let bold_font = load_ttf_font("fonts/ttf/Hack-Bold.ttf").await.unwrap();

        // Create a minimal style for the editor
        let editor_skin = {
            let editbox_style = root_ui()
                .style_builder()
                .color_selected(Color::from_rgba(200, 200, 255, 255)) // Light blue selection
                .with_font(&regular_font)
                .unwrap()
                .build();

            let window_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(240, 240, 240, 255)) // Light gray background
                .with_font(&regular_font)
                .unwrap()
                .build();

            Skin {
                editbox_style,
                window_style,
                ..root_ui().default_skin()
            }
        };

        Self {
            selected_cell: None,
            regular_font,
            editor_content: String::new(),
            spread_sheet,
            bold_font,
            editor_skin,
        }
    }

    pub async fn start(&mut self) {
        request_new_screen_size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);

        loop {
            clear_background(BACKGROUND_COLOR);

            self.draw_editor();
            self.draw_cells(
                (0.0, EDITOR_WINDOW_HEIGHT),
                (screen_width(), screen_height()),
            );

            next_frame().await
        }
    }

    fn draw_editor(&mut self) {
        // Push our custom skin before drawing the editor
        root_ui().push_skin(&self.editor_skin);

        let window_id = hash!();
        root_ui().window(
            window_id,
            vec2(0.0, EDITOR_TOP_MARGIN),
            vec2(screen_width(), EDITOR_WINDOW_HEIGHT),
            |ui| {
                let input_text_id = hash!();
                InputText::new(input_text_id)
                    .label("")
                    .position(vec2(ROW_LABEL_WIDTH, EDITOR_TOP_MARGIN + EDITOR_PADDING))
                    .size(vec2(screen_width() - ROW_LABEL_WIDTH * 2.0, EDITOR_HEIGHT))
                    .ui(ui, &mut self.editor_content);

                // Focus the editor when a cell is selected
                if self.selected_cell.is_some() {
                    ui.set_input_focus(input_text_id);
                } else {
                    ui.set_input_focus(hash!());
                }

                if is_key_pressed(KeyCode::Enter) {
                    self.commit_editor();
                    self.selected_cell = None;
                    self.editor_content.clear();
                }
            },
        );

        // Pop the skin after we're done
        root_ui().pop_skin();
    }

    fn draw_cells(&mut self, start: (f32, f32), end: (f32, f32)) {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        let grid_height = end_y - start_y - COL_LABEL_HEIGHT;
        let grid_width = end_x - start_x - ROW_LABEL_WIDTH;

        let cell_height = grid_height / GRID_ROWS as f32;
        let cell_width = grid_width / GRID_COLS as f32;

        // Handle if mouse clicked
        let mut hovered: Option<Index> = None;
        let (x, y) = mouse_position();
        if is_point_in_rect((x, y), start, end) {
            let col = ((x - start_x - ROW_LABEL_WIDTH) / cell_width) as i32;
            let row = ((y - start_y - COL_LABEL_HEIGHT) / cell_height) as i32;
            let x_idx = col.try_into().expect("Got negative idx from click");
            let y_idx = row.try_into().expect("Got negative idx from click");

            hovered = Some(Index { x: x_idx, y: y_idx });

            if is_mouse_button_pressed(MouseButton::Left) {
                if is_key_down(KeyCode::LeftControl) {
                    if self.selected_cell.is_some()
                        && &Some('=') == &self.editor_content.chars().nth(0)
                    {
                        self.editor_content.push_str(&format!(
                            "{}{}",
                            column_idx_to_string(x_idx),
                            y_idx + 1
                        ))
                    }
                } else {
                    self.change_selected_cell(Index { x: x_idx, y: y_idx });
                }
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

        // Draw the column labels
        for col in 0..GRID_COLS {
            let label_start_x = start_x + col as f32 * cell_width + ROW_LABEL_WIDTH;
            let label_start_y = start_y;
            self.draw_error_label(
                col,
                false, // Indicating column
                (label_start_x, label_start_y),
                (cell_width, COL_LABEL_HEIGHT),
            );
        }

        // Draw the row labels
        for row in 0..GRID_ROWS {
            let label_start_x = start_x;
            let label_start_y = start_y + row as f32 * cell_height + COL_LABEL_HEIGHT;
            self.draw_error_label(
                row,
                true, // Indicating row
                (label_start_x, label_start_y),
                (ROW_LABEL_WIDTH, cell_height),
            );
        }

        // Draw all cells in the grid
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let cell_start_x = start_x + col as f32 * cell_width + ROW_LABEL_WIDTH;
                let cell_start_y = start_y + row as f32 * cell_height + COL_LABEL_HEIGHT;

                // Adjust the height of the last row to account for any floating-point error
                let adjusted_cell_height = if row == GRID_ROWS - 1 {
                    grid_height - (row as f32 * cell_height)
                } else {
                    cell_height
                };

                self.draw_cell(
                    Index { x: col, y: row },
                    (cell_start_x, cell_start_y),
                    (cell_width, adjusted_cell_height),
                );
            }
        }

        // Draw dialog box for hovered cell
        if let Some(idx) = hovered {
            let cell_end_x = start_x + idx.x as f32 * cell_width + ROW_LABEL_WIDTH + cell_width;
            let cell_end_y = start_y + idx.y as f32 * cell_height + COL_LABEL_HEIGHT;
            let dialog_pos = (cell_end_x, cell_end_y);
            self.draw_dialog(idx, dialog_pos, cell_width, cell_height);
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

        draw_rectangle_lines(start_x, start_y, width, height, border_width, border_color);

        let mut text: String = if Some(index) == self.selected_cell {
            self.editor_content.clone()
        } else {
            let computed = self.spread_sheet.get_computed(index);
            if let Some(Err(_)) = computed {
                let triangle_len = 10.;
                draw_triangle(
                    vec2(start_x + width, start_y),
                    vec2(start_x + width - triangle_len, start_y),
                    vec2(start_x + width, start_y + triangle_len),
                    RED,
                );
            }

            computed_to_text(computed)
        };

        if !text.is_empty() {
            let mut text_dimensions =
                measure_text(&text, Some(&self.regular_font), CELL_FONT_SIZE, 1.0);

            let mut is_oversize = false;
            let original = text.clone();
            if text_dimensions.width > width {
                is_oversize = true;
                for _ in 0.."...".len() {
                    text.pop();
                }
                let mut text_width = text_dimensions.width;
                while text_width > width {
                    if let None = text.pop() {
                        break;
                    }
                    text_width = measure_text(&text, Some(&self.regular_font), CELL_FONT_SIZE, 1.0)
                        .width
                        + measure_text("...", Some(&self.regular_font), CELL_FONT_SIZE, 1.0).width;
                }
                text.push_str("...");
                text_dimensions =
                    measure_text(&text, Some(&self.regular_font), CELL_FONT_SIZE, 1.0);
            }

            let text_x = center_x - text_dimensions.width / 2.0;
            let text_y = center_y + text_dimensions.height / 2.0; // Adjust y for baseline alignment

            draw_text_ex(
                &text,
                text_x,
                text_y,
                TextParams {
                    font: Some(&self.regular_font),
                    font_size: CELL_FONT_SIZE,
                    font_scale: 1.0,
                    font_scale_aspect: 1.0,
                    rotation: 0.0,
                    color: CELL_TEXT_COLOR,
                },
            );

            if is_oversize {
                if is_point_in_rect(mouse_position(), start, (start.0 + width, start.1 + height)) {
                    self.draw_oversize_label(original, center_x, center_y)
                }
            }
        }
    }

    fn draw_error_label(
        &self,
        idx: usize,
        is_row: bool,
        start: (f32, f32),
        dimensions: (f32, f32),
    ) {
        let (start_x, start_y) = start;
        let (width, height) = dimensions;
        let center_x = start_x + width / 2.0;
        let center_y = start_y + height / 2.0;

        let is_selected_label = {
            if let Some(selected) = self.selected_cell {
                if is_row {
                    selected.y == idx
                } else {
                    selected.x == idx
                }
            } else {
                false
            }
        };

        if is_selected_label {
            // Draw background
            draw_rectangle(start_x, start_y, width, height, SELECTED_LABEL_BACKGROUND);
        }

        draw_rectangle_lines(start_x, start_y, width, height, 1.0, LABEL_BORDER_COLOR);
        let text = if is_row {
            (idx + 1).to_string()
        } else {
            column_idx_to_string(idx)
        };
        let text_dimensions = measure_text(&text, Some(&self.regular_font), LABEL_FONT_SIZE, 1.0);

        let text_x = center_x - text_dimensions.width / 2.0;
        let text_y = center_y + text_dimensions.height / 2.0; // Adjust y for baseline alignment

        draw_text_ex(
            &text,
            text_x,
            text_y,
            TextParams {
                font: Some(if is_selected_label {
                    &self.bold_font
                } else {
                    &self.regular_font
                }),
                font_size: LABEL_FONT_SIZE,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: LABEL_TEXT_COLOR,
            },
        );
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

    fn draw_dialog(&self, idx: Index, pos: (f32, f32), cell_width: f32, cell_height: f32) {
        if let Some(err) = self.spread_sheet.get_error(idx) {
            let dialog_width: f32 = cell_width;
            let dialog_height: f32 = cell_height * 2.0;
            const DIALOG_FONT_SIZE: u16 = 14;

            let (dialog_x, dialog_y) = pos;

            // Draw dialog background
            draw_rectangle(
                dialog_x,
                dialog_y,
                dialog_width,
                dialog_height,
                GRID_BACKGROUND_COLOR,
            );
            draw_rectangle_lines(dialog_x, dialog_y, dialog_width, dialog_height, 4.0, RED);

            // Prepare dialog text
            let dialog_text = format!("Error: {}", err_to_info(err));

            let lines = split_into_lines(
                &dialog_text,
                &self.regular_font,
                DIALOG_FONT_SIZE,
                dialog_width - 10.0,
            );

            // Calculate vertical starting position for centering the text block
            let total_text_height = lines.len() as f32 * (DIALOG_FONT_SIZE as f32 + 4.0); // 4.0 for line spacing
            let mut text_y = dialog_y + (dialog_height - total_text_height) / 2.0;

            // Draw each line of text
            for line in lines {
                let text_dimensions =
                    measure_text(&line, Some(&self.bold_font), DIALOG_FONT_SIZE, 1.0);
                let text_x = dialog_x + (dialog_width - text_dimensions.width) / 2.0;

                draw_text_ex(
                    &line,
                    text_x,
                    text_y,
                    TextParams {
                        font: Some(&self.bold_font),
                        font_size: DIALOG_FONT_SIZE,
                        font_scale: 1.0,
                        font_scale_aspect: 1.0,
                        rotation: 0.0,
                        color: BLACK,
                    },
                );

                text_y += DIALOG_FONT_SIZE as f32 + 4.0; // Move to next line
            }
        }
    }

    fn draw_oversize_label(&self, original: String, center_x: f32, center_y: f32) {
        const LINE_SIZE: f32 = 50.0;
        const LABEL_PADDING: f32 = 20.0;

        let (line_end_x, line_end_y) = match (center_x> screen_width()/2.0 , center_y> screen_height()/2.0){
            (true, true) => (center_x - LINE_SIZE, center_y - LINE_SIZE),
            (true, false) =>(center_x - LINE_SIZE, center_y + LINE_SIZE),
            (false, true) =>(center_x + LINE_SIZE, center_y - LINE_SIZE),
            (false, false) =>(center_x + LINE_SIZE, center_y + LINE_SIZE),
        };

        draw_line(center_x, center_y, line_end_x, line_end_y, 2.0, BLACK);

        let text_dimensions =
            measure_text(&original, Some(&self.regular_font), CELL_FONT_SIZE, 1.0);
        let (label_width, label_height) = (
            text_dimensions.width + LABEL_PADDING,
            text_dimensions.height + LABEL_PADDING,
        );
        let label_rect_start_x = line_end_x - label_width / 2.;
        let label_rect_start_y = line_end_y - label_height / 2.;
        draw_rectangle(
            label_rect_start_x,
            label_rect_start_y,
            label_width,
            label_height,
            BLACK,
        );

        draw_text_ex(
            &original,
            label_rect_start_x + LABEL_PADDING / 2.0,
            label_rect_start_y + LABEL_PADDING / 2.0 + text_dimensions.height,
            TextParams {
                font: Some(&self.regular_font),
                font_size: CELL_FONT_SIZE,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: WHITE,
            },
        );
    }
}

fn column_idx_to_string(mut idx: usize) -> String {
    let mut s = String::new();

    loop {
        let rem = (idx % 26) as u8;
        s.insert(0, (b'A' + rem) as char); // Prepend the character
        if idx < 26 {
            break;
        }
        idx = idx / 26 - 1;
    }

    s
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

/*
   Format a float into scientific notation such as: 42.0 -> 4.200e+01
   width controls the amount of left padded spaces
   precision is the amount of decimals
   exp_pad controls the amount of left padded 0s
*/
fn fmt_f64(num: f64, width: usize, precision: usize, exp_pad: usize) -> String {
    if !num.is_finite() {
        return num.to_string();
    }
    let mut num = format!("{:.precision$e}", num, precision = precision);
    // Safe to `unwrap` as `num` is guaranteed to contain `'e'`
    let exp = num.split_off(num.find('e').expect("safe"));

    let (sign, exp) = if exp.starts_with("e-") {
        ('-', &exp[2..])
    } else {
        ('+', &exp[1..])
    };
    num.push_str(&format!("e{}{:0>pad$}", sign, exp, pad = exp_pad));

    format!("{:>width$}", num, width = width)
}

fn computed_to_text(computed: Option<Result<Value, ComputeError>>) -> String {
    match computed {
        Some(value) => match value {
            Ok(inner) => match inner {
                Value::Text(s) => s,
                Value::Number(num) => {
                    if num >= 1E15 {
                        fmt_f64(num, 10, 3, 2)
                    } else {
                        num.to_string()
                    }
                }
                Value::Bool(b) => b.to_string(),
            },
            Err(err) => err.to_string(),
        },
        None => String::new(),
    }
}

fn err_to_info(err: ComputeError) -> String {
    match err {
        ComputeError::ParseError(reason) => reason,
        ComputeError::TypeError(message) => message,
        ComputeError::UnfindableReference(message) => message,
        ComputeError::Cycle => "Detected cyclic computation".to_string(),
        ComputeError::UnknownFunction(f) => format!("Unknown function '{f}'"),
        ComputeError::InvalidArgument(message) => message,
    }
}

fn split_into_lines(text: &str, font: &Font, font_size: u16, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let candidate = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let text_dimensions = measure_text(&candidate, Some(font), font_size, 1.0);

        if text_dimensions.width <= max_width {
            current_line = candidate;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
