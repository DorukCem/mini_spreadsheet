use crate::common_types::Index;

pub fn get_cell_idx(cell_name: &str) -> Option<Index> {
    let mut x: usize = 0;
    let mut y = 0;

    for (i, c) in cell_name.chars().enumerate() {
        if c.is_ascii_digit() {
            // Parse row number
            y = cell_name[i..].parse::<usize>().ok()?;
            break;
        } else {
            // Parse column letters
            x = x * 26 + (c as usize - 'A' as usize + 1);
        }
    }
    if x == 0 || y == 0 {
        return None;
    }
    // Adjust for 0-based indexing
    Some(Index { x: x - 1, y: y - 1 })
}
