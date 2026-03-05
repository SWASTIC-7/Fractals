#![no_std]

use bytemuck::{Pod, Zeroable};
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

/// Step function: returns 0.0 if x < edge, else 1.0
fn step(edge: f32, x: f32) -> f32 {
    if x < edge { 0.0 } else { 1.0 }
}

/// Smoothstep interpolation
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Folds the 2D space to generate the fractal and returns the distance to it
fn sierpinski_carpet(uv: &mut Vec2, steps: i32) -> f32 {
    let mut scale = 4.0_f32;
    *uv = *uv * 4.0;
    uv.x = f32::abs(uv.x);
    uv.y = f32::abs(uv.y);

    for _ in 0..steps {
        scale *= 3.0;
        *uv = *uv * 3.0;
        uv.x = f32::abs(uv.x);
        uv.y = f32::abs(uv.y);

        // Create holes in the center of each 3x3 grid
        let mask = 1.0 - step(uv.x, 3.0) * step(uv.y, 3.0);
        *uv = *uv * mask;

        // Fold space to create the carpet pattern
        uv.y -= 3.0;
        uv.y = f32::abs(uv.y);
        uv.y -= 3.0;
        uv.y = f32::abs(uv.y);

        uv.x -= 3.0;
        uv.x = f32::abs(uv.x);
        uv.x -= 3.0;
        uv.x = f32::abs(uv.x);
    }

    // Calculate distance to the unit square
    let clamped_x = uv.x.clamp(0.0, 1.0);
    let clamped_y = uv.y.clamp(0.0, 1.0);
    let diff = *uv - vec2(clamped_x, clamped_y);
    let d = f32::sqrt(diff.x * diff.x + diff.y * diff.y) / scale;

    *uv = *uv / scale;
    d
}

/// Get iteration count based on time (animates 0-6 iterations over 12 seconds)
fn get_iteration_count(time: f32) -> i32 {
    let iteration = ((time % 12.0) * 0.5) as i32;
    if iteration > 6 { 6 } else { iteration }
}

#[spirv(fragment)]
pub fn main_fs(
    frag_coord: Vec2,
    #[spirv(flat)] resolution: Vec2,
    #[spirv(flat)] time: f32,
    output: &mut Vec4,
) {
    // Normalized pixel coordinates (centered, aspect-correct)
    let uv = 2.0 * (frag_coord - 0.5 * resolution) / resolution.y;
    let uv2 = uv; // Copy for coloring

    let mut uv_fractal = uv;

    // Get animated iteration count
    let recursion_count = get_iteration_count(time);

    // Calculate distance to the fractal
    let d = sierpinski_carpet(&mut uv_fractal, recursion_count);

    // Line smoothness based on resolution
    let line_smoothness = 3.0 / resolution.y;
    let offset = 0.5;

    // Color channels with gradients
    let r = smoothstep(line_smoothness, 0.0, d) * 0.5 * (uv2.x * 0.5 + 0.5 + offset);
    let g = smoothstep(line_smoothness, 0.0, d) * 0.5 * (-uv2.x * 0.5 + 0.5 + offset);
    let b = smoothstep(line_smoothness, 0.0, d) * 0.5 * (uv2.y * 0.5 + 0.5 + offset);

    let mut col = vec3(r, g, b);

    // Draw coordinate axes
    let axis_smoothness = 2.0 / resolution.y;
    let x_axis = smoothstep(axis_smoothness, 0.0, f32::abs(uv_fractal.y));
    let y_axis = smoothstep(axis_smoothness, 0.0, f32::abs(uv_fractal.x));

    // X axis: magenta (red + blue)
    col.x += x_axis;
    col.z += x_axis;
    // Y axis: cyan (blue + green)
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
