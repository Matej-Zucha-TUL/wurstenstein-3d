#version 460 core

in float seed;

out float seed_from_vert;

uniform float time;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

float hash(float n) {
	return fract(sin(n) * 43758.5453123);
}

void main() {
	float dist = hash(seed) * 2.0;
	float angle = hash(seed * 1234.0) * 2 * 3.141592653589793;

	float dx = cos(angle) * dist;
	float dz = sin(angle) * dist;

	float y = -4 * time * time + 4 * time;

	vec4 model_coord = model * vec4(time * dx, y, time * dz, 1.0);

	gl_PointSize = 3.0;
	gl_Position = projection * view * model_coord;

	seed_from_vert = seed;
}

