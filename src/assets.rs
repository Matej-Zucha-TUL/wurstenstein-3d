use crate::{background::Background, model::{Model, VertexAttributes}, shader::{Program, ProgramBuilder, ShaderType}};

use std::{io::Cursor, sync::Arc};

use nalgebra_glm as glm;
use glow::Context;
use image::{DynamicImage, ImageReader};
use tobj::Mesh;

fn load_mesh(bytes: &[u8]) -> Mesh {
	let mut model_data = Cursor::new(bytes);
	let (model, _material) =
		tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
			Err(tobj::LoadError::ReadError)
		})
		.unwrap();
	let model = model.into_iter().next().unwrap();
	model.mesh
}

fn load_texture(bytes: &[u8]) -> DynamicImage {
	ImageReader::new(Cursor::new(bytes))
		.with_guessed_format()
		.unwrap()
		.decode()
		.unwrap()
}

pub struct Assets {
	pub normal_program: Program,
	pub rizz_program: Program,
	pub background_program: Program,
	pub powerup_program: Program,
	pub background: Background,
	pub terrain: Model,
	pub player: Model,
	pub enemy: Model,
	pub powerup_hp: Model,
	pub powerup_energy: Model,
	pub powerup_speed: Model,
}

impl Assets {
	pub fn init(gl: Arc<Context>) -> Self {
		// Load shaders

		let normal_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/main.frag"))
			.link();

		let rizz_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/rizz.frag"))
			.link();

		let powerup_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/powerup.frag"))
			.link();

		let background_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/screen.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/starfield.frag"))
			.link();

		// Load background effect

		let mut background = Background::new(gl.clone());
		background.register(&background_program, "aPos");

		// Load models

		let player_mesh = load_mesh(include_bytes!("../assets/objects/pastry/pastry.obj"));
		let enemy_mesh = load_mesh(include_bytes!("../assets/objects/apple/apple.obj"));
		let powerup_hp_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-hp.obj"));
		let powerup_energy_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-energy.obj"));
		let powerup_speed_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-speed.obj"));

		let terrain_tex = load_texture(include_bytes!("../assets/textures/ferris.png"));
		let player_tex = load_texture(include_bytes!("../assets/objects/pastry/pastry_tex.png"));
		let enemy_tex = load_texture(include_bytes!("../assets/objects/apple/apple_tex.png"));

		let vertex_attribs = VertexAttributes {
			position: Some("aPos".into()),
			normal: Some("aNormal".into()),
			texcoord: Some("aTexCoord".into()),
		};

		let terrain = Model::new(gl.clone())
			.with_mesh(&normal_program, crate::playfield::EXAMPLE_MAZE.generate_mesh(), &vertex_attribs)
			.with_texture(&normal_program, terrain_tex, "tex_unit");

		let player = Model::new(gl.clone())
			.with_mesh(&normal_program, player_mesh, &vertex_attribs)
			.with_texture(&normal_program, player_tex, "tex_unit")
			.with_scale(glm::vec3(20.0, 20.0, 20.0));

		let enemy = Model::new(gl.clone())
			.with_mesh(&normal_program, enemy_mesh, &vertex_attribs)
			.with_texture(&normal_program, enemy_tex, "tex_unit")
			.with_scale(glm::vec3(30.0, 30.0, 30.0));

		let powerup_hp = Model::new(gl.clone())
			.with_mesh(&normal_program, powerup_hp_mesh, &vertex_attribs)
			.with_scale(glm::vec3(2.0, 2.0, 2.0));

		let powerup_energy = Model::new(gl.clone())
			.with_mesh(&normal_program, powerup_energy_mesh, &vertex_attribs)
			.with_scale(glm::vec3(2.0, 2.0, 2.0));

		let powerup_speed = Model::new(gl.clone())
			.with_mesh(&normal_program, powerup_speed_mesh, &vertex_attribs)
			.with_scale(glm::vec3(2.0, 2.0, 2.0));

		Self {
			normal_program,
			rizz_program,
			background_program,
			powerup_program,
			background,
			terrain,
			player,
			powerup_hp,
			powerup_energy,
			powerup_speed,
			enemy
		}
	}
}

