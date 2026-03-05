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

    // Calculate zoom based on time (smooth exponential zoom)
    // Slower rate (0.08) for smoother zoom experience
    let zoom = f32::exp(time * 0.08);

    // Scale UV by zoom and offset to location
    let c = uv / zoom + location;

    // Calculate mandelbrot iterations - increase with zoom for more detail
    let max_iter = RECURSION_LIMIT.min((50.0 + f32::log2(zoom + 1.0) * 50.0) as i32);
    let recursion_count = mandelbrot(c, max_iter);

    // Coloring the fractal
    if recursion_count == max_iter {
        // Inside the Mandelbrot set - black
        *output = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        // Outside - only color near the boundary (high iteration counts)
        // Points that escape quickly (low iterations) should be black
        let normalized = recursion_count as f32 / max_iter as f32;
        
        // Only show color for points near the boundary (high iteration count)
        // This creates the "outline" effect
        let threshold = 0.1; // Points below this threshold are black
        
        if normalized < threshold {
            // Far from boundary - black background
            *output = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            // Near boundary - apply gradient based on position
            let intensity = (normalized - threshold) / (1.0 - threshold);
            let offset = 0.5;

            let r = intensity * (uv2.x * 0.5 + 0.5 + offset);
            let b = intensity * (uv2.y * 0.5 + 0.5 + offset);
            let g = intensity * (-uv2.x * 0.5 + 0.5 + offset);

            *output = vec4(r, g, b, 1.0);
        }
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
