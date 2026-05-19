use std::sync::Arc;

use glow::*;
use nalgebra_glm as glm;

pub struct TransparentObject<'a> {
	position: glm::Vec3,
	render: Box<dyn FnOnce() + 'a>
}

pub struct TransparentRenderer<'a> {
	gl: Arc<Context>,
	objects: Vec<TransparentObject<'a>>
}

impl<'a> TransparentRenderer<'a> {
	pub fn new(gl: Arc<Context>) -> Self {
		Self {
			gl,
			objects: vec![]
		}
	}

	pub fn add_object<F: FnOnce() + 'a>(&mut self, position: glm::Vec3, render: F) {
		self.objects.push(TransparentObject { position, render: Box::new(render) });
	}

	pub fn render(mut self, view_mtx: glm::Mat4) {
		for obj in &mut self.objects {
			obj.position = glm::vec4_to_vec3(&(view_mtx * glm::vec3_to_vec4(&obj.position)));
		}

		self.objects.sort_by(|a, b| a.position[2].partial_cmp(&b.position[2]).unwrap());

		unsafe {
			self.gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
			self.gl.enable(BLEND);
			self.gl.depth_mask(false);
			self.gl.disable(CULL_FACE);
		}

		for obj in self.objects {
			(obj.render)();
		}

		unsafe {
			self.gl.enable(CULL_FACE);
			self.gl.disable(BLEND);
			self.gl.depth_mask(true);
		}
	}
}

