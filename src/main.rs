use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    input::keyboard::{ElementState as KeyboardElementState, KeyboardInput},
    prelude::*,
    render::pass::ClearColor,
};
use rand::prelude::*;

mod field;
mod pieces;

use field::Playfield;
use pieces::{get_solid, get_solid_base, Piece, PieceType};

fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Scoreboard { _score: 0 })
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut piece_bag: ResMut<PieceBag>) {
    // Add the game's entities to our world
    commands
        // cameras
        .spawn(Camera2dComponents {
            transform: Transform::from_translation(Vec3::new(32f32 * 5f32, 32f32 * 10f32, 1.0)),
            ..Default::default()
        })
        // .spawn(UiCameraComponents::default()) // FIXME: the UI camera causes some wgpu internal crash if TextureSheetSprites are used
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
        })
        .spawn((
            piece_bag.next(),
            Piece {
                x: 5,
                y: 18,
                rot: 0,
            },
        ));
}

struct Scoreboard {
    _score: usize,
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

fn player_input_system(
    mut playfield: ResMut<Playfield>,
    mut state: ResMut<State>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    mut query: Query<(&PieceType, &mut Piece)>,
) {
    for (t, mut p) in &mut query.iter() {
        // delete old pos
        for (x, y) in get_solid(t, &*p).iter() {
            playfield.field[*y as usize][*x as usize] = 0;
        }
        let mut pnew = p.clone();
        // update input
        for event in state.event_reader.iter(&keyboard_input_events) {
            // println!("{:?}", event);
            match event {
                KeyboardInput {
                    key_code: Some(key_code),
                    state: KeyboardElementState::Pressed,
                    ..
                } => match key_code {
                    KeyCode::Left => pnew.x -= 1,
                    KeyCode::Right => pnew.x += 1,
                    KeyCode::Space | KeyCode::Up => pnew.rot += 1,
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
            // println!("hit ground. next: {:?}", next);
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
}

struct Preview;

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
    preview: Option<PieceType>,
}

impl PieceBag {
    fn next_int(&mut self) -> PieceType {
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

    fn next(&mut self) -> PieceType {
        let ret = match self.preview {
            None => self.next_int(),
            Some(piece) => piece,
        };
        self.preview = Some(self.next_int());
        ret
    }
    fn peek_preview(&mut self) -> PieceType {
        if self.preview.is_none() {
            self.preview = Some(self.next_int());
        }
        self.preview.unwrap()
    }
}

impl Plugin for BetrisPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            // Adds a system that prints diagnostics to the console
            .add_plugin(PrintDiagnosticsPlugin::default())
            .add_resource(Playfield::new())
            // .add_system(modify_test.system())
            .add_system(player_input_system.system())
            .add_system(piece_update_system.system())
            // .add_system(check_lines_system.system())
            // .add_plugin(field::SolidFieldPlugin)
            .add_plugin(field::TexturedFieldPlugin)
            // sentinel
            ;
    }
}
