#version 460 core

// in vec4 gl_FragCoord;
// in bool gl_FrontFacing;
// in vec2 gl_PointCoord;

precision mediump float;

uniform float screen_w;
uniform float time;

out vec4 color;

void main() {
	uint seed = int(gl_FragCoord[1]);

	for(int i = 0; i < 32; i++) {
		seed ^= seed << 13;
		seed ^= seed >> 17;
		seed ^= seed << 5;
	}

	float speed = float((seed % 512) + 512) / 1024.0;

	float x_pos = fract(speed * (time / 3.0 + 6.7) + float(seed % 128) / 128.0);

	float x = gl_FragCoord[0] / screen_w;

	float scaled_color = 1.0 * speed;

	color = (distance(x, x_pos) < 1 / (screen_w)) ? vec4(scaled_color, scaled_color, scaled_color, 1) : vec4(0, 0, 0, 1);
}

