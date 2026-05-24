#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct ShaderPrograms {
	pub main_frag: String,
	pub main_vert: String,
	pub rizz_frag: String,
	pub powerup_frag: String,
	pub background_vert: String,
	pub background_frag: String,
	pub explosion_vert: String,
	pub explosion_frag: String,
}

#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Models {
	pub pastry: Vec<u8>,
	pub sausage_bullet: Vec<u8>,
	pub sausage_tip: Vec<u8>,
	pub enemy: Vec<u8>,
	pub powerup_hp: Vec<u8>,
	pub powerup_energy: Vec<u8>,
	pub powerup_speed: Vec<u8>,
}

#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Textures {
	pub player: Vec<u8>,
	pub enemy: Vec<u8>,
	pub terrain: Vec<u8>,
}

#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Music {
	pub space_debris: Vec<u8>,
	pub humntrgt: Vec<u8>,
	pub brewery: Vec<u8>,
}

#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Sounds {
	pub player_jump: Vec<u8>,
	pub player_explosion: Vec<u8>,
	pub player_death: Vec<u8>,
	pub player_shoot: Vec<u8>,
	pub enemy_hit: Vec<u8>,
	pub enemy_death: Vec<u8>,
	pub enemy_explosion: Vec<u8>,
	pub enemy_shoot: Vec<u8>,
	pub powerup_hp_pickup: Vec<u8>,
	pub powerup_energy_pickup: Vec<u8>,
	pub powerup_speed_pickup: Vec<u8>,
}

#[derive(wincode::SchemaWrite, wincode::SchemaRead)]
pub struct Assets {
	pub shader_programs: ShaderPrograms,
	pub models: Models,
	pub textures: Textures,
	pub music: Music,
	pub sounds: Sounds,
}

