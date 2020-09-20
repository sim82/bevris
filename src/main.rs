use arrayvec::ArrayVec;
use bevy::{
    input::keyboard::{ElementState as KeyboardElementState, KeyboardInput},
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use rand::prelude::*;

/// An implementation of the classic game "Breakout"
fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_resource(State {
            timer: Timer::new(std::time::Duration::from_millis(250), true),
            fast_timer: Timer::new(std::time::Duration::from_millis(50), true),
            ..Default::default()
        })
        .init_resource::<PieceBag>()
        .add_startup_system(setup.system())
        .add_plugin(BetrisPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents {
            transform: Transform::from_translation(Vec3::new(32f32 * 5f32, 32f32 * 10f32, 1.0)),
            ..Default::default()
        })
        .spawn(UiCameraComponents::default())
        // scoreboard
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                value: "Score:".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 40.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);
}

struct Scoreboard {
    score: usize,
}

struct Playfield {
    field: [[u8; 10]; 22],
}

struct Field {
    x: i32,
    y: i32,
}

impl Playfield {
    fn new() -> Self {
        Playfield {
            field: [[0u8; 10]; 22],
        }
    }
}

struct FieldMaterials {
    materials: Vec<Handle<ColorMaterial>>,
}

#[derive(Debug)]
enum PieceType {
    I,
    L,
    J,
    S,
    Z,
    T,
    O,
}

#[derive(Clone)]
struct Piece {
    x: i32,
    y: i32,
    rot: i32,
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

fn get_solid(t: &PieceType, p: &Piece) -> [(i32, i32); 4] {
    let piece_i = ".o..\n\
                         .o..\n\
                         .o..\n\
                         .o..\n\
                         ....\n\
                         oooo\n\
                         ....\n\
                         ....";

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
    let base = match *t {
        PieceType::I => parse_piece(piece_i),
        PieceType::L => parse_piece(piece_l),
        PieceType::J => parse_piece(piece_j),
        PieceType::S => parse_piece(piece_s),
        PieceType::Z => parse_piece(piece_z),
        PieceType::O => parse_piece(piece_o),
        PieceType::T => parse_piece(piece_t),
    };
    let trans: ArrayVec<[_; 4]> = base[p.rot as usize % base.len()]
        .iter()
        .map(|(x, y)| (x + p.x, y + p.y))
        .collect();
    trans.into_inner().unwrap()
}

fn get_color(t: &PieceType) -> usize {
    match *t {
        PieceType::I => 2,
        PieceType::L => 3,
        PieceType::J => 4,
        PieceType::S => 5,
        PieceType::Z => 6,
        PieceType::O => 7,
        PieceType::T => 8,
    }
}

impl FieldMaterials {
    fn new(mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        let colors = [
            Color::rgb(0.0, 0.0, 0.0),
            Color::rgb(1.0, 1.0, 1.0),
            Color::rgb(1.0, 0.0, 0.0),
            Color::rgb(0.0, 1.0, 0.0),
            Color::rgb(0.0, 0.0, 1.0),
            Color::rgb(1.0, 1.0, 0.0),
            Color::rgb(0.0, 1.0, 1.0),
            Color::rgb(1.0, 0.0, 1.0),
            Color::rgb(1.0, 0.5, 0.5),
        ];

        FieldMaterials {
            materials: colors
                .iter()
                .map(|c| materials.add(c.clone().into()))
                .collect(),
        }
    }
}

fn init_field(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut piece_bag: ResMut<PieceBag>,
) {
    let field_materials = FieldMaterials::new(materials);
    let mut i = 0;
    for y in 0..22 {
        for x in 0..10 {
            commands
                .spawn(SpriteComponents {
                    material: field_materials.materials[1],
                    transform: Transform::from_translation(Vec3::new(
                        (x * 32) as f32,
                        (y * 32) as f32,
                        1.0,
                    )),
                    sprite: Sprite::new(Vec2::new(32f32, 32f32)),
                    ..Default::default()
                })
                .with(Field { x, y });
            i += 1;
        }
    }
    commands.insert_resource(field_materials);
    commands.spawn((
        piece_bag.next(),
        Piece {
            x: 5,
            y: 18,
            rot: 0,
        },
    ));
}

fn field_update_system(
    playfield: Res<Playfield>,
    materials: Res<FieldMaterials>,
    mut query: Query<(&Field, &Sprite, &mut Handle<ColorMaterial>)>,
) {
    for (field, _, mut material) in &mut query.iter() {
        *material =
            materials.materials[playfield.field[field.y as usize][field.x as usize] as usize];
    }
}

fn modify_test(mut playfield: ResMut<Playfield>) {
    playfield
        .field
        .iter_mut()
        .for_each(|rows| rows.iter_mut().for_each(|f| *f = rand::random::<u8>() % 8));
}

fn player_input_system(
    mut commands: Commands,
    time: Res<Time>,
    mut playfield: ResMut<Playfield>,
    mut state: ResMut<State>,
    mut piece_bag: ResMut<PieceBag>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &PieceType, &mut Piece)>,
) {
    for (ent, t, mut p) in &mut query.iter() {
        // delete old pos
        for (x, y) in get_solid(t, &*p).iter() {
            playfield.field[*y as usize][*x as usize] = 0;
        }
        let mut pnew = p.clone();
        // update input
        for event in state.event_reader.iter(&keyboard_input_events) {
            println!("{:?}", event);
            match event {
                KeyboardInput {
                    key_code: Some(key_code),
                    state: KeyboardElementState::Pressed,
                    ..
                } => match key_code {
                    KeyCode::Left => pnew.x -= 1,
                    KeyCode::Right => pnew.x += 1,
                    KeyCode::Space => pnew.rot += 1,
                    _ => (),
                },
                _ => (),
            }
        }

        let illegal_user_move = get_solid(&t, &pnew)
            .iter()
            .map(|(x, y)| {
                *x < 0
                    || *x >= 10
                    || *y < 0
                    || *y >= 22
                    || playfield.field[*y as usize][*x as usize] != 0
            })
            .any(|x| x);

        if !illegal_user_move {
            *p = pnew;
        }
    }
}

