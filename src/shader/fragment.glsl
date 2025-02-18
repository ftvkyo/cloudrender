#version 330

precision mediump float;

in vec2 texcoord_frag;

out vec4 color;

void main() {
    float d = length(texcoord_frag);

    if (d < 1.0) {
        float brightness = cos(d * 3.1415 / 2);
        float alpha = smoothstep(0.01, 1.0, brightness);
        color = vec4(vec3(brightness), alpha);
    } else {
        color = vec4(0.0);
    }
}
