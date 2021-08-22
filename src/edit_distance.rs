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

pub fn edit_distance<T: std::cmp::Eq>(s1: Vec<T>, s2: Vec<T>) -> Vec<EditType> {
    let h = s1.len() + 1;
    let w = s2.len() + 1;
    let mut m: Vec<usize> = vec![Default::default(); w * h];
    // Left is insert, up is delete, and diagonal is substitution
    let mut backtrack: Vec<EditType> = vec![Default::default(); w * h];
    let mut backtrack2: Vec<(i32, i32)> = vec![Default::default(); w * h];
    let get = |x: usize, y: usize| x + (y * w);
    // Init
    for i in 0..h {
        m[get(0, i)] = i;
        backtrack[get(0, i)] = EditType::Delete(if i > 0 { i - 1 } else { 0 });
    }
    for i in 0..w {
        m[get(i, 0)] = i;
        backtrack[get(i, 0)] = EditType::Insert(if i > 0 { i - 1 } else { 0 });
    }
    backtrack[get(0, 0)] = EditType::Unknown;
    // Compute
    let v2 = vec![(-1, 0), (0, -1), (-1, -1)];
    for y in 1..(s1.len() + 1) {
        for x in 1..(s2.len() + 1) {
            let substitution_cost = if s1[y - 1] == s2[x - 1] { 0 } else { 2 };
            let v = vec![
                m[get(x - 1, y)] + 1,
                m[get(x, y - 1)] + 1,
                m[get(x - 1, y - 1)] + substitution_cost,
            ];
            let i = min_arg(&v);
            let v3 = match i {
                0 => EditType::Insert(x - 1),
                1 => EditType::Delete(y - 1),
                2 => EditType::Substitute(y - 1, x - 1, substitution_cost == 0),
                _ => EditType::Unknown,
            };
            backtrack[get(x, y)] = v3;
            backtrack2[get(x, y)] = v2[i];
            let mn = v[i];
            m[get(x, y)] = mn;
        }
    }
    let mut m4: Vec<Vec<(i32, i32)>> = Vec::new();
    for y in 0..h {
        let mut t = Vec::new();
        for x in 0..w {
            t.push(backtrack2[get(x, y)]);
        }
        m4.push(t);
    }
    let mut pos = (w - 1, h - 1);
    backtrack.push(EditType::Unknown);
    let mut n = backtrack.swap_remove(get(pos.0, pos.1));
    let mut r2: Vec<EditType> = Vec::new();
    while !matches!(n, EditType::Unknown) {
        match n {
            EditType::Insert(_) => pos = (pos.0 - 1, pos.1),
            EditType::Delete(_) => pos = (pos.0, pos.1 - 1),
            EditType::Substitute(_, _, _) => pos = (pos.0 - 1, pos.1 - 1),
            EditType::Unknown => pos = (0, 0),
        }
        r2.push(n);
        backtrack.push(EditType::Unknown);
        n = backtrack.swap_remove(get(pos.0, pos.1));
    }
    r2
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
