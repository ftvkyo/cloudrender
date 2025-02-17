#version 330

precision mediump float;

in vec2 texcoord_frag;

out vec4 color;

void main() {
    if (sqrt(texcoord_frag.x * texcoord_frag.x + texcoord_frag.y * texcoord_frag.y) < 1.0) {
        color = vec4(1.0);
    } else {
        color = vec4(0.0);
    }
}
