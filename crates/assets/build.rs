include!("./assets_struct.rs");

fn main() {
	println!("cargo::rerun-if-changed=files");

	let assets = Assets {
		main_frag_program: include_str!("./files/shaders/frag/main.frag").to_string(),
		main_vert_program: include_str!("./files/shaders/vert/main.vert").to_string(),
		rizz_frag_program: include_str!("./files/shaders/frag/rizz.frag").to_string(),
		background_frag_program: include_str!("./files/shaders/frag/starfield.frag").to_string(),
		background_vert_program: include_str!("./files/shaders/vert/screen.vert").to_string(),
		powerup_frag_program: include_str!("./files/shaders/frag/powerup.frag").to_string(),
		player: include_bytes!("./files/objects/pastry/pastry.obj").to_vec(),
		player_tex: include_bytes!("./files/objects/pastry/pastry.png").to_vec(),
		enemy: include_bytes!("./files/objects/apple/apple.obj").to_vec(),
		enemy_tex: include_bytes!("./files/objects/apple/apple_tex.png").to_vec(),
		terrain_tex: include_bytes!("./files/textures/ferris.png").to_vec(),
		powerup_hp: include_bytes!("./files/objects/powerups/powerup-hp.obj").to_vec(),
		powerup_energy: include_bytes!("./files/objects/powerups/powerup-energy.obj").to_vec(),
		powerup_speed: include_bytes!("./files/objects/powerups/powerup-speed.obj").to_vec(),
		mod_file: include_bytes!("./files/music/space_debris.mod").to_vec()
	};

	let data = wincode::serialize(&assets).unwrap();
	let data = miniz_oxide::deflate::compress_to_vec(&data, 5);

	std::fs::write("../../assets.bin", data).unwrap();
}
