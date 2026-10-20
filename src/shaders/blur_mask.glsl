//_DEFINES_
precision highp float;
varying vec2 v_coords;
uniform sampler2D tex;
uniform float alpha;
uniform float u_preserve_alpha;
#if defined(DEBUG_FLAGS)
uniform float tint;
#endif
void main() {
    float source = texture2D(tex, v_coords).a;
    float a = mix(step(0.001, source), source, u_preserve_alpha);
    gl_FragColor = vec4(a) * alpha;
}
