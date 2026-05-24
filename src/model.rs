use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use glow::*;
use image::DynamicImage;
use log::*;
use tobj::Mesh;
use nalgebra_glm as glm;

use crate::shader::Program;

struct ModelVertices {
	vao: NativeVertexArray,
	vertex_count: usize,
}

struct ModelTexture {
	tex: NativeTexture,
	sampler: String
}

pub struct VertexAttributes {
	pub position: Option<Cow<'static, str>>,
	pub normal: Option<Cow<'static, str>>,
	pub texcoord: Option<Cow<'static, str>>,
}

#[derive(Clone)]
pub struct Transform {
	pub position: glm::Vec3,
	pub scale: glm::Vec3,
	/// Yaw, Pitch, Roll
	pub rotation: glm::Vec3
}

impl Transform {
	pub fn origin() -> Self {
		Self {
			position: glm::vec3(0.0, 0.0, 0.0),
			scale: glm::vec3(1.0, 1.0, 1.0),
			rotation: glm::vec3(0.0, 0.0, 0.0)
		}
	}

	pub fn with_position(mut self, position: glm::Vec3) -> Self {
		self.position = position;
		self
	}

	pub fn with_scale(mut self, scale: glm::Vec3) -> Self {
		self.scale = scale;
		self
	}

	pub fn with_rotation(mut self, rotation: glm::Vec3) -> Self {
		self.rotation = rotation;
		self
	}
}

pub struct Model {
	gl: Arc<Context>,
	vtx: OnceLock<ModelVertices>,
	tex: Vec<ModelTexture>,
	scale: glm::Vec3
}

impl Model {
	pub fn new(gl: Arc<Context>) -> Self {
		Self {
			gl,
			vtx: OnceLock::new(),
			tex: Vec::new(),
			scale: glm::vec3(1.0, 1.0, 1.0)
		}
	}

	pub fn add_mesh(&mut self, program: &Program, mut mesh: Mesh, attribs: &VertexAttributes) {
		if mesh.texcoords.len() == 0 {
			mesh.texcoords.resize(mesh.positions.len() / 3 * 2, 0.0);
		}
		assert_eq!(mesh.positions.len() / 3, mesh.normals.len() / 3);
		assert_eq!(mesh.positions.len() / 3, mesh.texcoords.len() / 2);

		let merged_vertices = mesh
			.positions
			.chunks(3)
			.zip(mesh.normals.chunks(3))
			.zip(mesh.texcoords.chunks(2))
			.flat_map(|((p, n), t)| [p[0], p[1], p[2], n[0], n[1], n[2], t[0], t[1]])
			.collect::<Vec<_>>();

		let vao;

		unsafe {
			vao = self.gl.create_named_vertex_array().unwrap();

			let vbo = self.gl.create_named_buffer().unwrap();
			let ebo = self.gl.create_named_buffer().unwrap();

			if let Some(position_attr) = &attribs.position {
				let position = self
					.gl
					.get_attrib_location(program.program, position_attr.as_ref())
					.unwrap();

				self.gl
					.vertex_array_attrib_format_f32(vao, position, 3, FLOAT, false, 0);
				self.gl.vertex_array_attrib_binding_f32(vao, position, 0);
				self.gl.enable_vertex_array_attrib(vao, position);
			}

			if let Some(normal_attr) = &attribs.normal {
				let normal = self
					.gl
					.get_attrib_location(program.program, normal_attr.as_ref())
					.unwrap();

				self.gl
					.vertex_array_attrib_format_f32(vao, normal, 3, FLOAT, false, 12);
				self.gl.vertex_array_attrib_binding_f32(vao, normal, 0);
				self.gl.enable_vertex_array_attrib(vao, normal);
			}

			if let Some(texcoord_attr) = &attribs.texcoord {
				let texcoords = self
					.gl
					.get_attrib_location(program.program, texcoord_attr.as_ref())
					.unwrap();

				self.gl
					.vertex_array_attrib_format_f32(vao, texcoords, 2, FLOAT, false, 24);
				self.gl.vertex_array_attrib_binding_f32(vao, texcoords, 0);
				self.gl.enable_vertex_array_attrib(vao, texcoords);
			}

			self.gl.named_buffer_data_u8_slice(
				vbo,
				bytemuck::cast_slice(&merged_vertices),
				STATIC_DRAW,
			);
			self.gl.named_buffer_data_u8_slice(
				ebo,
				bytemuck::cast_slice(&mesh.indices),
				STATIC_DRAW,
			);

			self.gl.vertex_array_vertex_buffer(vao, 0, Some(vbo), 0, 32);
			self.gl.vertex_array_element_buffer(vao, Some(ebo));
		}

		let _ = self.vtx.set(ModelVertices {
			vao,
			vertex_count: mesh.indices.len(),
		});
	}

	pub fn add_texture(&mut self, program: &Program, image: DynamicImage, sampler_attrib: &str) {
		let width = image.width() as i32;
		let height = image.height() as i32;
		let raw_img = image.flipv().into_rgb8().into_raw();

		assert_eq!(width * height * 3, raw_img.len() as i32);

		let tex;
		let sampler = sampler_attrib.to_string();

		unsafe {
			tex = self.gl.create_named_texture(TEXTURE_2D).unwrap();
			self.gl.texture_storage_2d(tex, 1, RGB8, width, height);
			self.gl.texture_sub_image_2d(
				tex,
				0,
				0,
				0,
				width,
				height,
				RGB,
				UNSIGNED_BYTE,
				PixelUnpackData::Slice(Some(&raw_img)),
			);
			self.gl
				.texture_parameter_i32(tex, TEXTURE_MIN_FILTER, NEAREST as i32);
			self.gl
				.texture_parameter_i32(tex, TEXTURE_MAG_FILTER, LINEAR as i32);
		}

		self.tex.push(ModelTexture { tex, sampler });
	}

	pub fn with_mesh(mut self, program: &Program, mesh: Mesh, attribs: &VertexAttributes) -> Self {
		self.add_mesh(program, mesh, attribs);
		self
	}

	pub fn with_texture(mut self, program: &Program, image: DynamicImage, sampler_attrib: &str) -> Self {
		self.add_texture(program, image, sampler_attrib);
		self
	}

	pub fn with_scale(mut self, scale: glm::Vec3) -> Self {
		self.scale = scale;
		self
	}

	pub fn draw(&self, transform: &Transform, program: &Program, model_attrib: &str) {
		program.activate();

		let model_mtx = glm::translate(&glm::Mat4::identity(), &transform.position);
		let model_mtx = glm::scale(&model_mtx, &transform.scale.component_mul(&self.scale));
		let model_mtx = glm::rotate_z(&model_mtx, transform.rotation[2]);
		let model_mtx = glm::rotate_y(&model_mtx, transform.rotation[0]);
		let model_mtx = glm::rotate_x(&model_mtx, transform.rotation[1]);

		program.set_uniform_matrix_f32_4(model_attrib, model_mtx.as_slice().try_into().unwrap());

		unsafe {
			for (tex_unit, tex) in self.tex.iter().enumerate() {
				program.set_uniform_i32(&tex.sampler, tex_unit as i32);
				self.gl.bind_texture_unit(tex_unit as u32, Some(tex.tex));
			}

			if let Some(vtx) = self.vtx.get() {
				self.gl.bind_vertex_array(Some(vtx.vao));
				self.gl
					.draw_elements(TRIANGLES, vtx.vertex_count as i32, UNSIGNED_INT, 0);
			}
		}
	}
}
