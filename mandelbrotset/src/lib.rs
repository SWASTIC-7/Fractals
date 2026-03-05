#![no_std]

use bytemuck::{Pod, Zeroable};
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

const RECURSION_LIMIT: i32 = 500;

/// Mandelbrot set iteration - returns the number of iterations before escaping
fn mandelbrot(c: Vec2, max_iter: i32) -> i32 {
    let mut z = vec2(0.0, 0.0);

    for i in 0..max_iter {
        // z = z^2 + c  (complex multiplication)
        let new_x = z.x * z.x - z.y * z.y + c.x;
        let new_y = 2.0 * z.x * z.y + c.y;
        z = vec2(new_x, new_y);

        // Check if escaped (|z| > 2)
        if z.x * z.x + z.y * z.y > 4.0 {
            return i;
        }
    }

    max_iter
}

/// Smoothstep interpolation
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
    // Zoom location (this is the famous seahorse valley)
    let location = vec2(-0.745428, 0.113009);

    // Normalized pixel coordinates (centered, aspect-correct)
    let uv = 2.0 * (frag_coord - 0.5 * resolution) / resolution.y;
    let uv2 = uv; // Copy for coloring

    // Calculate zoom based on time (exponential zoom)
    let zoom_time = time * 0.032;
    let zoom = f32::powf(zoom_time + 1.0, (zoom_time + 1.0) / 10.0);

    // Scale UV by zoom and offset to location
    let c = uv / zoom + location;

    // Calculate mandelbrot iterations
    let recursion_count = mandelbrot(c, RECURSION_LIMIT);

    // Put the amount of iterations in range [0, 1]
    let f = recursion_count as f32 / RECURSION_LIMIT as f32;

    // Coloring the fractal
    if recursion_count == RECURSION_LIMIT {
        // Inside the Mandelbrot set - black
        *output = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        // Outside - color based on iteration count with smooth coloring
        let ff = f32::powf(f, 1.0 - f * f32::max(0.0, 50.0 - time));
        let smoothness = 1.0;

        let r = smoothstep(0.0, smoothness, ff) * (uv2.x * 0.5 + 0.5);
        let b = smoothstep(0.0, smoothness, ff) * (uv2.y * 0.5 + 0.5);
        let g = smoothstep(0.0, smoothness, ff) * (-uv2.x * 0.5 + 0.5);

        *output = vec4(r, g, b, 1.0);
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
