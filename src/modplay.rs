#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
mod bindings {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;

use tinyaudio::prelude::*;

const INGAME_RULES: *const *const ModRule_t = [
	&ModRule_t {
		order: 0,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 8,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	&ModRule_t {
		order: 12,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 17,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	&ModRule_t {
		order: 19,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 9,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	std::ptr::null()
].as_ptr();

pub fn start_music() {
	let module = include_bytes!("../assets/music/space_debris.mod");

	let _ = unsafe { InitMOD(module.as_ptr(), 44100) };

	let params = OutputDeviceParameters {
		channels_count: 2,
		sample_rate: 44100,
		channel_sample_count: 512,
	};

	let _ = unsafe { UpdateMODRules(INGAME_RULES) };

	let device = run_output_device(params, {
		move |data| {
			let mut buf = vec![0i16; data.len()];

			let _ = unsafe { RenderMOD(buf.as_mut_ptr(), buf.len() as i32 / 2) };

			for (sample, out) in buf.iter().zip(data.iter_mut()) {
				*out = *sample as f32 / 32768.0;
			}
		}
	})
	.unwrap();

	Box::leak(Box::new(device));
}

