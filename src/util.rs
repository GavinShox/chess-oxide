pub fn notation_to_index(n: &str) -> usize {
    let file: char = n.chars().next().unwrap();
    let rank: char = n.chars().nth(1).unwrap();
    let rank_starts = [56, 48, 40, 32, 24, 16, 8, 0]; // 1st to 8th rank starting indexes

    let file_offset = match file {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 0,
    };
    file_offset + rank_starts[(rank.to_digit(10).unwrap() - 1) as usize]
}

pub fn index_to_notation(i: usize) -> String {
    let file = match i % 8 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => ' ',
    };
    let rank_num = 8 - i / 8;
    let rank = char::from_digit(rank_num.try_into().unwrap(), 10).unwrap();
    format!("{}{}", file, rank)
}
