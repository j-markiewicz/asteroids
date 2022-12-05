use std::{f32::consts::PI, ops::Range};

use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;
use rand::Rng;

use crate::util::abs_max;

pub fn player_systems() -> SystemSet {
    SystemSet::new()
        .label("player systems")
        .with_system(count_score)
        .with_system(do_player_movement)
        .with_system(player_sprite)
        .with_system(edge_bounce)
}

pub fn fuel_systems() -> SystemSet {
    SystemSet::new()
        .label("fuel can systems")
        .with_system(generate_fuel)
        .with_system(fuel_collision)
        .with_system(remove_stray_fuel)
}

pub fn movement_systems() -> SystemSet {
    SystemSet::new()
        .label("generic movement systems")
        .with_system(process_accelleration)
        .with_system(process_movement)
}

pub const PLAYER_Z: f32 = 100.0;
pub const POINTS_PER_SECOND: f32 = 2.0;

pub const INITIAL_FUEL: f32 = 60.0;
pub const FUEL_INTERVAL: f32 = 5.0;
pub const FUEL_PER_CAN: f32 = 10.0;
pub const FUEL_CAN_Z: f32 = 90.0;
pub const FUEL_CAN_SCALE: f32 = 1.5;

pub const LATERAL_ACCELLERATION: f32 = 50.0;
pub const ANGULAR_ACCELLERATION: f32 = PI / 4.0;

pub const LINEAR_DRAG: f32 = 0.001; //0.004;
pub const ANGULAR_DRAG: f32 = PI / 16.0; //PI / 4.0;

pub const INITIAL_LINEAR_VELOCITY: Range<f32> = -100.0..100.0;
pub const INITIAL_ANGULAR_VELOCITY: Range<f32> = -PI..PI;

const SPRITE_INDEX_IDLE: usize = 0;
const SPRITE_INDEX_THRUSTING: usize = 1;

const GAMEPAD_DEADZONE: f32 = 0.1;

#[derive(Component, Debug)]
pub struct Player {
    pub fuel: f32,
    pub is_thrusting: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            fuel: INITIAL_FUEL,
            is_thrusting: false,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Accelleration {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Component, Default, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

impl Velocity {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            x: rng.gen_range(INITIAL_LINEAR_VELOCITY),
            y: rng.gen_range(INITIAL_LINEAR_VELOCITY),
            r: rng.gen_range(INITIAL_ANGULAR_VELOCITY),
        }
    }
}

#[derive(Component)]
pub struct FuelCan;

#[derive(Resource, Debug)]
pub struct Score {
    points: u64,
    next_point: Timer,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            points: 0,
            next_point: Timer::from_seconds(1.0 / POINTS_PER_SECOND, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct FuelTimer(pub Timer);

impl Default for FuelTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(FUEL_INTERVAL, TimerMode::Repeating))
    }
}

pub fn init_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_atlas = texture_atlases.add(TextureAtlas::from_grid(
        assets.load("rocket.png"),
        [32.0, 32.0].into(),
        1,
        2,
        None,
        None,
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas,
            transform: Transform {
                translation: Vec3 {
                    z: PLAYER_Z,
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        Player::default(),
        Velocity::default(),
        Accelleration::default(),
    ));
}

pub fn count_score(time: Res<Time>, mut score: ResMut<Score>) {
    if score.next_point.tick(time.delta()).just_finished() {
        score.points += 1;
    }

    screen_print!(col: Color::BLUE, "SCORE: {}", score.points);
}

pub fn do_player_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut query: Query<(&mut Player, &Transform, &mut Accelleration)>,
) {
    let mut gamepad_thrust = 0.0;
    let mut gamepad_rotation = 0.0;

    for gamepad in gamepads.iter() {
        // thrust
        if button_inputs.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadUp,
        }) {
            gamepad_thrust = abs_max(gamepad_thrust, 1.0);
        }

        if let Some(val) = axes.get(GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        }) {
            gamepad_thrust = abs_max(gamepad_thrust, val);
        }

        if let Some(val) = button_axes.get(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::RightTrigger2,
        }) {
            gamepad_thrust = abs_max(gamepad_thrust, val);
        }

        if gamepad_thrust <= 0.0 {
            gamepad_thrust = 0.0;
        }

        // rotation
        if button_inputs.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadRight,
        }) {
            gamepad_rotation = abs_max(gamepad_rotation, 1.0);
        } else if button_inputs.pressed(GamepadButton {
            gamepad,
            button_type: GamepadButtonType::DPadLeft,
        }) {
            gamepad_rotation = abs_max(gamepad_rotation, -1.0);
        }

        if let Some(val) = axes.get(GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        }) {
            gamepad_rotation = abs_max(gamepad_rotation, val);
        }
    }

    for (mut player, trans, mut accel) in query.iter_mut() {
        if gamepad_thrust > GAMEPAD_DEADZONE {
            player.is_thrusting = true;
            player.fuel -= gamepad_thrust * time.delta_seconds();

            let rotation = -trans.rotation.to_euler(EulerRot::ZYX).0;

            accel.x = rotation.sin() * LATERAL_ACCELLERATION * gamepad_thrust;
            accel.y = rotation.cos() * LATERAL_ACCELLERATION * gamepad_thrust;
        } else if player.fuel > 0.0 && (keyboard_input.any_pressed([KeyCode::W, KeyCode::Up])) {
            player.is_thrusting = true;
            player.fuel -= time.delta_seconds();

            let rotation = -trans.rotation.to_euler(EulerRot::ZYX).0;

            accel.x = rotation.sin() * LATERAL_ACCELLERATION;
            accel.y = rotation.cos() * LATERAL_ACCELLERATION;
        } else {
            player.is_thrusting = false;
            accel.x = 0.0;
            accel.y = 0.0;
        }

        if !(-GAMEPAD_DEADZONE..GAMEPAD_DEADZONE).contains(&gamepad_rotation) {
            accel.r = -gamepad_rotation * ANGULAR_ACCELLERATION;
        } else if keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]) {
            accel.r = ANGULAR_ACCELLERATION;
        } else if keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]) {
            accel.r = -ANGULAR_ACCELLERATION;
        } else {
            accel.r = 0.0;
        }

        screen_print!(
            "fuel: {:.3}% ({:.3}),\ntrans: {trans:?},\naccel: {accel:?},\nthrust: {:?}",
            player.fuel / INITIAL_FUEL * 100.0,
            player.fuel,
            player.is_thrusting
        );
    }
}

