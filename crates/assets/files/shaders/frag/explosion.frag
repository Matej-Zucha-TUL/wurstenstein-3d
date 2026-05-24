#version 460 core

precision mediump float;

in float seed_from_vert;

out vec4 color;

uniform float time;

void main() {
	color = vec4(1.0, seed_from_vert, 0.0, 1.0 - time);
}

