use super::{get_color, get_solid_base, LineTransition, PieceBag, PieceType, Preview};
use bevy::prelude::*;
use bevy::{
    property::PropertyTypeRegistry,
    type_registry::{ComponentRegistry, TypeRegistry},
};
use rand::prelude::*;
use std::collections::HashSet;
pub struct Playfield {
    pub field: [[u8; 10]; 22],
}

impl Playfield {
    pub fn new() -> Self {
        Playfield {
            field: [[0u8; 10]; 22],
        }
    }
}

pub struct Field {
    x: i32,
    y: i32,
    r: u32,
}

pub struct FieldMaterials {
    pub materials: Vec<Handle<ColorMaterial>>,
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

    // fn from_tilemap()
}

fn init_field_solid(mut commands: Commands, materials: ResMut<Assets<ColorMaterial>>) {
    let field_materials = FieldMaterials::new(materials);
    // tragicomic inversion: use sprites to emulate a primitive tiled background.
    // don't tell the TED chip in your c16, it might commit suicide...
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
                .with(Field {
                    x,
                    y,
                    r: rand::thread_rng().gen::<u32>(),
                });
        }
    }
    commands.insert_resource(field_materials);
}

fn field_update_system_solid(
    playfield: Res<Playfield>,
    materials: Res<FieldMaterials>,
    mut query: Query<(&Field, &Sprite, &mut Handle<ColorMaterial>)>,
) {
    for (field, _, mut material) in &mut query.iter() {
        *material =
            materials.materials[playfield.field[field.y as usize][field.x as usize] as usize];
    }
}

fn preview_system_solid(
    mut commands: Commands,
    mut piece_bag: ResMut<PieceBag>,
    field_materials: Res<FieldMaterials>,
    mut preview_query: Query<(Entity, &Preview, &PieceType)>,
) {
    let current_preview = piece_bag.peek_preview();
    let mut create_preview = true;
    for (ent, _, piece_type) in &mut preview_query.iter() {
        if *piece_type != current_preview {
            println!("despawn preview");
            commands.despawn(ent);
        } else {
            create_preview = false; // this assumes that all Preview entities have the same PieceType
        }
    }

    let preview_pos = Vec3::new(32. * 12., 32. * 15., 0.);
    if create_preview {
        for (i, (x, y)) in get_solid_base(&current_preview)[0].iter().enumerate() {
            println!("spawn preview {}", i);
            commands
                .spawn(SpriteComponents {
                    material: field_materials.materials[get_color(&current_preview)],
                    transform: Transform::from_translation(
                        Vec3::new((x * 32) as f32, (y * 32) as f32, 1.0) + preview_pos,
                    ),
                    sprite: Sprite::new(Vec2::new(32f32, 32f32)),
                    ..Default::default()
                })
                .with(Preview)
                .with(current_preview);
        }
    }
}

pub struct SolidFieldPlugin;

impl Plugin for SolidFieldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_field_solid.system())
            // .add_system(scene_save_system.thread_local_system())
            .add_system(preview_system_solid.system())
            .add_system(field_update_system_solid.system());
    }
}

// textured

fn init_field_textured(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server
        .load_sync(&mut textures, "assets/textures/gb.png")
        // .load_sync(&mut textures, "assets/textures/gabe-idle-run.png")
        .unwrap();
    let texture = textures.get(&texture_handle).unwrap();
    let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 16, 2);
    let texture_atlas_handle = texture_atlases.add_default(texture_atlas);
    // texture_atlases.add_default(texture_atlas);
    // tragicomic inversion: use sprites to emulate a primitive tiled background.
    // don't tell the TED chip in your c16, it might commit suicide...
    for y in 0..22 {
        for x in 0..10 {
            commands
                .spawn(SpriteSheetComponents {
                    texture_atlas: texture_atlas_handle,
                    transform: Transform::from_scale(4.0).with_translation(Vec3::new(
                        (x * 32) as f32,
                        (y * 32) as f32,
                        1.0,
                    )),
                    ..Default::default()
                })
                .with(Field {
                    x,
                    y,
                    r: rand::thread_rng().gen::<u32>(),
                });
        }
    }
}

fn clamp(v: i32, max: u32) -> u32 {
    if v < 0 {
        0
    } else if v as u32 >= max {
        max
    } else {
        v as u32
    }
}

fn field_update_system_textured(
    playfield: Res<Playfield>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&Field, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
    mut query_line_transitions: Query<&LineTransition>,
) {
    let mut eliminate_lines = HashSet::new();
    let mut progress = 0.0;
    for lt in &mut query_line_transitions.iter() {
        eliminate_lines = lt.to_eliminate.iter().cloned().collect();
        progress = lt.timer.elapsed as f32 / lt.timer.duration as f32;
    }

    for (field, mut sprite, texture_atlas_handle) in &mut query.iter() {
        let texture_atlas = texture_atlases.get(&texture_atlas_handle).unwrap();

        let r = (field.r % 4) as i32;
        // sprite.index = 16 + clamp(r + (progress * 12f32) as i32, 8) as u32;

        let global_progress = (progress * 12f32) as i32;
        let explode = eliminate_lines.contains(&(field.y as usize)) && global_progress >= r;

        if explode {
            sprite.index = 16 + (global_progress - r) as u32;
        } else {
            sprite.index = (playfield.field[field.y as usize][field.x as usize] as usize
                % texture_atlas.textures.len()) as u32;
        }
    }
}

fn preview_system_textured(
    mut commands: Commands,
    mut piece_bag: ResMut<PieceBag>,
    mut preview_query: Query<(Entity, &Preview, &PieceType)>,
) {
    let current_preview = piece_bag.peek_preview();
    let mut create_preview = true;
    for (ent, _, piece_type) in &mut preview_query.iter() {
        if *piece_type != current_preview {
            // println!("despawn preview");
            commands.despawn(ent);
        } else {
            create_preview = false; // this assumes that all Preview entities have the same PieceType
        }
    }
    // let texture_atlas = texture_atlases.get(&Handle::default()).unwrap();
    let preview_pos = Vec3::new(32. * 12., 32. * 15., 0.);
    if create_preview {
        for (x, y) in get_solid_base(&current_preview)[0].iter() {
            // println!("spawn preview {}", i);
            commands
                .spawn(SpriteSheetComponents {
                    texture_atlas: Handle::default(),
                    transform: Transform::from_scale(4.0).with_translation(
                        Vec3::new((x * 32) as f32, (y * 32) as f32, 1.0) + preview_pos,
                    ),
                    sprite: TextureAtlasSprite {
                        index: get_color(&current_preview) as u32,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Preview)
                .with(current_preview);
        }
    }
}

pub struct TexturedFieldPlugin;

fn scene_save_system(world: &mut World, resources: &mut Resources) {
    let type_registry = resources.get::<TypeRegistry>().unwrap();
    let scene = Scene::from_world(&world, &type_registry.component.read());

    let ron_string = scene
        .serialize_ron(&type_registry.property.read())
        .expect("failed to serialize scene");

    std::fs::write("scene.scn", ron_string).expect("faild to write scene");
}

impl Plugin for TexturedFieldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(init_field_textured.system())
        .add_system(field_update_system_textured.system())
        .add_system(preview_system_textured.system())
        // .add_system(scene_save_system.thread_local_system())

        // sentinel 
       ;
    }
}
