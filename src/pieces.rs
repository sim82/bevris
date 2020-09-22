use arrayvec::ArrayVec;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PieceType {
    I,
    L,
    J,
    S,
    Z,
    T,
    O,
}

#[derive(Clone)]
pub struct Piece {
    pub x: i32,
    pub y: i32,
    pub rot: i32,
}

fn parse_piece(i: &str) -> Vec<[(i32, i32); 4]> {
    let lines = i.lines().collect::<Vec<_>>();
    // let out: Vec<Vec<(i32, i32)>>
    let out = lines
        .chunks(4)
        .map(|lines| {
            lines
                .iter()
                .enumerate()
                .map(|(y, line)| {
                    line.bytes().enumerate().map(move |(x, c)| match c as char {
                        'o' => Some((x as i32, 3 - y as i32)),
                        _ => None,
                    })
                })
                .flatten()
                .filter_map(|x| x)
                .collect::<ArrayVec<[(i32, i32); 4]>>()
        })
        .map(|x| x.into_inner().unwrap())
        .collect::<Vec<_>>();
    out
}

pub fn get_solid_base(t: &PieceType) -> Vec<[(i32, i32); 4]> {
    let piece_i = "....\n\
                         oooo\n\
                         ....\n\
                         ....\n\
                         .o..\n\
                         .o..\n\
                         .o..\n\
                         .o..";

    let piece_l = "....\n\
                         ooo.\n\
                         o...\n\
                         ....\n\
                         .o..\n\
                         .o..\n\
                         .oo.\n\
                         ....\n\
                         ..o.\n\
                         ooo.\n\
                         ....\n\
                         ....\n\
                         oo..\n\
                         .o..\n\
                         .o..\n\
                         ....";

    let piece_j = "....\n\
                         ooo.\n\
                         ..o.\n\
                         ....\n\
                         .o..\n\
                         .o..\n\
                         oo..\n\
                         ....\n\
                         o...\n\
                         ooo.\n\
                         ....\n\
                         ....\n\
                         .oo.\n\
                         .o..\n\
                         .o..\n\
                         ....";

    let piece_s = "....\n\
                         .oo.\n\
                         oo..\n\
                         ....\n\
                         o...\n\
                         oo..\n\
                         .o..\n\
                         ....";

    let piece_z = "....\n\
                         oo..\n\
                         .oo.\n\
                         ....\n\
                         .o..\n\
                         oo..\n\
                         o...\n\
                         ....";

    let piece_o = "....\n\
                         .oo.\n\
                         .oo.\n\
                         ....";

    let piece_t = "....\n\
                         ooo.\n\
                         .o..\n\
                         ....\n\
                         .o..\n\
                         oo..\n\
                         .o..\n\
                         ....\n\
                         .o..\n\
                         ooo.\n\
                         ....\n\
                         ....\n\
                         .o..\n\
                         .oo.\n\
                         .o..\n\
                         ....";

    // TODO: cache this somewhere
    match *t {
        PieceType::I => parse_piece(piece_i),
        PieceType::L => parse_piece(piece_l),
        PieceType::J => parse_piece(piece_j),
        PieceType::S => parse_piece(piece_s),
        PieceType::Z => parse_piece(piece_z),
        PieceType::O => parse_piece(piece_o),
        PieceType::T => parse_piece(piece_t),
    }
}

pub fn get_solid(t: &PieceType, p: &Piece) -> [(i32, i32); 4] {
    let base = get_solid_base(t);
    let trans: ArrayVec<[_; 4]> = base[p.rot as usize % base.len()]
        .iter()
        .map(|(x, y)| (x + p.x, y + p.y))
        .collect();
    trans.into_inner().unwrap()
}
