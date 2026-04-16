#version 460 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

out vec2 tex_coord;
out vec3 normal; // N
out vec3 obj_to_light; // L
out vec3 obj_to_camera; // V

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 light_position = vec3(0, 0, 10.0);
uniform float scale = 1.0;

void main() {
	vec4 coord = vec4(aPos * scale, 1.0);
	vec4 model_coord = model * coord;
	vec4 view_space_coord = view * model_coord;
	vec3 camera_pos = vec3(view[3]);

	normal = mat3(model) * aNormal;
	obj_to_light = light_position - model_coord.xyz;
	obj_to_camera = camera_pos - model_coord.xyz;

	gl_Position = projection * view_space_coord;
	tex_coord = aTexCoord;
}
