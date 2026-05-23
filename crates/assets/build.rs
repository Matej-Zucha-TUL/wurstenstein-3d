include!("./assets_struct.rs");

fn main() {
	println!("cargo::rerun-if-changed=files");

	let assets = Assets {
		shader_programs: ShaderPrograms {
			main_frag: include_str!("./files/shaders/frag/main.frag").to_string(),
			main_vert: include_str!("./files/shaders/vert/main.vert").to_string(),
			rizz_frag: include_str!("./files/shaders/frag/rizz.frag").to_string(),
			background_frag: include_str!("./files/shaders/frag/starfield.frag").to_string(),
			background_vert: include_str!("./files/shaders/vert/screen.vert").to_string(),
			powerup_frag: include_str!("./files/shaders/frag/powerup.frag").to_string(),
		},
		models: Models {
			player: include_bytes!("./files/objects/pastry/pastry.obj").to_vec(),
			enemy: include_bytes!("./files/objects/apple/apple.obj").to_vec(),
			powerup_hp: include_bytes!("./files/objects/powerups/powerup-hp.obj").to_vec(),
			powerup_energy: include_bytes!("./files/objects/powerups/powerup-energy.obj").to_vec(),
			powerup_speed: include_bytes!("./files/objects/powerups/powerup-speed.obj").to_vec(),
		},
		textures: Textures {
			player: include_bytes!("./files/objects/pastry/pastry.png").to_vec(),
			enemy: include_bytes!("./files/objects/apple/apple_tex.png").to_vec(),
			terrain: include_bytes!("./files/textures/ferris.png").to_vec(),
		},
		music: Music {
			mod_file: include_bytes!("./files/music/space_debris.mod").to_vec(),
		},
		sounds: Sounds {
			player_jump: include_bytes!("./files/sounds/sfx_movement_jump14.wav").to_vec(),
			player_explosion: include_bytes!("./files/sounds/sfx_exp_medium1.wav").to_vec(),
			player_death: include_bytes!("./files/sounds/sfx_deathscream_human11.wav").to_vec(),
			player_shoot: include_bytes!("./files/sounds/sfx_weapon_shotgun3.wav").to_vec(),
			enemy_hit: include_bytes!("./files/sounds/sfx_weapon_shotgun2.wav").to_vec(),
			enemy_explosion: include_bytes!("./files/sounds/sfx_exp_medium2.wav").to_vec(),
			enemy_death: include_bytes!("./files/sounds/sfx_deathscream_alien4.wav").to_vec(),
			enemy_shoot: include_bytes!("./files/sounds/sfx_weapon_shotgun2.wav").to_vec(),
			powerup_hp_pickup: include_bytes!("./files/sounds/sfx_sounds_powerup6.wav").to_vec(),
			powerup_energy_pickup: include_bytes!("./files/sounds/sfx_sounds_powerup9.wav").to_vec(),
			powerup_speed_pickup: include_bytes!("./files/sounds/sfx_sounds_powerup16.wav").to_vec(),
		}
	};

	let data = wincode::serialize(&assets).unwrap();
	let data = miniz_oxide::deflate::compress_to_vec(&data, 5);

	std::fs::write("../../assets.bin", data).unwrap();
}
