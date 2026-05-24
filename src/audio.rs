use std::io::Cursor;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use assets::{Music, Sounds};
use modplay::*;
use tinyaudio::prelude::*;
use nalgebra_glm as glm;

const TITLE_RULES: *const *const ModRule_t = [
	&ModRule_t {
		order: 7,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 6,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	std::ptr::null()
].as_ptr();

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
		order: 9,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 19,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	&ModRule_t {
		order: 28,
		row: 0,
		actions: [
			&ModAction_t {
				kind: ModActionKind_t_Jump,
				__bindgen_anon_1: ModAction_t__bindgen_ty_1 {
					jump: ModActionJump_t {
						order: 19,
						row: 0
					}
				}
			} as *const _,
			std::ptr::null()
		].as_ptr()
	} as *const _,
	std::ptr::null()
].as_ptr();

pub enum MusicRequest {
	Title,
	Stop,
	InGame,
	Death,
	Win,
}

pub enum SoundRequest {
	PlayerJump,
	PlayerExplosion,
	PlayerDeath,
	PlayerShoot,
	EnemyHit,
	EnemyDeath,
	EnemyExplosion,
	EnemyShoot,
	PowerupHpPickup,
	PowerupEnergyPickup,
	PowerupSpeedPickup
}

pub enum AudioRequest {
	PlayMusic { kind: MusicRequest },
	PlaySound { kind: SoundRequest, position: Option<[f32; 3]>, radius: f32 },
	SetPosition { position: [f32; 3], rotation: f32 },
}

pub struct Audio {
	tx: Sender<AudioRequest>,
	device: OutputDevice
}

