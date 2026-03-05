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
const PI: f32 = 3.141592653589793;

/// Julia set constants - different c values create different Julia sets
const JULIA_CONSTANTS: [(f32, f32); 6] = [
    (-0.7176, -0.3842),
    (-0.4, -0.59),
    (0.34, -0.05),
    (0.355, 0.355),
    (-0.54, 0.54),
    (0.355534, -0.337292),
];

/// Julia set iteration - returns the number of iterations before escaping
fn julia_set(z_initial: Vec2, constant: Vec2, max_iter: i32) -> i32 {
    let mut z = z_initial;

    for i in 0..max_iter {
        // z = z^2 + c (complex multiplication)
        let new_x = z.x * z.x - z.y * z.y + constant.x;
        let new_y = 2.0 * z.x * z.y + constant.y;
        z = vec2(new_x, new_y);

        // Check if escaped (|z| > 2)
        if z.x * z.x + z.y * z.y > 4.0 {
            return i;
        }
    }

    max_iter
}

/// Smoothstep function for smooth color transitions
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
    // Cycle through different Julia set constants every 5 seconds
    let constant_index = ((time / 5.0) as i32 % 6) as usize;
    let constant = vec2(
        JULIA_CONSTANTS[constant_index].0,
        JULIA_CONSTANTS[constant_index].1,
    );

    // Normalized pixel coordinates (centered, aspect-correct)
    let uv = 2.0 * (frag_coord - 0.5 * resolution) / resolution.y;
    let uv2 = uv; // Copy for coloring

    // Rotation angle (60 degrees)
    let a = PI / 3.0;
    let cos_a = f32::cos(a);
    let sin_a = f32::sin(a);
    let u_basis = vec2(cos_a, sin_a);
    let v_basis = vec2(-sin_a, cos_a);

    // Rotate UV
    let uv_rotated = vec2(
        uv.x * u_basis.x + uv.y * u_basis.y,
        uv.x * v_basis.x + uv.y * v_basis.y,
    );
    let uv_scaled = uv_rotated * 0.9;

    // Julia set calculation
    let recursion_count = julia_set(uv_scaled, constant, RECURSION_LIMIT);

    // Coloring
    if recursion_count == RECURSION_LIMIT {
        // Inside the Julia set - black
        *output = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        let f = recursion_count as f32 / RECURSION_LIMIT as f32;
        let ff = f32::powf(f, 1.0 - f);

        // Threshold for boundary effect
        let threshold = 0.05;
        
        if f < threshold {
            // Far from boundary - black background
            *output = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            // Near boundary - apply position-based gradient
            let intensity = smoothstep(0.0, 1.0, ff);
            let r = intensity * (uv2.x * 0.5 + 0.3);
            let b = intensity * (uv2.y * 0.5 + 0.3);
            let g = intensity * (-uv2.x * 0.5 + 0.3);

            // Amplify colors
            let saturation = 5.0;
            *output = vec4(
                (r * saturation).clamp(0.0, 1.0),
                (g * saturation).clamp(0.0, 1.0),
                (b * saturation).clamp(0.0, 1.0),
                1.0,
            );
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
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *vtx_pos = pos.extend(0.0).extend(1.0);
    *frag_coord = vec2(uv.x * constants.width as f32, uv.y * constants.height as f32);
    *resolution = vec2(constants.width as f32, constants.height as f32);
    *time = constants.time;
}
