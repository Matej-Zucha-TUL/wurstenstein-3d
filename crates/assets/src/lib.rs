include!("../assets_struct.rs");

impl Assets {
	pub fn parse_from_data(data: &[u8]) -> Result<Self, String> {
		let data = miniz_oxide::inflate::decompress_to_vec(data)
			.map_err(|e| format!("Error decompressing assets file: {e:?}"))?;

		wincode::deserialize(&data)
			.map_err(|e| format!("Error deserializing assets file: {e:?}"))
	}
}

