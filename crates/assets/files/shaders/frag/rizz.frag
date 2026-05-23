#version 460 core

#define M_PI 3.1415926535897932384626433832795

// in vec4 gl_FragCoord;
// in bool gl_FrontFacing;
// in vec2 gl_PointCoord;

precision mediump float;

uniform float screen_w;
uniform float screen_h;
uniform float time;

in vec2 tex_coord;

uniform sampler2D tex_unit;

out vec4 color;

void main() {
	float r_mul = sin(time) * 0.25 + 0.75;
	float g_mul = sin(time * 0.67 + 1) * 0.25 + 0.75;
	float b_mul = sin(time * 2) * 0.25 + 0.75;

	float r = sin((gl_FragCoord[0] / screen_w + time / 5) * M_PI * 10 * r_mul) * 0.5 + 0.5;
	float g = sin((gl_FragCoord[1] / screen_h + time / 3.7) * M_PI * 10 * g_mul) * 0.5 + 0.5;
	float b = sin(time * M_PI * b_mul) * 0.5 + 0.5;
	color = vec4(r, g, b, 1) * texture(tex_unit, tex_coord);
}

