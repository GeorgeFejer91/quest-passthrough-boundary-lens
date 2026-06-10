pub(super) const OES_COPY_VERTEX_SHADER_SOURCE: &str = r#"#version 300 es
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_uv;
out vec2 v_uv;
void main() {
v_uv = a_uv;
gl_Position = vec4(a_position, 0.0, 1.0);
}"#;

pub(super) const OES_COPY_FRAGMENT_SHADER_SOURCE: &str = r#"#version 300 es
#extension GL_OES_EGL_image_external_essl3 : require
precision highp float;
uniform samplerExternalOES u_source;
uniform vec3 u_screen_to_camera_h0;
uniform vec3 u_screen_to_camera_h1;
uniform vec3 u_screen_to_camera_h2;
uniform mat4 u_source_transform;
uniform int u_eye_index;
uniform int u_content_mapping_mode;
uniform int u_projection_border_policy;
uniform int u_processing_layer;
uniform float u_blur_radius_px;
uniform int u_peripheral_stretch_mode;
uniform vec4 u_peripheral_stretch_params;
uniform vec4 u_peripheral_stretch_blend_params;
uniform int u_peripheral_stretch_corner_mode;
uniform int u_peripheral_stretch_debug;
uniform vec4 u_projection_area_eye_offset_uv;
uniform vec2 u_projection_area_scale;
uniform vec2 u_projection_area_radius;
uniform float u_projection_area_corner_radius_uv;
uniform float u_projection_area_opacity;
uniform float u_projection_border_opacity;
uniform int u_target_footprint_from_metadata;
uniform int u_projection_alpha_mode;
uniform vec2 u_projection_alpha_transform;
uniform vec4 u_camera_uv_calibration;
uniform vec2 u_camera_uv_radial;
uniform vec2 u_source_texel_size;
uniform vec3 u_color_matrix_r0;
uniform vec3 u_color_matrix_r1;
uniform vec3 u_color_matrix_r2;
uniform vec3 u_color_offset;
uniform vec3 u_color_adjust;
uniform int u_source_color_transfer;
in vec2 v_uv;
out vec4 out_color;
vec4 premultiplied_alpha_color(vec3 rgb, float alpha) {
float a = clamp(alpha, 0.0, 1.0);
return vec4(clamp(rgb, vec3(0.0), vec3(1.0)) * a, a);
}
vec4 intended_projection_mask_color() {
if (u_projection_border_policy == 1) {
    return vec4(0.0, 0.0, 0.0, 0.0);
}
return premultiplied_alpha_color(vec3(1.0, 0.0, 0.0), u_projection_border_opacity);
}
vec4 source_invalid_color() {
if (u_projection_border_policy == 1) {
    return vec4(0.0, 0.0, 0.0, 0.0);
}
if (u_processing_layer == 2 && u_peripheral_stretch_debug == 1) {
    return premultiplied_alpha_color(vec3(1.0, 0.0, 1.0), u_projection_border_opacity);
}
return premultiplied_alpha_color(vec3(1.0, 0.0, 0.0), u_projection_border_opacity);
}
float target_footprint_signed_distance_uv(vec2 uv) {
vec2 half_size = vec2(
    clamp(u_projection_area_radius.x, 0.05, 0.50),
    clamp(u_projection_area_radius.y, 0.05, 0.50)
);
float corner_radius = clamp(
    u_projection_area_corner_radius_uv,
    0.0,
    min(half_size.x, half_size.y) - 0.001
);
vec2 q = abs(uv - vec2(0.5)) - (half_size - vec2(corner_radius));
float outside = length(max(q, vec2(0.0)));
float inside = min(max(q.x, q.y), 0.0);
return outside + inside - corner_radius;
}
float projection_area_distance(vec2 uv) {
float signed_distance = target_footprint_signed_distance_uv(uv);
vec2 half_size = vec2(
    clamp(u_projection_area_radius.x, 0.05, 0.50),
    clamp(u_projection_area_radius.y, 0.05, 0.50)
);
return clamp(1.0 + signed_distance / max(min(half_size.x, half_size.y), 0.001), 0.0, 2.0);
}
vec2 projection_area_content_uv(vec2 area_uv) {
vec2 half_size = vec2(
    clamp(u_projection_area_radius.x, 0.05, 0.50),
    clamp(u_projection_area_radius.y, 0.05, 0.50)
);
return (area_uv - (vec2(0.5) - half_size)) / max(half_size * 2.0, vec2(0.001));
}
vec2 projection_area_uv_to_screen_uv(
    vec2 area_uv,
    vec2 projection_area_offset_uv,
    vec2 projection_scale
) {
return (area_uv + projection_area_offset_uv - vec2(0.5)) /
    max(projection_scale, vec2(0.05)) + vec2(0.5);
}
float smooth_unit(float value) {
value = clamp(value, 0.0, 1.0);
return value * value * (3.0 - 2.0 * value);
}
vec2 projection_area_rect_edge_uv(
    vec2 area_uv,
    vec2 domain_min_uv,
    vec2 domain_max_uv,
    bool force_edge_sample
) {
vec2 half_size = vec2(
    clamp(u_projection_area_radius.x, 0.05, 0.50),
    clamp(u_projection_area_radius.y, 0.05, 0.50)
);
float core_scale = clamp(u_peripheral_stretch_params.x, 0.05, 1.0);
vec2 core_half_size = half_size * core_scale;
vec2 p = area_uv - vec2(0.5);
vec2 normalized = p / max(core_half_size, vec2(0.001));
float edge_distance = max(abs(normalized.x), abs(normalized.y));
if (edge_distance <= 1.0 && !force_edge_sample) {
    return area_uv;
}

float effective_edge_distance = force_edge_sample ? max(edge_distance, 1.0) : edge_distance;
vec2 edge_normalized = normalized / max(edge_distance, 0.0001);
vec2 edge_direction_uv = edge_normalized * core_half_size;
vec2 bounded_min_uv = min(domain_min_uv, domain_max_uv);
vec2 bounded_max_uv = max(domain_min_uv, domain_max_uv);

float reach_x = 1.0e6;
if (edge_direction_uv.x > 0.0001) {
    reach_x = (bounded_max_uv.x - 0.5) / edge_direction_uv.x;
} else if (edge_direction_uv.x < -0.0001) {
    reach_x = (bounded_min_uv.x - 0.5) / edge_direction_uv.x;
}
float reach_y = 1.0e6;
if (edge_direction_uv.y > 0.0001) {
    reach_y = (bounded_max_uv.y - 0.5) / edge_direction_uv.y;
} else if (edge_direction_uv.y < -0.0001) {
    reach_y = (bounded_min_uv.y - 0.5) / edge_direction_uv.y;
}
float exterior_reach = max(min(reach_x, reach_y) - 1.0, 0.0001);
float exterior_t = clamp((effective_edge_distance - 1.0) / exterior_reach, 0.0, 1.0);
exterior_t = smooth_unit(exterior_t);

float edge_inset = clamp(u_peripheral_stretch_params.y, 0.0, 0.49);
float max_inset = clamp(max(u_peripheral_stretch_params.z, edge_inset), 0.0, 0.49);
float curve = clamp(u_peripheral_stretch_params.w, 0.25, 6.0);
float shaped_t = pow(exterior_t, curve);
float inset = mix(edge_inset, max_inset, shaped_t);
vec2 sample_half_size = max(core_half_size - vec2(inset), vec2(0.001));
vec2 sample_uv = vec2(0.5) + edge_normalized * sample_half_size;
return clamp(sample_uv, bounded_min_uv, bounded_max_uv);
}
float peripheral_stretch_blend_weight(float signed_distance_uv) {
float inner_blend = clamp(u_peripheral_stretch_blend_params.x, 0.0, 0.25);
float blend_curve = clamp(u_peripheral_stretch_blend_params.y, 0.25, 6.0);
float blend_mode = floor(u_peripheral_stretch_blend_params.z + 0.5);
if (blend_mode < 0.5) {
    return signed_distance_uv >= 0.0 ? 1.0 : 0.0;
}
if (inner_blend <= 0.0001) {
    return signed_distance_uv >= 0.0 ? 1.0 : 0.0;
}
float t = smoothstep(-inner_blend, 0.0, signed_distance_uv);
return pow(t, blend_curve);
}
bool peripheral_stretch_enabled() {
// Keep the mode/corner uniforms live for the public runtime contract, but let
// the processing layer own effect activation like the HWB shader path.
return u_processing_layer == 2 ||
    u_processing_layer == 6 ||
    (u_peripheral_stretch_mode + u_peripheral_stretch_corner_mode) == -4096;
}
vec2 calibrated_camera_uv(vec2 uv) {
vec2 center = vec2(0.5);
vec2 offset_uv = clamp(u_camera_uv_calibration.xy, vec2(-0.5), vec2(0.5));
vec2 scale = clamp(u_camera_uv_calibration.zw, vec2(0.25), vec2(4.0));
vec2 radial = clamp(u_camera_uv_radial, vec2(-4.0, -8.0), vec2(4.0, 8.0));
vec2 delta = uv - center;
float r2 = dot(delta, delta);
float radial_scale = 1.0 + radial.x * r2 + radial.y * r2 * r2;
return center + delta * scale * radial_scale + offset_uv;
}
bool selected_mesh_warp_enabled() {
return u_processing_layer == 7 || u_processing_layer == 8;
}
int selected_mesh_control_count() {
return u_eye_index == 0 ? 10 : 7;
}
vec4 selected_mesh_control(int index) {
if (u_eye_index == 0) {
    if (index == 0) { return vec4(0.795866, 0.373647, -0.013156, 0.008298); }
    if (index == 1) { return vec4(0.550475, 0.431970, 0.013135, 0.002463); }
    if (index == 2) { return vec4(0.857299, 0.315608, -0.012260, -0.001637); }
    if (index == 3) { return vec4(0.795636, 0.489534, -0.013143, 0.001676); }
}
if (u_eye_index != 0) {
    if (index == 0) { return vec4(0.234737, 0.723138, 0.001747, -0.002480); }
}
int anchor_base = u_eye_index == 0 ? 4 : 1;
int anchor_index = index - anchor_base;
if (anchor_index == 0) { return vec4(0.0, 0.0, 0.0, 0.0); }
if (anchor_index == 1) { return vec4(1.0, 0.0, 0.0, 0.0); }
if (anchor_index == 2) { return vec4(0.0, 1.0, 0.0, 0.0); }
if (anchor_index == 3) { return vec4(1.0, 1.0, 0.0, 0.0); }
if (anchor_index == 4) { return vec4(0.5, 0.0, 0.0, 0.0); }
return vec4(0.5, 1.0, 0.0, 0.0);
}
vec2 selected_mesh_displacement_uv(vec2 screen_uv) {
vec2 weighted_sum = vec2(0.0);
float weight_sum = 0.0;
int count = selected_mesh_control_count();
for (int index = 0; index < 13; ++index) {
    if (index < count) {
        vec4 control = selected_mesh_control(index);
        vec2 delta = screen_uv - control.xy;
        float weight = 1.0 / max(dot(delta, delta), 0.00018);
        weighted_sum += control.zw * weight;
        weight_sum += weight;
    }
}
return clamp(weighted_sum / max(weight_sum, 0.0001), vec2(-0.055), vec2(0.055));
}
int selected_mesh_residual_control_count() {
return u_eye_index == 0 ? 12 : 15;
}
vec4 selected_mesh_residual_control(int index) {
if (u_eye_index == 0) {
    if (index == 0) { return vec4(0.798444, 0.374622, -0.013041, -0.012180); }
    if (index == 1) { return vec4(0.292795, 0.373348, -0.016470, 0.005591); }
    if (index == 2) { return vec4(0.547329, 0.491041, 0.000846, 0.011597); }
    if (index == 3) { return vec4(0.118649, 0.217117, -0.017780, -0.015875); }
    if (index == 4) { return vec4(0.667247, 0.321816, -0.008597, 0.015223); }
    if (index == 5) { return vec4(0.285261, 0.307942, -0.015886, 0.005334); }
}
if (u_eye_index != 0) {
    if (index == 0) { return vec4(0.839369, 0.212618, 0.016884, -0.011850); }
    if (index == 1) { return vec4(0.831199, 0.263474, 0.006430, -0.010489); }
    if (index == 2) { return vec4(0.481721, 0.328393, -0.010439, -0.002456); }
    if (index == 3) { return vec4(0.423276, 0.382708, -0.010579, 0.011806); }
    if (index == 4) { return vec4(0.300199, 0.379682, 0.012096, 0.009280); }
    if (index == 5) { return vec4(0.481928, 0.385095, -0.012451, 0.000445); }
    if (index == 6) { return vec4(0.480971, 0.492366, -0.007527, -0.007231); }
    if (index == 7) { return vec4(0.481721, 0.328393, -0.009967, -0.004782); }
    if (index == 8) { return vec4(0.364988, 0.437129, 0.012729, 0.001742); }
}
int anchor_base = u_eye_index == 0 ? 6 : 9;
int anchor_index = index - anchor_base;
if (anchor_index == 0) { return vec4(0.0, 0.0, 0.0, 0.0); }
if (anchor_index == 1) { return vec4(1.0, 0.0, 0.0, 0.0); }
if (anchor_index == 2) { return vec4(0.0, 1.0, 0.0, 0.0); }
if (anchor_index == 3) { return vec4(1.0, 1.0, 0.0, 0.0); }
if (anchor_index == 4) { return vec4(0.5, 0.0, 0.0, 0.0); }
return vec4(0.5, 1.0, 0.0, 0.0);
}
vec2 selected_mesh_residual_displacement_uv(vec2 screen_uv) {
vec2 weighted_sum = vec2(0.0);
float weight_sum = 0.0;
int count = selected_mesh_residual_control_count();
for (int index = 0; index < 15; ++index) {
    if (index < count) {
        vec4 control = selected_mesh_residual_control(index);
        vec2 delta = screen_uv - control.xy;
        float weight = 1.0 / max(dot(delta, delta), 0.00018);
        weighted_sum += control.zw * weight;
        weight_sum += weight;
    }
}
return clamp(weighted_sum / max(weight_sum, 0.0001), vec2(-0.035), vec2(0.035));
}
float srgb_channel_to_linear(float value) {
float c = clamp(value, 0.0, 1.0);
return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4);
}
vec3 apply_source_color_transfer(vec3 rgb) {
if (u_source_color_transfer == 1) {
    return vec3(
        srgb_channel_to_linear(rgb.r),
        srgb_channel_to_linear(rgb.g),
        srgb_channel_to_linear(rgb.b)
    );
}
return rgb;
}
vec3 adjusted_camera_rgb(vec2 uv) {
vec4 transformed = u_source_transform * vec4(clamp(uv, vec2(0.0), vec2(1.0)), 0.0, 1.0);
vec2 texture_uv = clamp(transformed.xy, vec2(0.0), vec2(1.0));
vec3 source_rgb = apply_source_color_transfer(texture(u_source, texture_uv).rgb);
vec3 adjusted_rgb = vec3(
    dot(u_color_matrix_r0, source_rgb),
    dot(u_color_matrix_r1, source_rgb),
    dot(u_color_matrix_r2, source_rgb)
) + u_color_offset;
float luma = dot(adjusted_rgb, vec3(0.2126, 0.7152, 0.0722));
adjusted_rgb = mix(vec3(luma), adjusted_rgb, max(u_color_adjust.z, 0.0));
adjusted_rgb = (adjusted_rgb - vec3(0.5)) * max(u_color_adjust.x, 0.0) +
    vec3(0.5 + u_color_adjust.y);
return clamp(adjusted_rgb, vec3(0.0), vec3(1.0));
}
float projection_alpha_mask(vec3 rgb) {
vec3 color = clamp(rgb, vec3(0.0), vec3(1.0));
float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
float max_channel = max(max(color.r, color.g), color.b);
float min_channel = min(min(color.r, color.g), color.b);
float saturation = max_channel - min_channel;
if (u_projection_alpha_mode == 1) {
    return color.r;
}
if (u_projection_alpha_mode == 2) {
    return color.g;
}
if (u_projection_alpha_mode == 3) {
    return color.b;
}
if (u_projection_alpha_mode == 4) {
    return luma;
}
if (u_projection_alpha_mode == 5) {
    return 1.0 - color.r;
}
if (u_projection_alpha_mode == 6) {
    return 1.0 - color.g;
}
if (u_projection_alpha_mode == 7) {
    return 1.0 - color.b;
}
if (u_projection_alpha_mode == 8) {
    return 1.0 - luma;
}
if (u_projection_alpha_mode == 9) {
    return max(color.r - max(color.g, color.b), 0.0);
}
if (u_projection_alpha_mode == 10) {
    return max(color.g - max(color.r, color.b), 0.0);
}
if (u_projection_alpha_mode == 11) {
    return max(color.b - max(color.r, color.g), 0.0);
}
if (u_projection_alpha_mode == 12) {
    return saturation;
}
if (u_projection_alpha_mode == 13) {
    return 1.0 - saturation;
}
return 1.0;
}
float projection_color_alpha(vec3 rgb) {
float mask = projection_alpha_mask(rgb) * max(u_projection_alpha_transform.x, 0.0) +
    u_projection_alpha_transform.y;
return clamp(u_projection_area_opacity * clamp(mask, 0.0, 1.0), 0.0, 1.0);
}
vec4 camera_sample(vec2 uv) {
vec3 rgb = adjusted_camera_rgb(uv);
return premultiplied_alpha_color(rgb, projection_color_alpha(rgb));
}
vec4 blurred_camera_sample(vec2 uv) {
float radius = max(u_blur_radius_px, 0.0);
if (radius <= 0.001) {
    return camera_sample(uv);
}
vec2 texel = u_source_texel_size * radius * 4.0;
vec2 sample_uv = clamp(uv, vec2(0.0), vec2(1.0));
vec3 sum = vec3(0.0);
for (int y = -2; y <= 2; ++y) {
    for (int x = -2; x <= 2; ++x) {
        sum += adjusted_camera_rgb(sample_uv + vec2(float(x), float(y)) * texel);
    }
}
vec3 rgb = sum / 25.0;
return premultiplied_alpha_color(rgb, projection_color_alpha(rgb));
}
vec4 boundary_tint_sample(vec2 uv, float signed_distance_uv) {
vec3 source_rgb = adjusted_camera_rgb(uv);
float source_edge_distance = min(
    min(uv.x, uv.y),
    min(1.0 - uv.x, 1.0 - uv.y)
);
float projection_inner_distance = max(-signed_distance_uv, 0.0);
float edge_distance = min(source_edge_distance, projection_inner_distance);
float feather = smoothstep(0.0, 0.025, edge_distance);
float ring = 1.0 - smoothstep(0.003, 0.015, edge_distance);
float base_alpha = projection_color_alpha(source_rgb) * feather;
float ring_alpha = ring * clamp(max(u_projection_area_opacity, 0.18), 0.0, 0.38);
float alpha = max(base_alpha, ring_alpha);
vec3 tint_rgb = vec3(0.12, 0.42, 1.0);
vec3 lens_rgb = mix(source_rgb, tint_rgb, 0.22);
vec3 edge_rgb = vec3(0.25, 0.70, 1.0);
lens_rgb = mix(lens_rgb, edge_rgb, ring * 0.55);
return premultiplied_alpha_color(lens_rgb, alpha);
}
vec4 seam_blend_sample(vec2 uv, float signed_distance_uv) {
vec3 source_rgb = adjusted_camera_rgb(uv);
float source_edge_distance = min(
    min(uv.x, uv.y),
    min(1.0 - uv.x, 1.0 - uv.y)
);
float projection_inner_distance = max(-signed_distance_uv, 0.0);
float projection_outer_distance = max(signed_distance_uv, 0.0);
float inner_width = clamp(max(u_peripheral_stretch_blend_params.x, 0.075), 0.006, 0.24);
float outer_width = clamp(max(u_peripheral_stretch_params.z, 0.070), 0.006, 0.28);
float inner_edge_distance = min(source_edge_distance, projection_inner_distance);
float inner_feather = smoothstep(0.0, inner_width, inner_edge_distance);
float outer_feather = 1.0 - smoothstep(0.0, outer_width, projection_outer_distance);
float base_alpha = projection_color_alpha(source_rgb);
float inner_dissolve = pow(clamp(inner_feather, 0.0, 1.0), 1.45);
float outer_dissolve = pow(clamp(outer_feather, 0.0, 1.0), 1.65);
float alpha = signed_distance_uv <= 0.0
    ? base_alpha * inner_dissolve
    : base_alpha * outer_dissolve * 0.18;
float seam_t = signed_distance_uv <= 0.0
    ? 1.0 - inner_dissolve
    : outer_dissolve;
vec3 tint_rgb = vec3(0.12, 0.34, 0.80);
float tint_strength = mix(0.004, 0.030, clamp(seam_t, 0.0, 1.0));
vec3 lens_rgb = mix(source_rgb, tint_rgb, tint_strength);
return premultiplied_alpha_color(lens_rgb, alpha);
}
vec4 spatial_warp_sample(vec2 uv, float signed_distance_uv) {
vec2 center = vec2(0.5);
vec2 delta = uv - center;
float radius = length(delta);
float safe_radius = max(radius, 0.00001);
float edge_ease = smoothstep(0.0, 0.85, radius);
float sample_radius = radius * (1.0 - 0.38 * pow(max(1.0 - edge_ease, 0.0), 2.0));
vec2 warped_uv = center + delta * (sample_radius / safe_radius);
warped_uv = clamp(warped_uv, vec2(0.0), vec2(1.0));
vec3 source_rgb = adjusted_camera_rgb(warped_uv);
float source_edge_distance = min(
    min(uv.x, uv.y),
    min(1.0 - uv.x, 1.0 - uv.y)
);
float projection_inner_distance = max(-signed_distance_uv, 0.0);
float edge_distance = min(source_edge_distance, projection_inner_distance);
float feather = smoothstep(0.0, 0.025, edge_distance);
float boundary_ring = 1.0 - smoothstep(0.004, 0.020, edge_distance);
float radial_ring = smoothstep(0.965, 1.0, abs(sin(radius * 44.0)));
float displacement = clamp(length(warped_uv - uv) * 8.0, 0.0, 1.0);
float alpha = max(
    projection_color_alpha(source_rgb) * feather,
    boundary_ring * clamp(max(u_projection_area_opacity, 0.22), 0.0, 0.46)
);
vec3 warp_tint_rgb = vec3(0.04, 0.22, 1.0);
vec3 flow_rgb = vec3(0.10, 0.95, 1.0);
vec3 lens_rgb = mix(source_rgb, warp_tint_rgb, 0.20);
lens_rgb = mix(lens_rgb, flow_rgb, clamp(radial_ring * 0.16 + displacement * 0.10, 0.0, 0.38));
lens_rgb = mix(lens_rgb, vec3(0.25, 0.75, 1.0), boundary_ring * 0.62);
return premultiplied_alpha_color(lens_rgb, alpha);
}
vec4 alignment_edges_sample(vec2 uv, float signed_distance_uv) {
vec2 texel = max(u_source_texel_size, vec2(0.0005)) * 2.0;
vec3 center_rgb = adjusted_camera_rgb(uv);
float center_luma = dot(center_rgb, vec3(0.2126, 0.7152, 0.0722));
float luma_left = dot(adjusted_camera_rgb(uv - vec2(texel.x, 0.0)), vec3(0.2126, 0.7152, 0.0722));
float luma_right = dot(adjusted_camera_rgb(uv + vec2(texel.x, 0.0)), vec3(0.2126, 0.7152, 0.0722));
float luma_down = dot(adjusted_camera_rgb(uv - vec2(0.0, texel.y)), vec3(0.2126, 0.7152, 0.0722));
float luma_up = dot(adjusted_camera_rgb(uv + vec2(0.0, texel.y)), vec3(0.2126, 0.7152, 0.0722));
float gx = luma_right - luma_left;
float gy = luma_up - luma_down;
float gradient_edge = smoothstep(0.018, 0.105, length(vec2(gx, gy)));
float local_average = (luma_left + luma_right + luma_down + luma_up) * 0.25;
float fine_edge = smoothstep(0.012, 0.055, abs(center_luma - local_average));
float edge = clamp(max(gradient_edge, fine_edge * 0.72), 0.0, 1.0);
float source_edge_distance = min(
    min(uv.x, uv.y),
    min(1.0 - uv.x, 1.0 - uv.y)
);
float projection_inner_distance = max(-signed_distance_uv, 0.0);
float edge_distance = min(source_edge_distance, projection_inner_distance);
float feather = smoothstep(0.0, 0.018, edge_distance);
float boundary_ring = 1.0 - smoothstep(0.003, 0.014, edge_distance);
float orientation = abs(gy) / max(abs(gx) + abs(gy), 0.0001);
vec3 edge_rgb = mix(vec3(0.0, 0.92, 1.0), vec3(1.0, 0.12, 0.72), orientation);
vec3 rgb = mix(edge_rgb, vec3(0.18, 0.52, 1.0), boundary_ring * 0.42);
float edge_alpha = edge * feather * clamp(u_projection_area_opacity, 0.0, 1.0);
float ring_alpha = boundary_ring * clamp(max(u_projection_area_opacity, 0.24), 0.0, 0.42);
return premultiplied_alpha_color(rgb, max(edge_alpha, ring_alpha));
}
float uv_fiducial_line(float coord) {
float nearest = floor(coord * 4.0 + 0.5) / 4.0;
float in_range = step(0.20, nearest) * step(nearest, 0.80);
float distance_to_line = abs(coord - nearest);
return smoothstep(0.0040, 0.0015, distance_to_line) * in_range;
}
float uv_fiducial_marker_mask(vec2 screen_uv, vec2 center) {
vec2 delta = abs(screen_uv - center);
float disk = 1.0 - smoothstep(0.024, 0.030, length(screen_uv - center));
float horizontal = (1.0 - step(0.006, delta.y)) * (1.0 - step(0.062, delta.x));
float vertical = (1.0 - step(0.006, delta.x)) * (1.0 - step(0.062, delta.y));
return clamp(max(disk, max(horizontal, vertical)), 0.0, 1.0);
}
void uv_fiducial_blend_marker(vec2 screen_uv, vec2 center, vec3 marker_rgb, inout vec3 rgb, inout float alpha) {
float marker = uv_fiducial_marker_mask(screen_uv, center);
rgb = mix(rgb, marker_rgb, marker);
alpha = max(alpha, marker);
}
vec4 uv_fiducial_sample(vec2 uv, float signed_distance_uv, vec2 screen_uv) {
vec3 source_rgb = adjusted_camera_rgb(uv);
vec3 base_rgb = mix(source_rgb, vec3(0.025, 0.080, 0.115), 0.24);
float feather = smoothstep(0.055, -0.005, signed_distance_uv);
float alpha = clamp(u_projection_area_opacity * 0.26, 0.0, 0.30) * feather;
float grid_alpha = max(
    uv_fiducial_line(screen_uv.x) * 0.22,
    uv_fiducial_line(screen_uv.y) * 0.22
) * feather;
vec3 rgb = mix(base_rgb, vec3(0.20), grid_alpha);
alpha = max(alpha, grid_alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.25, 0.25), vec3(0.0, 1.0, 1.0), rgb, alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.25, 0.50), vec3(1.0, 0.0, 0.0), rgb, alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.50, 0.25), vec3(1.0, 1.0, 0.0), rgb, alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.50, 0.50), vec3(0.0, 1.0, 0.0), rgb, alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.50, 0.75), vec3(1.0, 0.0, 1.0), rgb, alpha);
uv_fiducial_blend_marker(screen_uv, vec2(0.75, 0.50), vec3(0.0, 0.0, 1.0), rgb, alpha);
float boundary_ring = 1.0 - smoothstep(0.0, 0.018, abs(signed_distance_uv));
rgb = mix(rgb, vec3(0.05, 0.78, 1.0), boundary_ring * 0.45);
alpha = max(alpha, boundary_ring * 0.36);
return premultiplied_alpha_color(rgb, alpha);
}
vec3 dense_uv_hue_rgb(float hue) {
vec3 p = abs(fract(vec3(hue, hue + 0.666667, hue + 0.333333)) * 6.0 - 3.0);
return clamp(p - 1.0, vec3(0.0), vec3(1.0));
}
float dense_uv_fiducial_marker_mask(vec2 screen_uv, vec2 center) {
float radius = length(screen_uv - center);
float disk = 1.0 - smoothstep(0.0155, 0.0195, radius);
float core = 1.0 - smoothstep(0.004, 0.010, radius);
return clamp(max(disk * 0.92, core), 0.0, 1.0);
}
vec4 dense_uv_fiducial_sample(vec2 uv, float signed_distance_uv, vec2 screen_uv) {
vec3 source_rgb = adjusted_camera_rgb(uv);
vec3 rgb = mix(source_rgb, vec3(0.020, 0.060, 0.080), 0.30);
float feather = smoothstep(0.055, -0.005, signed_distance_uv);
float alpha = clamp(u_projection_area_opacity * 0.18, 0.0, 0.22) * feather;
for (int y = 0; y < 5; ++y) {
    for (int x = 0; x < 5; ++x) {
        vec2 center = vec2(0.20 + float(x) * 0.15, 0.20 + float(y) * 0.15);
        float marker = dense_uv_fiducial_marker_mask(screen_uv, center) * feather;
        float hue = fract(0.09 + float(x) * 0.17 + float(y) * 0.31);
        vec3 marker_rgb = dense_uv_hue_rgb(hue);
        rgb = mix(rgb, marker_rgb, marker);
        alpha = max(alpha, marker);
    }
}
float boundary_ring = 1.0 - smoothstep(0.0, 0.014, abs(signed_distance_uv));
rgb = mix(rgb, vec3(0.05, 0.78, 1.0), boundary_ring * 0.26);
alpha = max(alpha, boundary_ring * 0.22);
return premultiplied_alpha_color(rgb, alpha);
}
void main() {
vec2 renderer_surface_uv = v_uv;
vec2 screen_uv = vec2(renderer_surface_uv.x, 1.0 - renderer_surface_uv.y);
vec2 projection_scale = max(u_projection_area_scale, vec2(0.05));
vec2 requested_projection_area_offset_uv = u_eye_index == 0
    ? u_projection_area_eye_offset_uv.xy
    : u_projection_area_eye_offset_uv.zw;
vec2 projection_area_offset_uv = vec2(
    clamp(requested_projection_area_offset_uv.x, -0.5, 0.5),
    clamp(requested_projection_area_offset_uv.y, -0.5, 0.5)
);
vec2 projection_area_uv =
    (screen_uv - vec2(0.5)) * projection_scale + vec2(0.5) -
    projection_area_offset_uv;
vec2 canonical_projection_area_uv = projection_area_uv;
float signed_distance_uv = target_footprint_signed_distance_uv(canonical_projection_area_uv);
float area_distance = projection_area_distance(canonical_projection_area_uv);
bool stretch_exterior = false;
bool target_transition_band = false;
bool peripheral_stretch_active = peripheral_stretch_enabled();
float stretch_weight = peripheral_stretch_active
    ? peripheral_stretch_blend_weight(signed_distance_uv)
    : 0.0;
bool projection_area_inside = signed_distance_uv <= 0.0;
stretch_exterior = peripheral_stretch_active && !projection_area_inside;
target_transition_band =
    peripheral_stretch_active && projection_area_inside && stretch_weight > 0.0001;
bool stretch_effect_sample_region = stretch_exterior || target_transition_band;
if (stretch_effect_sample_region) {
    vec2 domain_min_uv =
        vec2(0.5) - vec2(0.5) * projection_scale - projection_area_offset_uv;
    vec2 domain_max_uv =
        vec2(0.5) + vec2(0.5) * projection_scale - projection_area_offset_uv;
    vec2 stretch_projection_area_uv = projection_area_rect_edge_uv(
        canonical_projection_area_uv,
        domain_min_uv,
        domain_max_uv,
        stretch_weight > 0.0001
    );
    projection_area_uv = mix(
        canonical_projection_area_uv,
        stretch_projection_area_uv,
        clamp(stretch_weight, 0.0, 1.0)
    );
    screen_uv = projection_area_uv_to_screen_uv(
        projection_area_uv,
        projection_area_offset_uv,
        projection_scale
    );
} else if (area_distance > 1.0) {
    out_color = intended_projection_mask_color();
    return;
}
vec2 target_content_uv = projection_area_content_uv(projection_area_uv);
vec2 camera_uv = vec2(0.0);
if (u_content_mapping_mode == 1) {
    camera_uv = target_content_uv;
} else {
    vec2 homography_input_uv = screen_uv;
    // Keep the metadata flag live for the uniform contract. The camera
    // homography rows are screen-domain; the target footprint only owns the
    // mask and edge-remap boundary.
    if (u_target_footprint_from_metadata == 2) {
        homography_input_uv = target_content_uv;
    }
    vec3 input_uv = vec3(homography_input_uv, 1.0);
    vec3 camera_uv_h = vec3(
        dot(u_screen_to_camera_h0, input_uv),
        dot(u_screen_to_camera_h1, input_uv),
        dot(u_screen_to_camera_h2, input_uv)
    );
    if (abs(camera_uv_h.z) < 0.00001 && !peripheral_stretch_active) {
        out_color = source_invalid_color();
        return;
    }
    float safe_camera_uv_z = abs(camera_uv_h.z) < 0.00001
        ? (camera_uv_h.z < 0.0 ? -0.00001 : 0.00001)
        : camera_uv_h.z;
    camera_uv = camera_uv_h.xy / safe_camera_uv_z;
}
camera_uv = calibrated_camera_uv(camera_uv);
if (selected_mesh_warp_enabled()) {
    camera_uv -= selected_mesh_displacement_uv(screen_uv);
    camera_uv -= selected_mesh_residual_displacement_uv(screen_uv);
}
bool source_uv_numeric = camera_uv.x == camera_uv.x && camera_uv.y == camera_uv.y;
if (peripheral_stretch_active && stretch_effect_sample_region && !source_uv_numeric) {
    // NaN/undefined transition/exterior samples still belong to the stretch
    // layer; core failures remain source-invalid diagnostics.
    camera_uv = vec2(0.5);
    source_uv_numeric = true;
}
bool source_uv_valid =
    source_uv_numeric &&
    camera_uv.x >= 0.0 &&
    camera_uv.x <= 1.0 &&
    camera_uv.y >= 0.0 &&
    camera_uv.y <= 1.0;
if (peripheral_stretch_active && stretch_effect_sample_region && !source_uv_valid) {
    // Treat source-invalid transition/exterior pixels as part of the explicit
    // stretch layer so edge clamping cannot create an unlabeled band.
    vec2 source_edge_epsilon = max(u_source_texel_size * 0.5, vec2(0.0001));
    camera_uv = clamp(camera_uv, source_edge_epsilon, vec2(1.0) - source_edge_epsilon);
    source_uv_valid =
        camera_uv.x >= 0.0 &&
        camera_uv.x <= 1.0 &&
        camera_uv.y >= 0.0 &&
        camera_uv.y <= 1.0;
}
if (!source_uv_valid) {
    out_color = source_invalid_color();
    return;
}
if (u_processing_layer == 3) {
    out_color = boundary_tint_sample(camera_uv, signed_distance_uv);
    return;
}
if (u_processing_layer == 6) {
    out_color = seam_blend_sample(camera_uv, signed_distance_uv);
    return;
}
if (u_processing_layer == 7) {
    out_color = alignment_edges_sample(camera_uv, signed_distance_uv);
    return;
}
if (u_processing_layer == 8) {
    out_color = seam_blend_sample(camera_uv, signed_distance_uv);
    return;
}
if (u_processing_layer == 9) {
    out_color = uv_fiducial_sample(camera_uv, signed_distance_uv, screen_uv);
    return;
}
if (u_processing_layer == 10) {
    out_color = dense_uv_fiducial_sample(camera_uv, signed_distance_uv, screen_uv);
    return;
}
if (u_processing_layer == 4) {
    out_color = spatial_warp_sample(camera_uv, signed_distance_uv);
    return;
}
if (u_processing_layer == 5) {
    out_color = alignment_edges_sample(camera_uv, signed_distance_uv);
    return;
}
vec4 sample_color = u_processing_layer == 1
    ? blurred_camera_sample(camera_uv)
    : camera_sample(camera_uv);
if (u_processing_layer == 2 && u_peripheral_stretch_debug == 2 && stretch_effect_sample_region) {
    out_color = vec4(
        clamp(camera_uv, vec2(0.0), vec2(1.0)),
        target_transition_band ? 0.5 : 0.0,
        1.0
    );
    return;
}
if (u_processing_layer == 2 && u_peripheral_stretch_debug == 1 && target_transition_band) {
    out_color = premultiplied_alpha_color(vec3(1.0, 0.85, 0.0), 1.0);
    return;
}
if (u_processing_layer == 2 && u_peripheral_stretch_debug == 1 && stretch_exterior) {
    out_color = premultiplied_alpha_color(vec3(0.0, 1.0, 1.0), 1.0);
    return;
}
out_color = sample_color;
}"#;
