#version 330

precision mediump float;

in vec2 vert;
out vec4 color;

uniform vec4 selected_color;

void main() {
	color = selected_color;
}

