//_DEFINES_
precision highp float;
varying vec2 v_coords;
uniform sampler2D tex;
uniform float alpha;
uniform float u_preserve_alpha;
uniform float u_threshold_opacity;
#if defined(DEBUG_FLAGS)
uniform float tint;
#endif
void main() {
    float source = texture2D(tex, v_coords).a;
    float a = mix(step(0.001, source), source, u_preserve_alpha);
    // Match the quantized alpha capture before weighting native chrome.
    float chrome = step(0.5 / 255.0, source * alpha);
    gl_FragColor = mix(vec4(a) * alpha, vec4(chrome), u_threshold_opacity);
}
