#version 330

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

uniform bool rizz_mode;

in vec2 vert;
out vec4 color;

uniform vec4 selected_color;

void main() {
	if(rizz_mode) {
		float r = sin((gl_FragCoord[0] / screen_w + time / 5) * M_PI * 10) * 0.5 + 0.5;
		float g = sin((gl_FragCoord[1] / screen_h + time) * M_PI * 10) * 0.5 + 0.5;
		float b = sin(time * 5 * M_PI) * 0.5 + 0.5;
		color = vec4(r, g, b, 1);
	} else {
		color = selected_color * texture(tex_unit, tex_coord);
	}
}

