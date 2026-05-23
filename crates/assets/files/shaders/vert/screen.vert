#version 460 core

// Use this shader to play around with full-screen fragment shader effects.
// Requires 2 triangles in the vertex buffer, to span the whole screen.

in vec2 aPos;

void main() {
	gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0); 
}

