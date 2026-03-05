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

const MAXIMUM_RAY_STEPS: i32 = 100;
const MAXIMUM_DISTANCE: f32 = 1000.0;
const MINIMUM_DISTANCE: f32 = 0.01;
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

/// Apply 2D rotation to xy components
fn rotate_xy(p: Vec3, angle: f32) -> Vec3 {
    let (c, s) = rotate_2d(angle);
    vec3(p.x * c - p.y * s, p.x * s + p.y * c, p.z)
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

/// Map value from one range to another
fn map_range(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    min2 + (value - min1) * (max2 - min2) / (max1 - min1)
}

/// Menger Sponge SDF using space folding
fn menger_sponge(mut z: Vec3, time: f32) -> f32 {
    let iterations = 25;
    // Animated scale - oscillates between 2.0 and 4.0
    let scale = 2.0 + (f32::sin(time / 2.0) + 1.0);
    let offset = vec3(3.0, 3.0, 3.0);
    let bailout = 1000.0;
    
    let mut r = z.length();
    let mut n = 0;
    
    while n < iterations && r < bailout {
        // Take absolute values
        z.x = f32::abs(z.x);
        z.y = f32::abs(z.y);
        z.z = f32::abs(z.z);
        
        // Fold 1: if x - y < 0, swap x and y
        if z.x - z.y < 0.0 {
            let temp = z.x;
            z.x = z.y;
            z.y = temp;
        }
        // Fold 2: if x - z < 0, swap x and z
        if z.x - z.z < 0.0 {
            let temp = z.x;
            z.x = z.z;
            z.z = temp;
        }
        // Fold 3: if y - z < 0, swap y and z
        if z.y - z.z < 0.0 {
            let temp = z.y;
            z.y = z.z;
            z.z = temp;
        }
        
        // Scale and offset
        z.x = z.x * scale - offset.x * (scale - 1.0);
        z.y = z.y * scale - offset.y * (scale - 1.0);
        z.z = z.z * scale;
        
        // Conditional z offset
        if z.z > 0.5 * offset.z * (scale - 1.0) {
            z.z -= offset.z * (scale - 1.0);
        }
        
        r = z.length();
        n += 1;
    }
    
    // Return distance estimate
    (z.length() - 2.0) * f32::powf(scale, -(n as f32))
}

/// Distance estimator for the scene
fn distance_estimator(p: Vec3, time: f32) -> f32 {
    // Rotate the point for nice viewing angle
    let p = rotate_yz(p, 0.2 * PI);
    let p = rotate_xy(p, 0.3 * PI);
    let p = rotate_xz(p, 0.29 * PI);
    menger_sponge(p, time)
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
        let distance = distance_estimator(p, time);
        cur_pos = p;
        
        if min_dist_to_scene > distance {
            min_dist_to_scene = distance;
            min_dist_to_scene_pos = cur_pos;
        }
        
        total_distance += distance;
        steps = i as f32;
        
        if distance < MINIMUM_DISTANCE {
            hit = true;
            break;
        }
        if distance > MAXIMUM_DISTANCE {
            break;
        }
    }
    
    // Coloring
    let col: Vec3;
    
    if hit {
        // Hit the fractal - color based on distance from origin
        let hsv = vec3(0.8 + cur_pos.length() / 8.0, 1.0, 0.8);
        col = hsv_to_rgb(hsv);
    } else {
        // Glow effect for near-misses
        let hsv = vec3(0.8 + min_dist_to_scene_pos.length() / 8.0, 1.0, 0.8);
        let mut glow = hsv_to_rgb(hsv);
        glow = glow * (1.0 / min_dist_to_scene.max(0.01));
        let time_mod = map_range(f32::sin(time * 3.0), -1.0, 1.0, 1.0, 3.0);
        glow = glow / (15.0 * time_mod);
        col = glow;
    }
    
    // Apply ambient occlusion based on steps
    let iterations_factor = (steps + 1.0) / 10.0;
    let mut final_col = col / iterations_factor;
    
    // Distance attenuation
    let dist_to_scene = (ro - min_dist_to_scene_pos).length();
    final_col = final_col / (dist_to_scene * dist_to_scene).max(0.01);
    
    // Brightness boost
    final_col = final_col * 2000.0;
    
    // Clamp to valid range
    vec4(
        final_col.x.clamp(0.0, 1.0),
        final_col.y.clamp(0.0, 1.0),
        final_col.z.clamp(0.0, 1.0),
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
    let mut uv = (frag_coord - 0.5 * resolution) / resolution.y;
    uv = uv * 0.2;
    uv.y -= 0.015;
    
    // Camera setup - orbiting around the origin
    let mut ro = vec3(-40.0, 30.1, -10.0);
    
    // Horizontal rotation based on time (slower rotation)
    ro = rotate_xz(ro, -time * 2.0 * PI / 20.0);
    
    // Ray direction towards origin
    let rd = get_ray_direction(uv, ro, vec3(0.0, 1.0, 0.0), 1.0);
    
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
