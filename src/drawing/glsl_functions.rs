pub const CONVERT_UYVY422_YUV24 : &'static str = r#"
vec4 convert_uyvy422_yuv24(sampler2D uyvy422_tex, ivec2 pix_1) {
    // Based on: http://stackoverflow.com/q/25440114/3622526
    bool is_odd = mod(pix_1.x, 2) != 0;
    int offset = is_odd ? -1 : 1;
    ivec2 pix_2 = ivec2(pix_1.x + offset, pix_1.y);

    vec4 col_1 = texelFetch(uyvy422_tex, pix_1, 0);
    vec4 col_2 = texelFetch(uyvy422_tex, pix_2, 0);

    float y, cb, cr;
    if (is_odd) {
        vec2 uy = col_1.xy;
        vec2 vy = col_2.xy;
        y = uy.y;
        cb = vy.x;
        cr = uy.x;
    } else {
        vec2 uy = col_2.xy;
        vec2 vy = col_1.xy;
        y = vy.y;
        cb = vy.x;
        cr = uy.x;
    }

    return vec4(y, cb, cr, 1.0);
}
"#;

pub const CONVERT_YCBCRA_RGBA : &'static str = r#"
vec4 convert_ycbcra_rgba(vec4 ycbcra) {
    float y = ycbcra.x;
    float cb = ycbcra.y;
    float cr = ycbcra.z;
    float a = ycbcra.w;

    // From https://en.wikipedia.org/wiki/YCbCr#JPEG_conversion
    float r = y + 1.402*(cr-0.5);
    float g = y - 0.34414*(cb-0.5) - 0.71414*(cr-0.5);
    float b = y + 1.772*(cb-0.5);

    return vec4(r, g, b, a);
}
"#;

pub const VERTEX_SHADER_POS_TEX : &'static str = r#"
    #version 140
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;
    // uniform mat4 matrix;
    void main() {
        v_tex_coords = tex_coords;
        // gl_Position = matrix * vec4(position, 0.0, 1.0);
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;