fn piece_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut playfield: ResMut<Playfield>,
    mut state: ResMut<State>,
    mut piece_bag: ResMut<PieceBag>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &PieceType, &mut Piece)>,
) {
    state.timer.tick(time.delta_seconds);
    state.fast_timer.tick(time.delta_seconds);
    for (ent, t, mut p) in &mut query.iter() {
        let fast_move = if keyboard_input.pressed(KeyCode::Down) {
            if state.fast_generation.is_none() {
                state.fast_generation = Some(ent);
            }
            state.fast_generation == Some(ent)
        } else {
            state.fast_generation = None;
            false
        };

        let mut pnew = p.clone();
        let do_move =
            (!fast_move && state.timer.finished) || (fast_move && state.fast_timer.finished);
        state.timer.finished = false;
        state.fast_timer.finished = false;
        if do_move {
            pnew.y -= 1;
            state.timer.finished = false;
        }

        let on_ground = get_solid(&t, &pnew)
            .iter()
            .map(|(x, y)| *y < 0 || playfield.field[*y as usize][*x as usize] != 0)
            .any(|x| x);

        // draw new pos
        if !on_ground {
            *p = pnew;
        }

        let color = get_color(t);
        for (x, y) in get_solid(t, &*p).iter() {
            playfield.field[*y as usize][*x as usize] = color as u8;
        }

        if on_ground {
            let next = piece_bag.next();
            check_lines_system(&mut *playfield);
            println!("hit ground. next: {:?}", next);
            commands
                .spawn((
                    next,
                    Piece {
                        x: 5,
                        y: 18,
                        rot: 0,
                    },
                ))
                .despawn(ent);
        }
    }
}

fn check_lines_system(playfield: &mut Playfield) {
    loop {
        let mut eliminate = None;
        for (y, line) in playfield.field.iter().enumerate() {
            if line.iter().all(|x| *x != 0) {
                eliminate = Some(y);
                break;
            }
        }

        match eliminate {
            Some(line) => {
                for y in line..21 {
                    // TODO: bubble up with swap?
                    // std::mem::swap(&mut playfield.field[y], &mut playfield.field[y + 1]);
                    playfield.field[y] = playfield.field[y + 1].clone();
                }
                playfield.field[21] = [0u8; 10];
            }
            None => break,
        }
    }
    // let eliminate = playfield
    //     .field
    //     .iter()
    //     .map(|line| !line.iter().map(|field| *x == 0).any());
}

struct BetrisPlugin;
#[derive(Default)]
struct State {
    event_reader: EventReader<KeyboardInput>,
    timer: Timer,
    fast_timer: Timer,
    fast_generation: Option<Entity>,
}
#[derive(Default)]
struct PieceBag {
    bag: Vec<PieceType>,
}

impl PieceBag {
    fn next(&mut self) -> PieceType {
        if self.bag.is_empty() {
            self.bag = vec![
                PieceType::I,
                PieceType::L,
                PieceType::J,
                PieceType::S,
                PieceType::Z,
                PieceType::O,
                PieceType::T,
            ];

            self.bag.shuffle(&mut rand::thread_rng());
        }
        self.bag.pop().unwrap()
    }
}

impl Plugin for BetrisPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_field.system())
            .add_resource(Playfield::new())
            // .add_system(modify_test.system())
            .add_system(player_input_system.system())
            .add_system(piece_update_system.system())
            // .add_system(check_lines_system.system())
            .add_system(field_update_system.system());
    }
}
