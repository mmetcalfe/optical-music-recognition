void main() {
    // vec4 tex_coords = apply_homography(homog, v_tex_coords);
    // vec4 ycbcra = texture(tex, tex_coords);
    // vec4 ycbcra = texture(tex, v_tex_coords);

    // mat4 screen_to_frame = inverse(transpose(homog) * view * model);
    // mat4 screen_to_frame = inverse(homog * view * model);
    // mat4 screen_to_frame = inverse(inverse(homog) * view * model);
    // mat4 screen_to_frame = inverse(view * model * inverse(homog));
    // mat4 screen_to_frame = inverse(view * model * transpose(homog));
    mat4 screen_to_frame = inverse(reference_view * model);
    vec4 frame_pos_4 = screen_to_frame * vec4(screen_pos, 0.0, 1.0);

    vec3 frame_pos_homog = vec3(frame_pos_4.xy, 1.0);

    // frame_pos_homog.x *= 0.5*640;
    // frame_pos_homog.y *= 0.5*480;

    // vec3 homog_pos = homog * frame_pos_homog;
    // vec3 homog_pos = transpose(homog) * frame_pos_homog;
    // vec3 homog_pos = inverse(homog) * frame_pos_homog;
    vec3 homog_pos = inverse(transpose(homog)) * frame_pos_homog;
    // vec3 homog_pos = frame_pos_homog;

    vec2 homog_normalised = homog_pos.xy / homog_pos.z;

    // homog_normalised.x /= 0.5*640;
    // homog_normalised.y /= 0.5*480;

    vec2 tex_coord = (reference_scale * vec4(homog_normalised, 0.0, 1.0)).xy;

    // vec2 tex_coord = homog_normalised;
    // tex_coord += 1.0;
    // tex_coord /= 2.0;

    if (tex_coord.x > 1.0 || tex_coord.x < 0.0 ||
       tex_coord.y > 1.0 || tex_coord.y < 0.0) {
       color = vec4(0.0, 0.0, 0.0, 0.8);
       return;
    }
    vec4 ycbcra = texture(tex, tex_coord);
    vec4 rgba = convert_ycbcra_rgba(ycbcra);
    color = rgba;

    color.a = 0.5;
}
