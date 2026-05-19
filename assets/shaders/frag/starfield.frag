#version 460 core

precision mediump float;

uniform float screen_w;
uniform float time;

out vec4 color;

float hash(float n) {
	return fract(sin(n) * 43758.5453123);
}

void main() {
	float seed_val = hash(gl_FragCoord.y);
	
	float speed = seed_val * 0.5 + 0.5;
	float x_pos = fract(speed * (time / 3.0 + 6.7) + hash(gl_FragCoord.y * 1.234));
	
	float x = gl_FragCoord.x / screen_w;
	float dist = abs(x - x_pos) * screen_w;
	
	float star = step(dist, 1.0) * speed;
	
	color = vec4(vec3(star), 1.0);
}
