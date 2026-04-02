#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

out vec2 tex_coord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform float scale = 1.0;

void main() {
	gl_Position = projection * view * model * vec4(aPos * scale, 1.0);
	tex_coord = aTexCoord;
}
