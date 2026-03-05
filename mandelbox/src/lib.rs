#![no_std]

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4, vec2, vec3, vec4};
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

const MAXIMUM_RAY_STEPS: i32 = 200;
const MAXIMUM_DISTANCE: f32 = 100.0;
const MINIMUM_DISTANCE: f32 = 0.0005;
const PI: f32 = 3.141592653589793;

/// 2D rotation matrix components
fn rotate_2d(angle: f32) -> (f32, f32) {
    (f32::cos(angle), f32::sin(angle))
}

/// Apply 2D rotation to yz components
fn rotate_yz(p: Vec3, angle: f32) -> Vec3 {
    let (c, s) = rotate_2d(angle);
    vec3(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
}

/// Apply 2D rotation to xz components
fn rotate_xz(p: Vec3, angle: f32) -> Vec3 {
    let (c, s) = rotate_2d(angle);
    vec3(p.x * c - p.z * s, p.y, p.x * s + p.z * c)
}

/// HSV to RGB color conversion
fn hsv_to_rgb(c: Vec3) -> Vec3 {
    let k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    
    let px = f32::abs(fract(c.x + k.x) * 6.0 - k.w);
    let py = f32::abs(fract(c.x + k.y) * 6.0 - k.w);
    let pz = f32::abs(fract(c.x + k.z) * 6.0 - k.w);
    
    let rx = c.z * mix(k.x, (px - k.x).clamp(0.0, 1.0), c.y);
    let ry = c.z * mix(k.x, (py - k.x).clamp(0.0, 1.0), c.y);
    let rz = c.z * mix(k.x, (pz - k.x).clamp(0.0, 1.0), c.y);
    
    vec3(rx, ry, rz)
}

/// Fractional part of a float
fn fract(x: f32) -> f32 {
    x - f32::floor(x)
}

/// Linear interpolation
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Sphere fold - folds space around a sphere with min/max radius protection
fn sphere_fold(z: Vec3, dr: f32, min_radius2: f32, fixed_radius2: f32) -> (Vec3, f32) {
    let r2 = z.x * z.x + z.y * z.y + z.z * z.z;
    
    if r2 < min_radius2 {
        // Inside minimum radius - scale to fixed radius
        let temp = fixed_radius2 / min_radius2;
        (z * temp, dr * temp)
    } else if r2 < fixed_radius2 {
        // Between min and fixed radius - scale to fixed radius
        let temp = fixed_radius2 / r2;
        (z * temp, dr * temp)
    } else {
        // Outside fixed radius - no change
        (z, dr)
    }
}

/// Box fold - folds space at box boundaries
fn box_fold(z: Vec3) -> Vec3 {
    vec3(
        z.x.clamp(-1.0, 1.0) * 2.0 - z.x,
        z.y.clamp(-1.0, 1.0) * 2.0 - z.y,
        z.z.clamp(-1.0, 1.0) * 2.0 - z.z,
    )
}

/// Mandelbox distance estimator - standard implementation
fn mandelbox(pos: Vec3) -> f32 {
    let scale = -2.0;  // Standard mandelbox scale
    let min_radius = 0.5;
    let fixed_radius = 1.0;
    let min_radius2 = min_radius * min_radius;
    let fixed_radius2 = fixed_radius * fixed_radius;
    
    let mut z = pos;
    let mut dr = 1.0;
    
    for _ in 0..15 {
        // Box fold
        z = box_fold(z);
        
        // Sphere fold
        let (new_z, new_dr) = sphere_fold(z, dr, min_radius2, fixed_radius2);
        z = new_z;
        dr = new_dr;
        
        // Scale and translate
        z = z * scale + pos;
        dr = dr * f32::abs(scale) + 1.0;
    }
    
    z.length() / f32::abs(dr)
}

/// Combined distance estimator
fn distance_estimator(pos: Vec3) -> f32 {
    mandelbox(pos)
}

/// Calculate ray direction based on UV and camera
fn get_ray_direction(uv: Vec2, ro: Vec3, look_at: Vec3, zoom: f32) -> Vec3 {
    let forward = (look_at - ro).normalize();
    let right = vec3(0.0, 1.0, 0.0).cross(forward).normalize();
    let up = forward.cross(right);
    let center = ro + forward * zoom;
    let intersection = center + uv.x * right + uv.y * up;
    (intersection - ro).normalize()
}

/// Ray marcher - returns color
fn ray_march(ro: Vec3, rd: Vec3, time: f32) -> Vec4 {
    let mut total_distance = 0.0;
    let mut min_dist_to_scene = 100.0;
    let mut min_dist_to_scene_pos = ro;
    let mut cur_pos = ro;
    let mut hit = false;
    let mut steps = 0.0;
    
    for i in 0..MAXIMUM_RAY_STEPS {
        let p = ro + total_distance * rd;
        let distance = distance_estimator(p);
        cur_pos = p;
        
        if min_dist_to_scene > distance {
            min_dist_to_scene = distance;
            min_dist_to_scene_pos = cur_pos;
        }
        
        total_distance += distance;
        steps = (i + 1) as f32;
        
        if distance < MINIMUM_DISTANCE {
            hit = true;
            break;
        }
        if distance > MAXIMUM_DISTANCE {
            break;
        }
    }
    
    // Coloring
    let mut col: Vec3;
    
    if hit {
        // Hit the fractal - color based on distance from origin
        let hsv = vec3(0.8 + cur_pos.length() / 10.0, 1.0, 0.8);
        col = hsv_to_rgb(hsv);
        
        // Apply ambient occlusion based on steps
        col = col / (steps * 0.08).max(0.01);
        
        // Distance attenuation
        let dist_to_scene = (ro - min_dist_to_scene_pos).length();
        col = col / (dist_to_scene * dist_to_scene).max(0.01);
        
        // Brightness boost
        col = col * 20.0;
    } else {
        // Glow effect for near misses
        let hsv = vec3(0.8 + min_dist_to_scene_pos.length() / 10.0, 1.0, 0.8);
        col = hsv_to_rgb(hsv);
        
        // Glow based on how close we got
        let glow = 1.0 / (min_dist_to_scene * min_dist_to_scene + 0.001);
        col = col * glow * 0.001;
        
        // Pulsing glow
        let pulse = 0.5 + 0.5 * f32::sin(time * 2.0);
        col = col * (0.5 + pulse * 0.5);
    }
    
    // Clamp to valid range
    vec4(
        col.x.clamp(0.0, 1.0),
        col.y.clamp(0.0, 1.0),
        col.z.clamp(0.0, 1.0),
        1.0,
    )
}

#[spirv(fragment)]
pub fn main_fs(
    frag_coord: Vec2,
    #[spirv(flat)] resolution: Vec2,
    #[spirv(flat)] time: f32,
    output: &mut Vec4,
) {
    // Normalized pixel coordinates (centered, aspect-correct)
    let uv = (frag_coord - 0.5 * resolution) / resolution.y;
    
    // Camera setup - orbiting view at distance 4 from origin
    let mut ro = vec3(0.0, 0.0, -7.0);
    
    // Animate camera rotation
    ro = rotate_yz(ro, (time * PI + 1.0) / 20.0);
    ro = rotate_xz(ro, time * 2.0 * PI / 10.0);
    
    // Ray direction towards origin (where the fractal is)
    let rd = get_ray_direction(uv, ro, vec3(0.0, 0.0, 0.0), 1.0);
    
    // March the ray
    *output = ray_march(ro, rd, time);
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
