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

pub struct Pieces {
    i: Vec<[(i32, i32, i32); 4]>,
    l: Vec<[(i32, i32, i32); 4]>,
    j: Vec<[(i32, i32, i32); 4]>,
    s: Vec<[(i32, i32, i32); 4]>,
    z: Vec<[(i32, i32, i32); 4]>,
    o: Vec<[(i32, i32, i32); 4]>,
    t: Vec<[(i32, i32, i32); 4]>,
}

fn parse_piece(i: &str) -> Vec<[(i32, i32, i32); 4]> {
    let lines = i.lines().collect::<Vec<_>>();
    // let out: Vec<Vec<(i32, i32)>>
    let out = lines
        .chunks(4)
        .map(|lines| {
            lines
                .iter()
                .enumerate()
                .map(|(y, line)| {
                    line.chars().enumerate().map(move |(x, c)| match c {
                        '0'..='9' | 'a'..='f' => {
                            Some((x as i32, 3 - y as i32, c.to_digit(16).unwrap() as i32))
                        }
                        'o' => Some((x as i32, 3 - y as i32, 1)),
                        _ => None,
                    })
                })
                .flatten()
                .filter_map(|x| x)
                .collect::<ArrayVec<[(i32, i32, i32); 4]>>()
        })
        .map(|x| x.into_inner().unwrap())
        .collect::<Vec<_>>();
    out
}

impl Pieces {
    fn new() -> Self {
        let piece_i = "....\n\
                         9aab\n\
                         ....\n\
                         ....\n\
                         .c..\n\
                         .d..\n\
                         .d..\n\
                         .e..";

        let piece_l = "....\n\
                         333.\n\
                         3...\n\
                         ....\n\
                         .3..\n\
                         .3..\n\
                         .33.\n\
                         ....\n\
                         ..3.\n\
                         333.\n\
                         ....\n\
                         ....\n\
                         33..\n\
                         .3..\n\
                         .3..\n\
                         ....";

        let piece_j = "....\n\
                         444.\n\
                         ..4.\n\
                         ....\n\
                         .4..\n\
                         .4..\n\
                         44..\n\
                         ....\n\
                         4...\n\
                         444.\n\
                         ....\n\
                         ....\n\
                         .44.\n\
                         .4..\n\
                         .4..\n\
                         ....";

        let piece_s = "....\n\
                         .55.\n\
                         55..\n\
                         ....\n\
                         5...\n\
                         55..\n\
                         .5..\n\
                         ....";

        let piece_z = "....\n\
                         66..\n\
                         .66.\n\
                         ....\n\
                         .6..\n\
                         66..\n\
                         6...\n\
                         ....";

        let piece_o = "....\n\
                         .77.\n\
                         .77.\n\
                         ....";

        let piece_t = "....\n\
                         888.\n\
                         .8..\n\
                         ....\n\
                         .8..\n\
                         88..\n\
                         .8..\n\
                         ....\n\
                         .8..\n\
                         888.\n\
                         ....\n\
                         ....\n\
                         .8..\n\
                         .88.\n\
                         .8..\n\
                         ....";
        Pieces {
            i: parse_piece(piece_i),
            l: parse_piece(piece_l),
            j: parse_piece(piece_j),
            s: parse_piece(piece_s),
            z: parse_piece(piece_z),
            o: parse_piece(piece_o),
            t: parse_piece(piece_t),
        }
    }

    pub fn get_solid_base(&self, t: &PieceType) -> &Vec<[(i32, i32, i32); 4]> {
        match *t {
            PieceType::I => &self.i,
            PieceType::L => &self.l,
            PieceType::J => &self.j,
            PieceType::S => &self.s,
            PieceType::Z => &self.z,
            PieceType::O => &self.o,
            PieceType::T => &self.t,
        }
    }

    pub fn get_solid(&self, t: &PieceType, p: &Piece) -> [(i32, i32, i32); 4] {
        let base = self.get_solid_base(t);
        let trans: ArrayVec<[_; 4]> = base[p.rot as usize % base.len()]
            .iter()
            .map(|(x, y, c)| (x + p.x, y + p.y, *c))
            .collect();
        trans.into_inner().unwrap()
    }
}

impl Default for Pieces {
    fn default() -> Self {
        Pieces::new()
    }
}
