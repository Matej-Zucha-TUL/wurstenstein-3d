#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Assets {
	pub main_frag_program: String,
	pub main_vert_program: String,
	pub rizz_frag_program: String,
	pub powerup_frag_program: String,
	pub background_vert_program: String,
	pub background_frag_program: String,
	pub player: Vec<u8>,
	pub player_tex: Vec<u8>,
	pub enemy: Vec<u8>,
	pub enemy_tex: Vec<u8>,
	pub terrain_tex: Vec<u8>,
	pub powerup_hp: Vec<u8>,
	pub powerup_energy: Vec<u8>,
	pub powerup_speed: Vec<u8>,
	pub mod_file: Vec<u8>
}

