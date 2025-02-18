#version 330

precision mediump float;

in vec2 texcoord_frag;

out vec4 color;

void main() {
    float d = sqrt(texcoord_frag.x * texcoord_frag.x + texcoord_frag.y * texcoord_frag.y);

    if (d < 1.0) {
        float brightness = cos(d * 3.1415 / 2);
        float alpha = smoothstep(0.01, 1.0, brightness);
        color = vec4(vec3(brightness), alpha);
    } else {
        color = vec4(0.0);
    }
}
