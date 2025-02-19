#version 330

layout (location = 0) in vec3 position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 texcoord_frag;

void main() {
    int v = gl_VertexID % 4;

    if (v == 0) {
        texcoord_frag = vec2(-1.0, -1.0);
    } else if (v == 1) {
        texcoord_frag = vec2(1.0, -1.0);
    } else if (v == 2) {
        texcoord_frag = vec2(1.0, 1.0);
    } else if (v == 3) {
        texcoord_frag = vec2(-1.0, 1.0);
    }

    gl_Position = projection * view * model * vec4(position, 1.0);
}
