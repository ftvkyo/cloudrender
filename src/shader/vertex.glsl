#version 330

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texcoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 texcoord_frag;

void main() {
    texcoord_frag = texcoord;
    gl_Position = projection * view * model * vec4(position, 1.0);
    // gl_Position = model * vec4(position, 1.0);
}
