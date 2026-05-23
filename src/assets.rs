use crate::background::Background;
use crate::model::{Model, VertexAttributes};
use crate::shader::{Program, ProgramBuilder, ShaderType};

use std::io::Cursor;
use std::sync::Arc;

use nalgebra_glm as glm;
use glow::Context;
use image::{DynamicImage, ImageReader};
use parry2d::{math::{Pose, Vec2}, shape::Cuboid};
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
	pub player_bounding_box: BoundingBox,
	pub music: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
	pub min: (f32, f32, f32),
	pub max: (f32, f32, f32)
}

impl BoundingBox {
	pub fn generate_mesh(&self) -> Mesh {
		let positions = [
			[self.min.0, self.min.1, self.min.2],
			[self.max.0, self.min.1, self.min.2],
			[self.min.0, self.max.1, self.min.2],
			[self.max.0, self.max.1, self.min.2],
			[self.min.0, self.min.1, self.max.2],
			[self.max.0, self.min.1, self.max.2],
			[self.min.0, self.max.1, self.max.2],
			[self.max.0, self.max.1, self.max.2],
		];

		let normals = std::array::from_fn::<_, 8, _>(|idx| {
			let x = if idx & 1 != 0 { 1.0 } else { -1.0 };
			let y = if idx & 2 != 0 { 1.0 } else { -1.0 };
			let z = if idx & 4 != 0 { 1.0 } else { -1.0 };

			[
				x, 0.0, 0.0,
				0.0, y, 0.0,
				0.0, 0.0, z
			]
		}).concat();

		let texcoords = std::array::from_fn::<_, 24, _>(|_| [0.0, 0.0]).concat();

		// Repeat each vertex 3 times for it to have its own normal
		// (first one points in X direction, second in Y and third in Z)
		let positions = positions.map(|pos| pos.repeat(3)).concat();

		let indices = vec![
			// Left face
			0, 12, 6,
			6, 12, 18,

			// Right face
			3, 9, 15,
			9, 21, 15,

			// Bottom face
			1, 4, 13,
			4, 16, 13,

			// Top face
			7, 19, 10,
			10, 19, 22,

			// Back face
			2, 8, 5,
			5, 8, 11,

			// Front face
			14, 17, 20,
			17, 23, 20
		];

		Mesh {
			positions,
			normals,
			texcoords,
			indices,
			vertex_color: vec![],
			face_arities: vec![],
			normal_indices: vec![],
			texcoord_indices: vec![],
			material_id: None,
		}
	}

	pub fn get_collision_shape(&self) -> (Cuboid, Pose) {
		let w = (self.max.0 - self.min.0) / 2.0;
		let d = (self.max.2 - self.min.2) / 2.0;

		let dx = (self.max.0 + self.min.0) / 2.0;
		let dz = (self.max.2 + self.min.2) / 2.0;

		(
			Cuboid::new(Vec2::new(w, d)),
			Pose::translation(dx, dz)
		)
	}
}

impl Assets {
	pub fn init(gl: Arc<Context>) -> Self {
		let files = assets::Assets::parse_from_data(&std::fs::read("assets.bin").unwrap()).unwrap();

		// Load shaders

		let normal_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, &files.main_vert_program)
			.add_shader(ShaderType::Fragment, &files.main_frag_program)
			.link();

		let rizz_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, &files.main_vert_program)
			.add_shader(ShaderType::Fragment, &files.rizz_frag_program)
			.link();

		let powerup_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, &files.main_vert_program)
			.add_shader(ShaderType::Fragment, &files.powerup_frag_program)
			.link();

		let background_program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, &files.background_vert_program)
			.add_shader(ShaderType::Fragment, &files.background_frag_program)
			.link();

		// Load background effect

		let mut background = Background::new(gl.clone());
		background.register(&background_program, "aPos");

		// Load models

		let player_mesh = load_mesh(&files.player);
		let enemy_mesh = load_mesh(&files.enemy);
		let powerup_hp_mesh = load_mesh(&files.powerup_hp);
		let powerup_energy_mesh = load_mesh(&files.powerup_energy);
		let powerup_speed_mesh = load_mesh(&files.powerup_speed);

		let terrain_tex = load_texture(&files.terrain_tex);
		let player_tex = load_texture(&files.player_tex);
		let enemy_tex = load_texture(&files.enemy_tex);

		let player_scale = 20.0;

		let min_x = player_mesh.positions.chunks(3).map(|pos| pos[0]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;
		let min_y = player_mesh.positions.chunks(3).map(|pos| pos[1]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;
		let min_z = player_mesh.positions.chunks(3).map(|pos| pos[2]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;

		let max_x = player_mesh.positions.chunks(3).map(|pos| pos[0]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;
		let max_y = player_mesh.positions.chunks(3).map(|pos| pos[1]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;
		let max_z = player_mesh.positions.chunks(3).map(|pos| pos[2]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() * player_scale;

		let player_bounding_box = BoundingBox {
			min: (min_x, min_y, min_z),
			max: (max_x, max_y, max_z)
		};

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
			.with_scale(glm::vec3(player_scale, player_scale, player_scale));

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
			enemy,
			player_bounding_box,
			music: files.mod_file
		}
	}
}

