use std::fmt;

#[derive(Clone, Debug)]
pub enum EditType {
    Insert(usize),
    Delete(usize),
    Substitute(usize, usize, bool),
    Unknown,
}

impl Default for EditType {
    fn default() -> Self {
        EditType::Unknown
    }
}

impl fmt::Display for EditType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self {
            EditType::Insert(x) => format!("Insert({})", x),
            EditType::Delete(x) => format!("Delete({})", x),
            EditType::Substitute(x, y, is_same) => {
                format!("Substitute({}, {}, is_same = {})", x, y, is_same)
            }
            EditType::Unknown => "Unknown".to_string(),
        };
        write!(f, "{}", v)
    }
}

pub fn edit_distance<T: std::cmp::Eq>(arg1: Vec<T>, arg2: Vec<T>) -> Vec<EditType> {
    let h = arg1.len() + 1;
    let w = arg2.len() + 1;
    let mut edit_matrix: Vec<usize> = vec![Default::default(); w * h];
    // Left is insert, up is delete, and diagonal is substitution
    let mut backtrack: Vec<EditType> = vec![Default::default(); w * h];
    // Compute 1D array position from 2D coordinates
    let get_array_position = |x: usize, y: usize| x + (y * w);
    // Init
    for i in 0..h {
        edit_matrix[get_array_position(0, i)] = i;
        backtrack[get_array_position(0, i)] = EditType::Delete(if i > 0 { i - 1 } else { 0 });
    }
    for i in 0..w {
        edit_matrix[get_array_position(i, 0)] = i;
        backtrack[get_array_position(i, 0)] = EditType::Insert(if i > 0 { i - 1 } else { 0 });
    }
    backtrack[get_array_position(0, 0)] = EditType::Unknown;
    // Compute
    for y in 1..(arg1.len() + 1) {
        for x in 1..(arg2.len() + 1) {
            let substitution_cost = if arg1[y - 1] == arg2[x - 1] { 0 } else { 2 };
            let prefix_dists = vec![
                edit_matrix[get_array_position(x - 1, y)] + 1,
                edit_matrix[get_array_position(x, y - 1)] + 1,
                edit_matrix[get_array_position(x - 1, y - 1)] + substitution_cost,
            ];
            let prefix_dist_index = min_arg(&prefix_dists);
            let v3 = match prefix_dist_index {
                0 => EditType::Insert(x - 1),
                1 => EditType::Delete(y - 1),
                2 => EditType::Substitute(y - 1, x - 1, substitution_cost == 0),
                _ => EditType::Unknown,
            };
            backtrack[get_array_position(x, y)] = v3;
            let mn = prefix_dists[prefix_dist_index];
            edit_matrix[get_array_position(x, y)] = mn;
        }
    }
    let mut pos = (w - 1, h - 1);
    let mut current_char = backtrack.get(get_array_position(pos.0, pos.1)).unwrap();
    let mut output: Vec<EditType> = Vec::new();
    while !matches!(current_char, EditType::Unknown) {
        match current_char {
            EditType::Insert(_) => pos = (pos.0 - 1, pos.1),
            EditType::Delete(_) => pos = (pos.0, pos.1 - 1),
            EditType::Substitute(_, _, _) => pos = (pos.0 - 1, pos.1 - 1),
            EditType::Unknown => pos = (0, 0),
        }
        output.push(current_char.clone());
        current_char = backtrack.get(get_array_position(pos.0, pos.1)).unwrap();
    }

    output.reverse();
    output
}

fn min_arg<T: std::cmp::Ord + std::clone::Clone>(v: &Vec<T>) -> usize {
    v.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_edit_distance() {
        let res = edit_distance("2334".chars().collect(), "1223344".chars().collect());
        println!("{:?}", res);
    }
}
