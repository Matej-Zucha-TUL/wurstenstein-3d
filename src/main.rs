use glutin::{
	config::{ConfigTemplateBuilder, GlConfig},
	context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
	display::{GetGlDisplay, GlDisplay},
	surface::{GlSurface, SwapInterval},
};
use winit::event::{Event, WindowEvent};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use glow::*;
use image::ImageReader;

use std::num::NonZeroU32;

fn main() {
	// Create window

	let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
	let window_builder = winit::window::Window::default_attributes()
		.with_title("Hello triangle!")
		.with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

	let template = ConfigTemplateBuilder::new();

	let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_builder));

	let (window, gl_config) = display_builder.build(&event_loop, template, |configs| {
		configs.reduce(|accum, config| {
			if config.num_samples() > accum.num_samples() {
				config
			} else {
				accum
			}
		}).unwrap()
	}).unwrap();

	let raw_window_handle = window
		.as_ref()
		.and_then(|window| window.window_handle().map(Into::into).ok());

	unsafe {
		// Inititalize OpenGL context

		let gl_display = gl_config.display();
		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
				major: 3,
				minor: 3,
			})))
			.build(raw_window_handle);

		let not_current_gl_context = gl_display.create_context(&gl_config, &context_attributes).unwrap();

		let window = window.unwrap();

		let attrs = window.build_surface_attributes(Default::default()).unwrap();
		let gl_surface = gl_display.create_window_surface(&gl_config, &attrs).unwrap();

		let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

		let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

		gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();

		let vertex_array = gl.create_vertex_array().unwrap();
		gl.bind_vertex_array(Some(vertex_array));

		// Load shaders

		let program = gl.create_program().unwrap();

		let shader_sources = [
			(glow::VERTEX_SHADER, include_str!("main.vert")),
			(glow::FRAGMENT_SHADER, include_str!("main.frag")),
		];

		let shaders = shader_sources.map(|(shader_type, shader_source)| {
			let shader = gl.create_shader(shader_type).unwrap();

			gl.shader_source(shader, shader_source);
			gl.compile_shader(shader);

			if !gl.get_shader_compile_status(shader) {
				panic!("{}", gl.get_shader_info_log(shader));
			}

			gl.attach_shader(program, shader);

			shader
		});

		gl.link_program(program);
		if !gl.get_program_link_status(program) {
			panic!("{}", gl.get_program_info_log(program));
		}

		for shader in shaders {
			gl.detach_shader(program, shader);
			gl.delete_shader(shader);
		}

		gl.use_program(Some(program));
		gl.clear_color(0.1, 0.2, 0.3, 1.0);

		let img = ImageReader::open("./assets/textures/ferris.png").unwrap().decode().unwrap().into_rgb8();
		let imgdata = PixelUnpackData::Slice(Some(img.as_raw()));

		let tex = gl.create_texture().unwrap();
		gl.bind_texture(TEXTURE_2D, Some(tex));
		gl.tex_image_2d(TEXTURE_2D, 0, RGB as i32, img.width() as i32, img.height() as i32, 0, RGB, UNSIGNED_BYTE, imgdata);

		#[allow(deprecated)] // Fuck you.
		let _ = event_loop.run(move |event, event_loop| {
			let Event::WindowEvent { window_id: _, event } = event else {
				return
			};

			match event {
				WindowEvent::CloseRequested => {
					event_loop.exit();
				}
				WindowEvent::RedrawRequested => {
					gl.clear(glow::COLOR_BUFFER_BIT);
					gl.draw_arrays(glow::TRIANGLES, 0, 3);
					gl_surface.swap_buffers(&gl_context).unwrap();
				},
				_ => (),
			}
		});
	}
}
