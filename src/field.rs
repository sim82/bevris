use super::{get_color, get_solid_base, PieceBag, PieceType, Preview};

use bevy::prelude::*;

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
                .with(Field { x, y });
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
    let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, 10, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

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
                .with(Field { x, y });
        }
    }
}

fn field_update_system_textured(
    playfield: Res<Playfield>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&Field, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (field, mut sprite, texture_atlas_handle) in &mut query.iter() {
        let texture_atlas = texture_atlases.get(&texture_atlas_handle).unwrap();
        sprite.index = (playfield.field[field.y as usize][field.x as usize] as usize
            % texture_atlas.textures.len()) as u32;
    }
}

pub struct TexturedFieldPlugin;

impl Plugin for TexturedFieldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(init_field_textured.system())
        .add_system(field_update_system_textured.system())
        // sentinel 
       ;
    }
}
