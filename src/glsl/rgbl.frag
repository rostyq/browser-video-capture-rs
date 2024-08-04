precision highp float;
uniform sampler2D u_texture;
varying vec2 v_texCoord;
const vec3 convert = vec3(0.2126, 0.7152, 0.0722);

void main() {
   vec4 pixel = texture2D(u_texture, v_texCoord);
   pixel.a = dot(pixel.rgb, convert);
   gl_FragColor = pixel;
}
