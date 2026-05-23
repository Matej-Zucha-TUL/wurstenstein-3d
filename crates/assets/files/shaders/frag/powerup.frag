#version 460 core

precision mediump float;

in vec2 tex_coord;
in vec3 world_pos;
in vec3 world_normal;

uniform vec3 base_color = vec3(1.0, 0.0, 0.0);

out vec4 color;

void main() {
	color = vec4(base_color, 0.40);
}

