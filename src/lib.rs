use glob::glob;
use rayon::prelude::*;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::str;

const HEADER_START: &[u8] = "ACROSS&DOWN\0".as_bytes();
const BLOCKED_CELL: u8 = b'.';

fn i_xy(i: u16, width: u16) -> (u16, u16) {
    (i % width, i / width)
}

fn xy_i(x: u16, y: u16, width: u16) -> usize {
    (y * width + x) as usize
}

pub fn parse_by_glob(glob_pattern: &str) -> std::io::Result<Vec<Puzzle>> {
    let results = glob(glob_pattern)
        .expect("Could not parse glob pattern")
        .par_bridge()
        .map(|p| parse(p.unwrap().as_path().to_str().unwrap()).unwrap())
        .collect();
    Ok(results)
}

pub fn parse(path: &str) -> std::io::Result<Puzzle> {
    // println!("{}", path);
    let mut file: File = File::open(path)?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    let mut offset = 0;
    loop {
        if &bytes[offset..offset + 12] == HEADER_START {
            offset -= 2;
            break;
        }
        offset += 1;
    }

    let header = &bytes[offset..offset + 52];

    let _prelude = &bytes[0..offset];
    let _checksum = &header[0..2];
    let _across_and_down = &header[2..14];
    let _cib_checksum = &header[14..16];
    let _masked_low_checksum = &header[16..20];
    let _masked_high_checksum = &header[20..24];
    let _version_string = str::from_utf8(&header[24..28]).expect("Version string incorrect");
    let _reserved_1c = &header[28..30];
    let _scrambled_checksum = &header[30..44];
    let width = u8::from_le_bytes(header[44..45].try_into().expect("Width not a u8")) as u16;
    let height = u8::from_le_bytes(header[45..46].try_into().expect("Width not a u8")) as u16;
    let _num_clues = u16::from_le_bytes(header[46..48].try_into().expect("Num clues not a u16"));
    let _unknown_bitmask = &header[48..50];
    let _scrambled_tag = &header[50..52];

    let num_cells = width as usize * height as usize;
    let board_offset = offset + 52;
    let player_board_offset = board_offset + num_cells;

    let board = &bytes[board_offset..board_offset + num_cells];
    let player_state = &bytes[player_board_offset..player_board_offset + num_cells];

    let string_bytes = &bytes[player_board_offset + num_cells..];

    let mut string_buffer = Vec::new();
    let mut strings = Vec::new();

    for char in string_bytes {
        if *char == b'\0' {
            let value = match str::from_utf8(&string_buffer) {
                Ok(v) => v,
                Err(_) => "",
            };
            strings.push(value.to_owned());
            string_buffer.clear();
            continue;
        }
        string_buffer.push(*char);
    }

    let title = &strings[0];
    let author = &strings[1];
    let copywrite = &strings[2];
    let info = &strings[strings.len() - 1];
    let mut clues = VecDeque::from(strings[3..strings.len()].to_owned());
    let mut clue_count: u16 = 1;
    let mut built_clues = Vec::new();

    // println!("{:?}", board);
    for i in 0..width as u16 * height as u16 {
        if board[i as usize] == BLOCKED_CELL {
            continue;
        }
        let (x, y) = i_xy(i, width);
        let mut starts_clue = false;
        if x == 0 || board[xy_i(x - 1, y, width)] == BLOCKED_CELL {
            if x + 1 < width && board[xy_i(x + 1, y, width)] != BLOCKED_CELL {
                starts_clue = true;
                let mut answer = Vec::new();
                for i in 0..width {
                    if i + x == width {
                        break;
                    }
                    let cell = board[xy_i(x + i, y, width)];
                    if cell == BLOCKED_CELL {
                        break;
                    }
                    answer.push(board[xy_i(x + i, y, width)])
                }
                let answer_unpacked = match str::from_utf8(&answer) {
                    Ok(v) => v,
                    Err(_) => "X"
                };
                built_clues.push(Clue {
                    number: clue_count,
                    grid_index: i,
                    value: clues.pop_front().expect("Failed to parse clue"),
                    answer: answer_unpacked.to_string(),
                    length: answer.len().try_into().expect("Too long of clue"),
                    orientation: ClueOrientation::Across,
                });
            }
        }
        if y == 0 || board[xy_i(x, y - 1, width)] == BLOCKED_CELL {
            if y + 1 < height && board[xy_i(x, y + 1, width)] != BLOCKED_CELL {
                starts_clue = true;
                let mut answer = Vec::new();
                for i in 0..height {
                    if i + y == height {
                        break;
                    }
                    let cell = board[xy_i(x, y + i, width)];
                    if cell == BLOCKED_CELL {
                        break;
                    }
                    answer.push(board[xy_i(x, y + i, width)])
                }
                let answer_unpacked = match str::from_utf8(&answer) {
                    Ok(v) => v,
                    Err(_) => "X"
                };
                built_clues.push(Clue {
                    number: clue_count,
                    grid_index: i,
                    value: clues.pop_front().expect("Failed to parse file."),
                    answer: answer_unpacked.to_string(),
                    length: i + 1,
                    orientation: ClueOrientation::Down,
                });
            }
        }
        if starts_clue {
            clue_count += 1;
        }
    }

    Ok(Puzzle {
        title: title.to_string(),
        author: author.to_string(),
        copywrite: copywrite.to_string(),
        info: info.to_string(),
        width,
        height,
        clues: built_clues,
        board: board
            .to_vec()
            .iter()
            .map(|b| char::from_u32((*b) as u32).unwrap())
            .collect(),
        player_state: player_state
            .to_vec()
            .iter()
            .map(|b| char::from_u32((*b) as u32).unwrap())
            .collect(),
    })
}

#[derive(Debug)]
pub struct Puzzle {
    title: String,
    author: String,
    copywrite: String,
    info: String,
    width: u16,
    height: u16,
    clues: Vec<Clue>,
    board: Vec<char>,
    player_state: Vec<char>,
}

#[derive(Debug)]
pub enum ClueOrientation {
    Across,
    Down,
}
#[derive(Debug)]
pub struct Clue {
    number: u16,
    grid_index: u16,
    value: String,
    answer: String,
    length: u16,
    orientation: ClueOrientation,
}
