#version 460 core

in vec3 aPos;
in vec3 aNormal;
in vec2 aTexCoord;

out vec2 tex_coord;
out vec3 world_pos;
out vec3 world_normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_matrix; // transpose(inverse(mat3(model))), computed CPU-side

void main() {
	vec4 model_coord = model * vec4(aPos, 1.0);

	world_pos = model_coord.xyz;
	world_normal = normal_matrix * aNormal;

	gl_Position = projection * view * model_coord;
	tex_coord = aTexCoord;
}