pub fn player_sprite(mut query: Query<(&Player, &mut TextureAtlasSprite)>) {
    for (player, mut sprite) in query.iter_mut() {
        if player.is_thrusting {
            sprite.index = SPRITE_INDEX_THRUSTING;
        } else {
            sprite.index = SPRITE_INDEX_IDLE;
        }
    }
}

pub fn process_accelleration(time: Res<Time>, mut query: Query<(&mut Velocity, &Accelleration)>) {
    for (mut vel, accel) in query.iter_mut() {
        // accelleration -> velocity
        vel.x += accel.x * time.delta_seconds();
        vel.y += accel.y * time.delta_seconds();
        vel.r = (vel.r + accel.r * time.delta_seconds()) % (2.0 * PI);

        // drag -> velocity
        vel.x -= vel.x.signum() * vel.x * vel.x * LINEAR_DRAG * time.delta_seconds();
        vel.y -= vel.y.signum() * vel.y * vel.y * LINEAR_DRAG * time.delta_seconds();
        vel.r -= vel.r.signum() * vel.r * vel.r * ANGULAR_DRAG * time.delta_seconds();
    }
}

pub fn process_movement(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut trans, vel) in query.iter_mut() {
        // velocity -> location
        trans.translation.x += vel.x * time.delta_seconds();
        trans.translation.y += vel.y * time.delta_seconds();
        trans.rotate_z(vel.r * time.delta_seconds());
    }
}

pub fn edge_bounce(
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let (width, height) = {
        let window = windows.primary();
        (window.width() / 2.0, window.height() / 2.0)
    };

    for (mut trans, mut vel) in query.iter_mut() {
        if trans.translation.x > width || trans.translation.x < -width {
            vel.x = -vel.x;
            vel.r = rand::thread_rng().gen_range(-(2.0 * PI)..(2.0 * PI));
            trans.translation.x = trans.translation.x.signum() * (width - 1.0);
        }

        if trans.translation.y > height || trans.translation.y < -height {
            vel.y = -vel.y;
            vel.r = rand::thread_rng().gen_range(-(2.0 * PI)..(2.0 * PI));
            trans.translation.y = trans.translation.y.signum() * (height - 1.0);
        }
    }
}

pub fn generate_fuel(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FuelTimer>,
    assets: Res<AssetServer>,
    windows: Res<Windows>,
) {
    timer.0.tick(time.delta());

    screen_print!("next fuel in {:.3}s", timer.0.remaining_secs());

    if timer.0.just_finished() {
        let (width, height) = {
            let window = windows.primary();
            (window.width(), window.height())
        };

        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-(width / 2.0)..(width / 2.0));
        let x = x + x.signum() * (width / 3.0);
        let y = rng.gen_range(-(height / 2.0)..(height / 2.0));
        let y = y + y.signum() * (height / 3.0);

        commands.spawn((
            SpriteBundle {
                texture: assets.load("fuel-can.png"),
                transform: Transform {
                    translation: Vec3 {
                        x,
                        y,
                        z: FUEL_CAN_Z,
                    },
                    scale: Vec3 {
                        x: FUEL_CAN_SCALE,
                        y: FUEL_CAN_SCALE,
                        z: FUEL_CAN_SCALE,
                    },
                    ..default()
                },
                ..default()
            },
            FuelCan,
            Velocity::random(),
        ));

        screen_print!(sec: 2.0, col: Color::ORANGE_RED, "new fuel at {x}, {y}");
    }
}

pub fn fuel_collision(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &Transform)>,
    mut fuel_query: Query<(Entity, &Transform), With<FuelCan>>,
) {
    for (mut player, trans) in player_query.iter_mut() {
        for (fuel, fuel_trans) in fuel_query.iter_mut() {
            // TODO: do proper collision
            if fuel_trans.translation.abs_diff_eq(trans.translation, 32.0) {
                commands.entity(fuel).despawn();
                player.fuel += FUEL_PER_CAN;
                screen_print!(sec: 2.0, col: Color::ORANGE_RED, "refueled for {FUEL_PER_CAN} fuel");
            }
        }
    }
}

pub fn remove_stray_fuel(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Transform), With<FuelCan>>,
) {
    let (width, height) = {
        let window = windows.primary();
        (window.width(), window.height())
    };

    for (fuel, trans) in query.iter() {
        if trans.translation.x > width
            || trans.translation.x < -width
            || trans.translation.y > height
            || trans.translation.y < -height
        {
            commands.entity(fuel).despawn()
        }
    }
}
