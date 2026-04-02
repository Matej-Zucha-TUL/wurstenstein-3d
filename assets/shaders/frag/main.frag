#version 330

#define M_PI 3.1415926535897932384626433832795

// in vec4 gl_FragCoord;
// in bool gl_FrontFacing;
// in vec2 gl_PointCoord;

precision mediump float;

uniform float screen_w;
uniform float screen_h;
uniform float time;

uniform vec3 ambient_material = vec3(0.5, 0.5, 0.5);
uniform vec3 diffuse_material = vec3(0.5, 0.5, 0.5);
uniform vec3 specular_material = vec3(0.5, 0.5, 0.5);
uniform float specular_shininess = 5.0;

in vec2 tex_coord;
in vec3 normal; // N
in vec3 obj_to_light; // L
in vec3 obj_to_camera; // V

uniform sampler2D tex_unit;

uniform bool rizz_mode;

out vec4 color;

void main() {
	vec3 normal = normalize(normal);
	vec3 obj_to_light = normalize(obj_to_light);
	vec3 obj_to_camera = normalize(obj_to_camera);
	vec3 reflection = reflect(-obj_to_light, normal);

	vec3 ambient = ambient_material;
	vec3 diffuse = max(dot(normal, obj_to_light), 0.0) * diffuse_material;
	vec3 specular = pow(max(dot(reflection, obj_to_camera), 0.0), specular_shininess) * specular_material;

	if(rizz_mode) {
		float r_mul = sin(time) * 0.25 + 0.75;
		float g_mul = sin(time * 0.67 + 1) * 0.25 + 0.75;
		float b_mul = sin(time * 2) * 0.25 + 0.75;

		float r = sin((gl_FragCoord[0] / screen_w + time / 5) * M_PI * 10 * r_mul) * 0.5 + 0.5;
		float g = sin((gl_FragCoord[1] / screen_h + time / 3.7) * M_PI * 10 * g_mul) * 0.5 + 0.5;
		float b = sin(time * M_PI * b_mul) * 0.5 + 0.5;
		color = vec4(r, g, b, 1) * texture(tex_unit, tex_coord);
	} else {
		color = vec4(ambient + diffuse, 1.0) * texture(tex_unit, tex_coord) + vec4(specular, 1.0);
		// color = vec4(ambient + diffuse, 1.0) * texture(tex_unit, tex_coord);
	}
}

