use glow::*;
use nalgebra_glm as glm;

use crate::assets::Assets;

use std::sync::Arc;

struct ExplosionData {
	gl: Arc<Context>,
	vao: NativeVertexArray,
	point_count: usize,
}

impl ExplosionData {
	pub fn init(gl: Arc<Context>, assets: &Assets, point_count: usize) -> Self {
		let vao;

		let seeds = (0..point_count).map(|x| x as f32 / point_count as f32).collect::<Vec<_>>();

		unsafe {
			vao = gl.create_named_vertex_array().unwrap();

			let vbo = gl.create_named_buffer().unwrap();

			let position = gl
				.get_attrib_location(assets.explosion_program.program, "seed")
				.unwrap();

			gl.vertex_array_attrib_format_f32(vao, position, 1, FLOAT, false, 0);
			gl.vertex_array_attrib_binding_f32(vao, position, 0);
			gl.enable_vertex_array_attrib(vao, position);

			gl.named_buffer_data_u8_slice(
				vbo,
				bytemuck::cast_slice(&seeds),
				STATIC_DRAW,
			);

			gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, 4);
		}

		Self {
			gl,
			vao,
			point_count
		}
	}

	pub fn draw(&self, assets: &Assets, position: [f32; 3]) {
		let model_mtx = glm::translate(&glm::Mat4::identity(), &position.into());

		assets.explosion_program.set_uniform_matrix_f32_4("model", model_mtx.as_slice().try_into().unwrap());

		assets.explosion_program.activate();

		unsafe {
			self.gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
			self.gl.enable(BLEND);
			self.gl.enable(PROGRAM_POINT_SIZE);
			self.gl.bind_vertex_array(Some(self.vao));
			self.gl.draw_arrays(POINTS, 0, self.point_count as i32);
		}
	}
}

#[derive(PartialEq, Eq)]
enum ExplosionState {
	Running,
	Ended
}

pub struct Explosion {
	base_position: [f32; 3],
	state: ExplosionState,
	timer: f32
}

impl Explosion {
	fn update(&mut self, dt: f32) {
		match self.state {
			ExplosionState::Running => {
				self.timer += dt;
				if self.timer >= 1.0 {
					self.state = ExplosionState::Ended;
				}
			},
			ExplosionState::Ended => {}
		}
	}
}

pub struct ExplosionManager {
	explosions: Vec<Option<Explosion>>,
	data: ExplosionData
}

impl ExplosionManager {
	pub fn new(gl: Arc<Context>, assets: &Assets) -> Self {
		Self {
			explosions: vec![],
			data: ExplosionData::init(gl, assets, 256)
		}
	}

	pub fn update(&mut self, dt: f32) {
		for explosion in &mut self.explosions {
			let Some(expl) = explosion else { continue };

			expl.update(dt);

			if expl.state == ExplosionState::Ended {
				*explosion = None;
			}
		}
	}

	pub fn add_explosion(&mut self, position: [f32; 3]) {
		// Try to find an existing slot

		let new_explosion = Explosion {
			base_position: position,
			state: ExplosionState::Running,
			timer: 0.0,
		};

		for explosion in &mut self.explosions {
			if explosion.is_some() { continue }

			*explosion = Some(new_explosion);
			return
		}

		self.explosions.push(Some(new_explosion));
	}

	pub fn render(&self, assets: &Assets) {
		for explosion in &self.explosions {
			let Some(expl) = explosion else { continue };

			assets.explosion_program.set_uniform_f32("time", expl.timer);
			self.data.draw(assets, expl.base_position);
		}
	}
}