impl Audio {
	pub fn init(music: &Music, sounds: &Sounds) -> Self {
		let (tx, rx) = mpsc::channel();
		let (music_tx, music_rx) = mpsc::channel();

		let space_debris = music.space_debris.clone();
		let humntrgt = music.humntrgt.clone();
		let brewery = music.brewery.clone();

		let params = OutputDeviceParameters {
			channels_count: 2,
			sample_rate: 44100,
			channel_sample_count: 512,
		};

		let (mut scene_handle, mut scene) = oddio::SpatialScene::new();

		let mut play_music = false;

		let device = run_output_device(params, move |data| {
			while let Ok(request) = music_rx.try_recv() {
				match request {
					MusicRequest::Stop => {
						play_music = false;
					},
					MusicRequest::Title => {
						play_music = true;
						unsafe {
							InitMOD(space_debris.as_ptr(), 44100);
							UpdateMODRules(TITLE_RULES);
						}
					},
					MusicRequest::InGame => {
						play_music = true;
						unsafe {
							InitMOD(space_debris.as_ptr(), 44100);
							UpdateMODRules(INGAME_RULES);
						}
					},
					MusicRequest::Death => {
						play_music = true;
						unsafe {
							InitMOD(humntrgt.as_ptr(), 44100);
						}
					},
					MusicRequest::Win => {
						play_music = true;
						unsafe {
							InitMOD(brewery.as_ptr(), 44100);
						}
					},
				}
			}

			let out_frames = oddio::frame_stereo(data);
			oddio::run(&mut scene, 44100, out_frames);

			let mut buf = vec![0i16; data.len()];

			if play_music {
				unsafe { RenderMOD(buf.as_mut_ptr(), buf.len() as i32 / 2); }
			}

			for (sample, out) in buf.iter().zip(data.iter_mut()) {
				*out = (*out + *sample as f32 / 32768.0 / 2.0) / 2.0;
			}
		})
		.unwrap();

		let decode_wav = |data: &[u8]| -> Arc<oddio::Frames<_>> {
			let reader = hound::WavReader::new(Cursor::new(data)).unwrap();
			let samples = reader.into_samples::<i16>().map(|x| x.unwrap() as f32 / 32768.0).collect::<Vec<_>>();
			oddio::Frames::from_slice(44100, &samples)
		};

		let player_jump = decode_wav(&sounds.player_jump);
		let player_explosion = decode_wav(&sounds.player_explosion);
		let player_death = decode_wav(&sounds.player_death);
		let player_shoot = decode_wav(&sounds.player_shoot);

		let enemy_hit = decode_wav(&sounds.enemy_hit);
		let enemy_explosion = decode_wav(&sounds.enemy_explosion);
		let enemy_death = decode_wav(&sounds.enemy_death);
		let enemy_shoot = decode_wav(&sounds.enemy_shoot);

		let powerup_hp_pickup = decode_wav(&sounds.powerup_hp_pickup);
		let powerup_energy_pickup = decode_wav(&sounds.powerup_energy_pickup);
		let powerup_speed_pickup = decode_wav(&sounds.powerup_speed_pickup);

		std::thread::spawn(move || {
			let mut handles: Vec<(Option<[f32; 3]>, oddio::Spatial)> = vec![];
			let mut player_position = [0.0, 0.0, 0.0];
			let mut player_rotation = 0.0;

			'requestloop: for request in rx {
				match request {
					AudioRequest::PlayMusic { kind } => music_tx.send(kind).unwrap(),
					AudioRequest::PlaySound { kind, position, radius } => {
						// Try to find an existing slot

						let mut generate_handle = || {
							let frames = match kind {
								SoundRequest::PlayerJump => player_jump.clone(),
								SoundRequest::PlayerExplosion => player_explosion.clone(),
								SoundRequest::PlayerDeath => player_death.clone(),
								SoundRequest::PlayerShoot => player_shoot.clone(),
								SoundRequest::EnemyHit => enemy_hit.clone(),
								SoundRequest::EnemyDeath => enemy_death.clone(),
								SoundRequest::EnemyExplosion => enemy_explosion.clone(),
								SoundRequest::EnemyShoot => enemy_shoot.clone(),
								SoundRequest::PowerupHpPickup => powerup_hp_pickup.clone(),
								SoundRequest::PowerupEnergyPickup => powerup_energy_pickup.clone(),
								SoundRequest::PowerupSpeedPickup => powerup_speed_pickup.clone(),
							};

							let frames = oddio::FramesSignal::from(frames);

							let translated_position = if let Some(position) = position {
								let pos = glm::vec2(position[0], position[2]) - glm::vec2(player_position[0], player_position[2]);
								let pos = glm::rotate_vec2(&pos, player_rotation);
								let pos = pos.as_slice();

								[pos[0], position[1] - player_position[1], pos[1]]
							} else {
								[0.0, 0.0, 0.0]
							};

							(
								position,
								scene_handle.play(frames, oddio::SpatialOptions {
									position: translated_position.into(),
									velocity: [0.0, 0.0, 0.0].into(),
									radius
								})
							)
						};

						for handle in &mut handles {
							if !handle.1.is_finished() { continue }

							*handle = generate_handle();
							continue 'requestloop;
						}

						handles.push(generate_handle());
					},
					AudioRequest::SetPosition { position, rotation } => {
						player_position = position;
						player_rotation = rotation;

						for handle in &mut handles {
							let Some(position) = handle.0 else { continue };

							let pos = glm::vec2(position[0], position[2]) - glm::vec2(player_position[0], player_position[2]);
							let pos = glm::rotate_vec2(&pos, player_rotation);
							let pos = pos.as_slice();

							let translated_position = [pos[0], position[1] - player_position[1], pos[1]];

							handle.1.set_motion(
								translated_position.into(),
								[0.0, 0.0, 0.0].into(),
								true
							);
						}
					}
				}
			}
		});

		Self {
			tx,
			device,
		}
	}

	pub fn play_music(&self, kind: MusicRequest) {
		self.tx.send(AudioRequest::PlayMusic { kind }).unwrap();
	}

	pub fn play_sound(&self, kind: SoundRequest, position: Option<[f32; 3]>, radius: f32) {
		self.tx.send(AudioRequest::PlaySound { kind, position, radius }).unwrap();
	}

	pub fn update_position(&self, position: [f32; 3], rotation: f32) {
		self.tx.send(AudioRequest::SetPosition { position, rotation }).unwrap();
	}
}

