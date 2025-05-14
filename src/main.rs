use std::ops::DerefMut;

use avian2d::prelude::*;
use bevy::prelude::*;

mod maze;

use maze::Maze;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Coin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(Maze::default())
        .insert_resource(Gravity::ZERO)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[cfg(not(target_arch = "wasm32"))]
                        title: "Misty Maze".to_string(),
                        #[cfg(not(target_arch = "wasm32"))]
                        resolution: (1280.0, 720.0).into(),
                        #[cfg(not(target_arch = "wasm32"))]
                        resize_constraints: WindowResizeConstraints {
                            min_width: 300.0,
                            min_height: 300.0,
                            ..default()
                        },

                        resizable: true,

                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#bevy-canvas".to_string()),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_systems(
            Startup,
            (setup, generate_maze, generate_player.after(generate_maze)),
        )
        .add_systems(Update, move_player)
        .run();
}

fn setup(mut commands: Commands, _window: Single<&Window>) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scale: 2.0f32.powi(-6),
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn generate_maze(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut maze: ResMut<Maze>,
) {
    let maze_config = maze::MazeConfig::new(80, 50);
    *maze = maze::Maze::random(maze_config);

    // tiles: 49(horizontal) x 22(vertical)
    // tile size: 16px x 16px
    // padding: 1px x 1px
    let tile_set: Handle<Image> = asset_server.load("colored.png");
    let tile_set_transparent: Handle<Image> = asset_server.load("colored-transparent.png");

    let layout = TextureAtlasLayout::from_grid(uvec2(16, 16), 49, 22, Some(uvec2(1, 1)), None);
    let handle = texture_atlas_layouts.add(layout);

    for x in 0..maze.width {
        for y in 0..maze.height {
            if maze.cells[x][y] == maze::Cell::Blocked {
                let mut is_wall = false;
                const DX: [i32; 8] = [1, 0, -1, 0, 1, 1, -1, -1];
                const DY: [i32; 8] = [0, 1, 0, -1, 1, -1, 1, -1];
                for i in 0..8 {
                    let nx = x as i32 + DX[i];
                    let ny = y as i32 + DY[i];
                    if nx < 0 || nx >= maze.width as i32 || ny < 0 || ny >= maze.height as i32 {
                        continue;
                    }
                    if maze.cells[nx as usize][ny as usize] != maze::Cell::Blocked {
                        is_wall = true;
                        break;
                    }
                }
                if !is_wall {
                    continue;
                }
            }

            let tile = match maze.cells[x][y] {
                maze::Cell::Open => 0,            // (0, 0)
                maze::Cell::Blocked => 16,        // (0, 16)
                maze::Cell::Safe => 14 * 49 + 39, // (14, 39)
            };

            if maze.cells[x][y] == maze::Cell::Blocked {
                commands.spawn((
                    Sprite::from_atlas_image(
                        tile_set.clone(),
                        TextureAtlas {
                            layout: handle.clone(),
                            index: tile,
                        },
                    ),
                    Transform::from_xyz(x as f32 + 0.5, y as f32 + 0.5, 0.0)
                        .with_scale(Vec3::splat(1.0 / 16.0)),
                    RigidBody::Static,
                    Collider::rectangle(16.0, 16.0),
                ));
            } else {
                commands.spawn((
                    Sprite::from_atlas_image(
                        tile_set.clone(),
                        TextureAtlas {
                            layout: handle.clone(),
                            index: tile,
                        },
                    ),
                    Transform::from_xyz(x as f32 + 0.5, y as f32 + 0.5, 0.0)
                        .with_scale(Vec3::splat(1.0 / 16.0)),
                ));
            }
        }
    }

    for &(x, y) in &maze.coins {
        commands.spawn((
            Coin,
            Sprite::from_atlas_image(
                tile_set_transparent.clone(),
                TextureAtlas {
                    layout: handle.clone(),
                    index: 4 * 49 + 22, // (4, 22)
                },
            ),
            Transform::from_xyz(x as f32 + 0.5, y as f32 + 0.5, 0.0)
                .with_scale(Vec3::splat(1.0 / 16.0)),
            RigidBody::Static,
            Collider::circle(4.0),
            Sensor,
        ));
    }
}

fn generate_player(
    mut commands: Commands,
    maze: Res<Maze>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            Player,
            Transform::from_xyz(maze.origin.0 as f32 + 0.5, maze.origin.1 as f32 + 0.5, 1.0),
            RigidBody::Dynamic,
            Collider::circle(0.25),
            CollisionEventsEnabled,
            Mesh2d(meshes.add(Circle::new(0.25))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
        ))
        .observe(
            |trigger: Trigger<OnCollisionStart>, mut commands: Commands, coins: Query<&Coin>| {
                let coin = trigger.collider;
                if coins.contains(coin) {
                    info!("Coin collected: {}", coin);
                    commands.entity(coin).despawn();
                }
            },
        );

    camera.translation.x = maze.origin.0 as f32 + 0.5;
    camera.translation.y = maze.origin.1 as f32 + 0.5;
}

#[allow(clippy::type_complexity)]
fn move_player(
    mut camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut player: Single<(&mut Transform, &mut LinearVelocity), (With<Player>, Without<Camera2d>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if direction != Vec2::ZERO {
        direction = direction.normalize();
    }

    let (player_transform, player_velocity) = player.deref_mut();
    player_velocity.0 = direction
        * if keyboard_input.pressed(KeyCode::ShiftLeft) {
            6.0
        } else {
            3.0
        };

    let (camera_transform, camera_projection) = camera.deref_mut();

    camera_transform.translation = player_transform.translation;

    if let Projection::Orthographic(orthographic) = camera_projection.as_mut() {
        let mut scale = orthographic.scale.log2();
        if keyboard_input.pressed(KeyCode::KeyQ) {
            scale -= 1.0 * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            scale += 1.0 * time.delta_secs();
        }
        scale = scale.clamp(-8.0, 0.0);
        orthographic.scale = 2.0f32.powf(scale);
    }
}
