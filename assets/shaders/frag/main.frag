#version 460 core

// in vec4 gl_FragCoord;
// in bool gl_FrontFacing;
// in vec2 gl_PointCoord;

precision mediump float;

uniform vec3 ambient_material = vec3(0.5, 0.5, 0.5);
uniform vec3 diffuse_material = vec3(0.5, 0.5, 0.5);
uniform vec3 specular_material = vec3(0.5, 0.5, 0.5);
uniform float specular_shininess = 5.0;

in vec2 tex_coord;
in vec3 normal; // N
in vec3 obj_to_light; // L
in vec3 obj_to_camera; // V

uniform sampler2D tex_unit;

out vec4 color;

void main() {
	vec3 normal = normalize(normal);
	vec3 obj_to_light = normalize(obj_to_light);
	vec3 obj_to_camera = normalize(obj_to_camera);
  vec3 reflection = reflect(-obj_to_light, normal);

	vec3 ambient = ambient_material;
	vec3 diffuse = max(dot(normal, obj_to_light), 0.0) * diffuse_material;
	vec3 specular = pow(max(dot(reflection, obj_to_camera), 0.0), specular_shininess) * specular_material;

  color = vec4(ambient + diffuse, 1.0) * texture(tex_unit, tex_coord) + vec4(specular, 0.0);
	// color = vec4(ambient + diffuse, 1.0) * texture(tex_unit, tex_coord);
}

