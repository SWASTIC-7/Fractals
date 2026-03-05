#![no_std]

use bytemuck::{Pod, Zeroable};
use core::f32::consts::PI;
use glam::{Vec2, Vec4, vec2, vec4};
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

const SQRT3: f32 = 1.732050807568877;

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

/// Returns the signed distance of a point to an equilateral triangle centered on screen
fn signed_dist_triangle(p: Vec2) -> f32 {
    let mut p = p;
    p.x = f32::abs(p.x) - 1.0;
    p.y = p.y + 1.0 / SQRT3;

    if p.x + SQRT3 * p.y > 0.0 {
        let new_x = (p.x - SQRT3 * p.y) / 2.0;
        let new_y = (-SQRT3 * p.x - p.y) / 2.0;
        p = vec2(new_x, new_y);
    }

    let clamped = if p.x < -2.0 {
        -2.0
    } else if p.x > 0.0 {
        0.0
    } else {
        p.x
    };
    p.x -= clamped;

    let len = f32::sqrt(p.x * p.x + p.y * p.y);
    let sign = if p.y < 0.0 { 1.0 } else { -1.0 };
    -len * sign
}

/// Folds the 2D space to generate the fractal and returns the distance to it
fn sierpinski_triangle(uv: &mut Vec2, recursion_count: i32) -> f32 {
    let mut scale = 0.9;
    *uv = *uv * scale;

    for _ in 0..recursion_count {
        scale *= 2.0;
        *uv = *uv * 2.0;
        uv.y -= 2.0 * SQRT3 / 3.0;
        uv.x = f32::abs(uv.x);
        *uv = reflect_uv(*uv, vec2(1.0, -SQRT3 / 3.0), (11.0 / 6.0) * PI);
    }

    let d = signed_dist_triangle(*uv) / scale;
    *uv = *uv / scale;
    d
}

/// Attempt different iteration counts based on time modulo, since we can't use variable loops
fn sierpinski_with_iterations(uv: &mut Vec2, time: f32) -> f32 {
    // Animation: cycle through 0-8 iterations every 16 seconds (2 seconds per iteration)
    let iteration = ((time % 16.0) * 0.5) as i32;
    
    // Clamp to reasonable range
    let iter = if iteration > 8 { 8 } else { iteration };
    
    sierpinski_triangle(uv, iter)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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

    let mut uv_fractal = uv + vec2(0.0, 0.30); // Offset to center the fractal

    // Get distance to fractal with animated iterations
    let d = sierpinski_with_iterations(&mut uv_fractal, time);

    // Line smoothness based on resolution
    let line_smoothness = 3.0 / resolution.y;

    // Mask: 1.0 inside triangle, 0.0 outside (with smooth edge)
    let mask = smoothstep(line_smoothness, 0.0, d);

    // Only color inside the triangle with gradient
    if mask > 0.0 {
        // Gradient colors based on position
        let r = 0.3 + 0.4 * (uv2.x * 0.5 + 0.5);
        let g = 0.2 + 0.5 * (-uv2.x * 0.5 + 0.5);
        let b = 0.5 + 0.4 * (uv2.y * 0.5 + 0.5);

        *output = vec4(r * mask, g * mask, b * mask, 1.0);
    } else {
        // Black background
        *output = vec4(0.0, 0.0, 0.0, 1.0);
    }
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
