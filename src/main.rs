#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::needless_pass_by_value)] // A bunch of Bevy things require this
#![allow(clippy::module_name_repetitions)]

use std::sync::{
	atomic::{AtomicU64, Ordering},
	Mutex,
};

use bevy::{
	prelude::*,
	render::settings::{WgpuSettings, WgpuSettingsPriority},
};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use player::{FuelTimer, Score};

mod asteroids;
mod player;
mod util;

#[bevy_main]
pub fn main() {
	#[cfg(target_arch = "wasm32")]
	console_error_panic_hook::set_once();

	App::new()
		.insert_resource(WgpuSettings {
			priority: WgpuSettingsPriority::Compatibility,
			..default()
		})
		.insert_resource(ClearColor(Color::BLACK))
		.insert_resource(Msaa { samples: 4 })
		.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
				.set(ImagePlugin::default_nearest()),
		)
		.add_plugin(OverlayPlugin {
			font_size: 16.0,
			..default()
		})
		.add_startup_system(setup)
		.add_startup_system(player::init_player)
		.add_startup_system(asteroids::initialize_asteroids)
		.add_system(bevy::window::close_on_esc)
		.add_system(print_info_text)
		.add_system_set(player::movement_systems())
		.add_system_set(player::player_systems())
		.add_system_set(player::fuel_systems())
		.add_system_set(asteroids::asteroid_systems())
		.run();
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>) {
	let window = windows.primary_mut();
	window.set_title("Asteroids".to_string());
	window.set_maximized(true);
	window.set_resizable(false);

	commands.spawn(Camera2dBundle::default());

	commands.insert_resource(Score::default());
	commands.insert_resource(FuelTimer::default());
}

#[allow(clippy::cast_precision_loss)]
fn print_info_text(time: Res<Time>) {
	const WINDOW_LEN: usize = 64;
	static WINDOW: Mutex<Vec<f32>> = Mutex::new(Vec::new());
	static FRAMES: AtomicU64 = AtomicU64::new(0);

	let td = time.delta_seconds();

	let tda: f32 = {
		let mut window = WINDOW.lock().unwrap();
		window.push(td);

		if window.len() > WINDOW_LEN {
			window.remove(0);
		}

		window.iter().copied().sum::<f32>() / (window.len() as f32)
	};

	screen_print!(
		col: Color::CYAN,
		"fps: {:.0} ({:.3}ms, {:.3}ms) frames: {}",
		tda.recip(),
		tda * 1000.0,
		td * 1000.0,
		FRAMES.fetch_add(1, Ordering::SeqCst)
	);

	let current_time = time.startup().elapsed().as_secs_f32();
	screen_print!("time: {current_time:.3}");
}
