fn get_letter(c: char) -> Result<&'static [i64], UnknownLetter> {
    Ok(match c.to_ascii_uppercase() {
        ' ' => &[0],
        'A' => &[
            777700077777,
            777007007777,
            770077700777,
            700777770077,
            700000000077,
            700777770077,
            700777770077,
        ],
        'B' => &[
            700000000777,
            700777770077,
            700777770077,
            700000000777,
            700777770077,
            700777770077,
            700000000777,
        ],
        'C' => &[
            770000007777,
            700777700777,
            700777777777,
            700777777777,
            700777777777,
            700777700777,
            770000007777,
        ],
        'D' => &[
            700000000777,
            700777770077,
            700777770077,
            700777770077,
            700777770077,
            700777770077,
            700000000777,
        ],
        'E' => &[
            700000000777,
            700777777777,
            700777777777,
            700000077777,
            700777777777,
            700777777777,
            700000000777,
        ],
        'F' => &[
            700000000777,
            700777777777,
            700777777777,
            700000077777,
            700777777777,
            700777777777,
            700777777777,
        ],
        'G' => &[
            770000007777,
            700777700777,
            700777777777,
            700777000077,
            700777700777,
            700777700777,
            770000007777,
        ],
        'H' => &[
            700777770077,
            700777770077,
            700777770077,
            700000000077,
            700777770077,
            700777770077,
            700777770077,
        ],
        'I' => &[
            777000077777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
            777000077777,
        ],
        'J' => &[
            700777777777,
            700777777777,
            700777777777,
            700777777777,
            700777700777,
            700777700777,
            770000007777,
        ],
        'K' => &[
            700777700777,
            700777007777,
            700770077777,
            700000777777,
            700770077777,
            700777007777,
            700777700777,
        ],
        'L' => &[
            700777777777,
            700777777777,
            700777777777,
            700777777777,
            700777777777,
            700777777777,
            700000000777,
        ],
        'M' => &[
            700777770077,
            700077700077,
            700007000077,
            700700070077,
            700777770077,
            700777770077,
            700777770077,
        ],
        'N' => &[
            700777700777,
            700077700777,
            700007700777,
            700700700777,
            700770000777,
            700777000777,
            700777700777,
        ],
        'O' => &[
            770000000777,
            700777770077,
            700777770077,
            700777770077,
            700777770077,
            700777770077,
            770000000777,
        ],
        'P' => &[
            700000000777,
            700777770077,
            700777770077,
            700000000777,
            700777777777,
            700777777777,
            700777777777,
        ],
        'Q' => &[
            770000000777,
            700777770077,
            700777770077,
            700777770077,
            700770070077,
            700777700777,
            770000070077,
        ],
        'R' => &[
            700000000777,
            700777770077,
            700777770077,
            700000000777,
            700777007777,
            700777700777,
            700777770077,
        ],
        'S' => &[
            770000007777,
            700777700777,
            700777777777,
            770000007777,
            777777700777,
            700777700777,
            770000007777,
        ],
        'T' => &[
            700000000777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
        ],
        'U' => &[
            700777700777,
            700777700777,
            700777700777,
            700777700777,
            700777700777,
            770777707777,
            777000077777,
        ],
        'V' => &[
            700777770077,
            700777770077,
            700777770077,
            700777770077,
            770077700777,
            777007007777,
            777700077777,
        ],
        'W' => &[
            700770077007,
            700770077007,
            700770077007,
            700770077007,
            700770077007,
            700770077007,
            770007700077,
        ],
        'X' => &[
            700777770077,
            770077700777,
            777007007777,
            777700077777,
            777007007777,
            770077700777,
            770077777007,
        ],
        'Y' => &[
            700777700777,
            770077007777,
            777000077777,
            777700777777,
            777700777777,
            777700777777,
            777700777777,
        ],
        'Z' => &[
            700000000777,
            777777007777,
            777770077777,
            777700777777,
            777007777777,
            770077777777,
            700000000777,
        ],
        _ => return Err(UnknownLetter(c)),
    })
}

#[derive(Debug)]
pub struct UnknownLetter(char);

impl std::fmt::Display for UnknownLetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown letter {:?}", self.0)
    }
}

impl std::error::Error for UnknownLetter {}

pub fn get_word(s: &str) -> Result<Vec<i64>, UnknownLetter> {
    let mut res = Vec::new();
    for c in s.chars() {
        res.extend(get_letter(c)?);
        res.push(0);
    }
    res.push(0);
    Ok(res)
}