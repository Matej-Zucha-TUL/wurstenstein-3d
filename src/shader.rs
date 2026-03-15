use glow::{Context, HasContext, NativeProgram, NativeShader};

use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub enum ShaderType {
	Vertex,
	Fragment
}

pub struct ProgramBuilder {
	gl: Arc<Context>,
	program: NativeProgram,
	shaders: Vec<NativeShader>
}

impl ProgramBuilder {
	pub fn new(gl: Arc<Context>) -> Self {
		let program = unsafe { gl.create_program().unwrap() };

		Self {
			gl,
			program,
			shaders: Vec::new()
		}
	}

	pub fn add_shader(mut self, ty: ShaderType, source: &str) -> Self {
		let ty = match ty {
			ShaderType::Vertex => glow::VERTEX_SHADER,
			ShaderType::Fragment => glow::FRAGMENT_SHADER
		};

		let shader = unsafe {
			let shader = self.gl.create_shader(ty).unwrap();

			self.gl.shader_source(shader, source);
			self.gl.compile_shader(shader);

			if !self.gl.get_shader_compile_status(shader) {
				log::warn!("Shader failed to compile: {}", self.gl.get_shader_info_log(shader));
				return self
			}

			self.gl.attach_shader(self.program, shader);

			shader
		};

		self.shaders.push(shader);

		self
	}

	pub fn link(self) -> Program {
		unsafe {
			self.gl.link_program(self.program);
			if !self.gl.get_program_link_status(self.program) {
				panic!("{}", self.gl.get_program_info_log(self.program));
			}

			for shader in self.shaders {
				self.gl.detach_shader(self.program, shader);
				self.gl.delete_shader(shader);
			}
		}

		Program {
			gl: self.gl,
			program: self.program
		}
	}
}

macro_rules! gen_uniform_setter {
	([$ty:ty; $len:literal]) => { paste::paste! {
		pub fn [<set_uniform_ $ty _ $len>](&self, name: &str, val: &[$ty; $len]) {
			unsafe {
				let loc = self.gl.get_uniform_location(self.program, name);

				if loc.is_none() {
					log::warn!(concat!(
						"Attempted to access uniform {:?} (type [",
						stringify!($ty),
						"; ",
						stringify!($len),
						"]), which does not exist"
					), name);
					return
				}

				self.gl.[<uniform_ $len _ $ty _slice>](loc.as_ref(), val);
			}
		}
	} };

	($ty:ty) => { paste::paste! {
		pub fn [<set_uniform_ $ty>](&self, name: &str, val: $ty) {
			unsafe {
				let loc = self.gl.get_uniform_location(self.program, name);

				if loc.is_none() {
					log::warn!(concat!(
						"Attempted to access uniform {:?} (type ",
						stringify!($ty),
						"), which does not exist"
					), name);
					return
				}

				self.gl.[<uniform_1_ $ty>](loc.as_ref(), val);
			}
		}
	} };

	($tt:tt, $($remaining:tt),*) => {
		gen_uniform_setter!($tt);
		gen_uniform_setter!($($remaining),*);
	}
}

pub struct Program {
	gl: Arc<Context>,
	program: NativeProgram
}

impl Program {
	pub fn activate(&self) {
		unsafe { self.gl.use_program(Some(self.program)); }
	}

	gen_uniform_setter!(
		f32, [f32; 1], [f32; 2], [f32; 3], [f32; 4],
		i32, [i32; 1], [i32; 2], [i32; 3], [i32; 4],
		u32, [u32; 1], [u32; 2], [u32; 3], [u32; 4]
	);
}

