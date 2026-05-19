use nalgebra_glm as glm;

pub struct TransparentObject<'a> {
	position: glm::Vec3,
	render: Box<dyn FnOnce() + 'a>
}

pub struct TransparentRenderer<'a> {
	objects: Vec<TransparentObject<'a>>
}

impl<'a> TransparentRenderer<'a> {
	pub fn new() -> Self {
		Self {
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

		for obj in self.objects {
			(obj.render)();
		}
	}
}

