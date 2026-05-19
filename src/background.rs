use std::sync::{Arc, OnceLock};

use glow::*;

use crate::shader::Program;

struct ModelVertices {
	vao: NativeVertexArray,
	vertex_count: usize,
}

pub struct Background {
	gl: Arc<Context>,
	vtx: OnceLock<ModelVertices>,
}

impl Background {
	pub fn new(gl: Arc<Context>) -> Self {
		Self {
			gl,
			vtx: OnceLock::new()
		}
	}

	pub fn register(&mut self, program: &Program, pos_attrib: &str) {
		let vao;

		let vertices: [f32; _] = [
			-1.0,  1.0,
			-1.0, -1.0,
			1.0, -1.0,

			-1.0,  1.0,
			1.0, -1.0,
			1.0,  1.0
		];

		unsafe {
			vao = self.gl.create_named_vertex_array().unwrap();

			let vbo = self.gl.create_named_buffer().unwrap();

			let position = self
				.gl
				.get_attrib_location(program.program, pos_attrib)
				.unwrap();

			self.gl
				.vertex_array_attrib_format_f32(vao, position, 2, FLOAT, false, 0);
			self.gl.vertex_array_attrib_binding_f32(vao, position, 0);
			self.gl.enable_vertex_array_attrib(vao, position);

			self.gl.named_buffer_data_u8_slice(
				vbo,
				bytemuck::cast_slice(&vertices),
				STATIC_DRAW,
			);

			self.gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, 8);
		}

		let _ = self.vtx.set(ModelVertices {
			vao,
			vertex_count: 6,
		});
	}

	pub fn draw(&self, program: &Program) {
		program.activate();

		unsafe {
			if let Some(vtx) = self.vtx.get() {
				self.gl.disable(DEPTH_TEST);
				self.gl.bind_vertex_array(Some(vtx.vao));
				self.gl.draw_arrays(TRIANGLES, 0, vtx.vertex_count as i32);
				self.gl.enable(DEPTH_TEST);
			}
		}
	}
}

