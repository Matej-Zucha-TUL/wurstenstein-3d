#version 460 core

precision mediump float;

#define MAX_POINT_LIGHTS 16

in vec2 tex_coord;
in vec3 world_pos;
in vec3 world_normal;

uniform sampler2D tex_unit;
uniform vec3 camera_position;

// ---- Material ----
uniform vec3 ambient_material   = vec3(0.5);
uniform float specular_shininess = 32.0;

// ---- Directional light ----
uniform vec3 directional_light_direction = vec3(0.0, -10.0, 10.0);
uniform vec3 directional_diffuse  = vec3(0.5);
uniform vec3 directional_specular = vec3(0.5);

// ---- Point lights ----
uniform vec3 point_position[MAX_POINT_LIGHTS];
uniform bool point_enabled [MAX_POINT_LIGHTS];
uniform vec3 point_diffuse [MAX_POINT_LIGHTS];
uniform vec3 point_specular[MAX_POINT_LIGHTS];
// Attenuation: 1 / (kc + kl*d + kq*d*d)
uniform float point_atten_constant  = 1.0;
uniform float point_atten_linear    = 0.09;
uniform float point_atten_quadratic = 0.032;

// ---- Spotlight (hard cone) ----
uniform bool  spot_enabled         = false;
uniform vec3  spot_position;
uniform vec3  spot_direction       = vec3(0.0, 0.0, -1.0); // direction the cone points
uniform float spot_cos_cutoff      = 0.9063;               // cos(25 deg) — feed cos from CPU
uniform vec3  spot_diffuse         = vec3(1.0);
uniform vec3  spot_specular        = vec3(1.0);

out vec4 color;

// Blinn-Phong contribution from one light, given the (already-normalized)
// surface->light direction L, view direction V, normal N.
vec3 blinn_phong(vec3 N, vec3 L, vec3 V, vec3 diffuse_col, vec3 specular_col) {
	float n_dot_l = max(dot(N, L), 0.0);
	vec3  H       = normalize(L + V);
	float n_dot_h = max(dot(N, H), 0.0);

	vec3 diffuse  = n_dot_l * diffuse_col;
	// Guard: no specular if the surface faces away from the light.
	vec3 specular = (n_dot_l > 0.0)
		? pow(n_dot_h, specular_shininess) * specular_col
		: vec3(0.0);

	return diffuse + specular;
}

void main() {
	vec3 N = normalize(world_normal);
	vec3 V = normalize(camera_position - world_pos);

	vec3 lit_rgb = vec3(0.0);
	vec3 spec_rgb = vec3(0.0);

	// --- Directional light (always on, no attenuation) ---
	{
    vec3 L = normalize(-directional_light_direction);
		float n_dot_l = max(dot(N, L), 0.0);
		vec3 H = normalize(L + V);
		float n_dot_h = max(dot(N, H), 0.0);

		lit_rgb  += n_dot_l * directional_diffuse;
		if (n_dot_l > 0.0) {
			spec_rgb += pow(n_dot_h, specular_shininess) * directional_specular;
		}
	}

	// --- Point lights ---
	for (int i = 0; i < MAX_POINT_LIGHTS; ++i) {
		if (!point_enabled[i]) continue;

		vec3  to_light = point_position[i] - world_pos;
		float dist     = length(to_light);
		vec3  L        = to_light / dist;

		float atten = 1.0 / (point_atten_constant
		                   + point_atten_linear    * dist
		                   + point_atten_quadratic * dist * dist);

		float n_dot_l = max(dot(N, L), 0.0);
		vec3  H       = normalize(L + V);
		float n_dot_h = max(dot(N, H), 0.0);

		lit_rgb  += atten * n_dot_l * point_diffuse[i];
		if (n_dot_l > 0.0) {
			spec_rgb += atten * pow(n_dot_h, specular_shininess) * point_specular[i];
		}
	}

	// --- Spotlight (hard cone) ---
	if (spot_enabled) {
		vec3 L = normalize(spot_position - world_pos);
		// Angle between cone axis and the direction from light to fragment.
		float cos_theta = dot(normalize(spot_direction), -L);
		if (cos_theta > spot_cos_cutoff) {
			float n_dot_l = max(dot(N, L), 0.0);
			vec3  H       = normalize(L + V);
			float n_dot_h = max(dot(N, H), 0.0);

			lit_rgb  += n_dot_l * spot_diffuse;
			if (n_dot_l > 0.0) {
				spec_rgb += pow(n_dot_h, specular_shininess) * spot_specular;
			}
		}
	}

	vec4 tex = texture(tex_unit, tex_coord);
	color = vec4(ambient_material + lit_rgb, 1.0) * tex + vec4(spec_rgb, 0.0);
}
