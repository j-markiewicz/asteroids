use std::ops::Range;

use bevy::prelude::*;
use rand::Rng;

use crate::player::Velocity;

pub fn asteroid_systems() -> SystemSet {
	SystemSet::new()
		.label("asteroid systems")
		.with_system(generate_asteroids)
		.with_system(remove_stray_asteroids)
}

pub const ASTEROID_SIZE: Range<f32> = 0.5..3.0;
pub const ASTEROID_Z: Range<f32> = 1.0..10.0;

pub const NUM_ASTEROIDS: usize = 100;

#[derive(Component, Debug)]
pub struct Asteroid {
	pub size: f32,
}

impl Asteroid {
	pub fn new() -> Self {
		Self {
			size: rand::thread_rng().gen_range(ASTEROID_SIZE),
		}
	}
}

pub fn generate_asteroids(
	mut commands: Commands,
	assets: Res<AssetServer>,
	windows: Res<Windows>,
	query: Query<(), With<Asteroid>>,
) {
	if NUM_ASTEROIDS > query.iter().len() {
		let asteroid = Asteroid::new();
		let (width, height) = {
			let window = windows.primary();
			(window.width(), window.height())
		};

		let mut rng = rand::thread_rng();
		let x = rng.gen_range(-(width / 2.0)..(width / 2.0));
		let x = x.signum().mul_add(width / 1.5, x);
		let y = rng.gen_range(-(height / 2.0)..(height / 2.0));
		let y = y.signum().mul_add(height / 1.5, y);

		commands.spawn((
			SpriteBundle {
				texture: assets.load("asteroid.png"),
				transform: Transform {
					translation: Vec3 {
						x,
						y,
						z: rng.gen_range(ASTEROID_Z),
					},
					scale: Vec3 {
						x: asteroid.size,
						y: asteroid.size,
						..default()
					},
					..default()
				},
				..default()
			},
			asteroid,
			Velocity::random(),
		));
	}
}

pub fn initialize_asteroids(
	mut commands: Commands,
	assets: Res<AssetServer>,
	windows: Res<Windows>,
) {
	let (width, height) = {
		let window = windows.primary();
		(window.width(), window.height())
	};

	let mut rng = rand::thread_rng();

	for _ in 0..(NUM_ASTEROIDS / 4) {
		let asteroid = Asteroid::new();

		let x = rng.gen_range(-(width / 2.0)..(width / 2.0));
		let x = x.signum().mul_add(width / 4.0, x);
		let y = rng.gen_range(-(height / 2.0)..(height / 2.0));
		let y = y.signum().mul_add(height / 4.0, y);

		commands.spawn((
			SpriteBundle {
				texture: assets.load("asteroid.png"),
				transform: Transform {
					translation: Vec3 {
						x,
						y,
						z: rng.gen_range(ASTEROID_Z),
					},
					scale: Vec3 {
						x: asteroid.size,
						y: asteroid.size,
						..default()
					},
					..default()
				},
				..default()
			},
			asteroid,
			Velocity::random(),
		));
	}
}

pub fn remove_stray_asteroids(
	mut commands: Commands,
	windows: Res<Windows>,
	query: Query<(Entity, &Transform), With<Asteroid>>,
) {
	let (width, height) = {
		let window = windows.primary();
		(window.width(), window.height())
	};

	for (asteroid, trans) in query.iter() {
		if trans.translation.x > width
			|| trans.translation.x < -width
			|| trans.translation.y > height
			|| trans.translation.y < -height
		{
			commands.entity(asteroid).despawn();
		}
	}
}
