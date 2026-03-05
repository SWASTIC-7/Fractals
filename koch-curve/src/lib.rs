#![no_std]

use bytemuck::{Pod, Zeroable};
use core::f32::consts::PI;
use glam::{Vec2, Vec4, vec2, vec3, vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,
}

/// Returns a normalized direction based on an angle (polar to cartesian)
fn polar_to_cartesian(angle: f32) -> Vec2 {
    vec2(f32::sin(angle), f32::cos(angle))
}

/// Reflects the UV based on a reflection line centered at point p with a given angle
fn reflect_uv(uv: Vec2, p: Vec2, angle: f32) -> Vec2 {
    let dir = polar_to_cartesian(angle);
    let diff = uv - p;
    let dot_val = diff.x * dir.x + diff.y * dir.y;
    let min_dot = if dot_val < 0.0 { dot_val } else { 0.0 };
    uv - dir * min_dot * 2.0
}

/// Smoothstep interpolation
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

const SQRT3: f32 = 1.732050807568877;

/// Folds the 2D space to generate the Koch curve fractal and returns the distance to it
fn koch_curve(uv: &mut Vec2, recursion_count: i32) -> f32 {
    let mut scale = 1.25_f32;
    *uv = *uv * scale;

    // This is here so that the first image is a straight line in the center
    if recursion_count >= 0 {
        uv.y -= SQRT3 / 6.0; // Translate Y coordinate up
        uv.x = f32::abs(uv.x); // Make a reflection line in the Y axis
        *uv = reflect_uv(*uv, vec2(0.5, 0.0), 11.0 / 6.0 * PI); // Make a reflection line to form a triangle
        uv.x += 0.5; // Translate X coordinate to the center of the line
    }

    for _ in 0..recursion_count {
        uv.x -= 0.5; // Translate X coordinate
        scale *= 3.0; // Increase scale for each recursion loop
        *uv = *uv * 3.0; // Scale down the shape
        uv.x = f32::abs(uv.x); // Create a reflection line in the Y axis
        uv.x -= 0.5; // Translate X coordinate
        *uv = reflect_uv(*uv, vec2(0.0, 0.0), (2.0 / 3.0) * PI); // Create angled reflection line to form the triangle
    }

    uv.x = f32::abs(uv.x); // Create a reflection line in the Y axis

    // Calculate distance to the fractal
    let clamped_x = if uv.x < 1.0 { uv.x } else { 1.0 };
    let diff = *uv - vec2(clamped_x, 0.0);
    let d = f32::sqrt(diff.x * diff.x + diff.y * diff.y) / scale;

    *uv = *uv / scale; // Reset the scaling in the uv
    d
}

/// Get iteration count based on time (animates -1 to 6 iterations over 14 seconds)
fn get_iteration_count(time: f32) -> i32 {
    let iteration = -1 + ((time % 14.0) * 0.5) as i32;
    if iteration > 6 { 6 } else { iteration }
}

#[spirv(fragment)]
pub fn main_fs(
    frag_coord: Vec2,
    #[spirv(flat)] resolution: Vec2,
    #[spirv(flat)] time: f32,
    output: &mut Vec4,
) {
    // Center UV coordinates on the center of the canvas
    let uv = (frag_coord - 0.5 * resolution) / resolution.y;
    let mut uv_fractal = uv;

    // Get animated iteration count
    let recursion_count = get_iteration_count(time);

    // Calculate distance to the fractal
    let d = koch_curve(&mut uv_fractal, recursion_count);

    let mut col = vec3(0.0, 0.0, 0.0);

    // Draw the fractal (red color)
    let line_smoothness = 4.0 / resolution.y;
    col.x += smoothstep(line_smoothness, 0.0, d) * 0.5;

    // Draw coordinate axes
    let axis_smoothness = 1.0 / resolution.y;

    // X axis: magenta (red + blue)
    let x_axis = smoothstep(axis_smoothness, 0.0, f32::abs(uv_fractal.y));
    col.x += x_axis;
    col.z += x_axis;

    // Y axis: cyan (blue + green)
    let y_axis = smoothstep(axis_smoothness, 0.0, f32::abs(uv_fractal.x));
    col.z += y_axis;
    col.y += y_axis;

    *output = vec4(col.x, col.y, col.z, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)] constants: &ShaderConstants,
    #[spirv(position)] vtx_pos: &mut Vec4,
    frag_coord: &mut Vec2,
    #[spirv(flat)] resolution: &mut Vec2,
    #[spirv(flat)] time: &mut f32,
) {
    // Fullscreen quad using 6 vertices (2 triangles)
    let positions = [
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, 1.0),
    ];

    let idx = vert_id as usize % 6;
    let pos = positions[idx];

    *vtx_pos = vec4(pos.x, pos.y, 0.0, 1.0);

    // Convert from clip space [-1,1] to pixel coordinates
    let w = constants.width as f32;
    let h = constants.height as f32;
    *frag_coord = vec2((pos.x + 1.0) * 0.5 * w, (pos.y + 1.0) * 0.5 * h);
    *resolution = vec2(w, h);
    *time = constants.time;
}
