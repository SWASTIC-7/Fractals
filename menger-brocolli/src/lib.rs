#![no_std]

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4, vec3, vec4};
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

const MAX_STEPS: i32 = 80;
const MAX_DIST: f32 = 1000.0;
const MIN_DIST: f32 = 0.01;

/// HSV to RGB - exact GLSL port
fn hsv2rgb(c: Vec3) -> Vec3 {
    let k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = vec3(
        f32::abs(fract(c.x + k.x) * 6.0 - k.w),
        f32::abs(fract(c.x + k.y) * 6.0 - k.w),
        f32::abs(fract(c.x + k.z) * 6.0 - k.w),
    );
    vec3(
        c.z * lerp(k.x, clamp01(p.x - k.x), c.y),
        c.z * lerp(k.x, clamp01(p.y - k.x), c.y),
        c.z * lerp(k.x, clamp01(p.z - k.x), c.y),
    )
}

fn fract(x: f32) -> f32 {
    x - f32::floor(x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn clamp01(x: f32) -> f32 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

fn map_range(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    min2 + (value - min1) * (max2 - min2) / (max1 - min1)
}

/// 2x2 rotation matrix multiply on vec2(y,x) - GLSL: z.yx *= Rotate(angle)
fn rotate_yx(y: f32, x: f32, angle: f32) -> (f32, f32) {
    let c = f32::cos(angle);
    let s = f32::sin(angle);
    // mat2(c, -s, s, c) * vec2(y, x)
    (c * y - s * x, s * y + c * x)
}

/// Ray direction - R function from GLSL
fn ray_dir(uv: Vec2, ro: Vec3, look_at: Vec3, zoom: f32) -> Vec3 {
    let f = (look_at - ro).normalize();
    let r = vec3(0.0, 1.0, 0.0).cross(f).normalize();
    let u = f.cross(r);
    let c = ro + f * zoom;
    let i = c + uv.x * r + uv.y * u;
    (i - ro).normalize()
}

/// sierpinski3 - exact GLSL port
fn sierpinski3(mut z: Vec3, time: f32) -> f32 {
    let iterations = 20; // reduced from 25 for performance
    let scale = 1.0 + f32::sin(time) * 0.1 + 0.9;
    let offset = vec3(2.0, 4.8, 0.0);
    let bailout = 1000.0;

    let mut r = z.length();
    let mut n = 0;
    
    while n < iterations && r < bailout {
        // abs
        z.x = f32::abs(z.x);
        z.y = f32::abs(z.y);
        z.z = f32::abs(z.z);

        // fold 1: if (z.x - z.y < 0.0) z.xy = z.yx
        if z.x - z.y < 0.0 {
            let t = z.x; z.x = z.y; z.y = t;
        }
        // fold 2: if (z.x - z.z < 0.0) z.xz = z.zx
        if z.x - z.z < 0.0 {
            let t = z.x; z.x = z.z; z.z = t;
        }
        // fold 3: if (z.y - z.z < 0.0) z.zy = z.yz
        if z.y - z.z < 0.0 {
            let t = z.y; z.y = z.z; z.z = t;
        }

        // z.yx *= Rotate(0.436332 + sin(iTime * 0.9) * 0.1 + 4.9)
        let rot_angle = 0.436332 + f32::sin(time * 0.9) * 0.1 + 4.9;
        let (new_y, new_x) = rotate_yx(z.y, z.x, rot_angle);
        z.y = new_y;
        z.x = new_x;

        // scale and offset
        z.x = z.x * scale - offset.x * (scale - 1.0);
        z.y = z.y * scale - offset.y * (scale - 1.0);
        z.z = z.z * scale;

        if z.z > 0.5 * offset.z * (scale - 1.0) {
            z.z -= offset.z * (scale - 1.0);
        }

        r = z.length();
        n += 1;
    }

    (z.length() - 2.0) * f32::powf(scale, -(n as f32))
}

/// DistanceEstimator - with rotations from GLSL
fn distance_estimator(mut p: Vec3, time: f32) -> f32 {
    let pi = 3.14159265;
    
    // p.yz *= Rotate(0.2 * PI)
    let a1 = 0.2 * pi;
    let c1 = f32::cos(a1);
    let s1 = f32::sin(a1);
    let py = p.y * c1 - p.z * s1;
    let pz = p.y * s1 + p.z * c1;
    p.y = py;
    p.z = pz;

    // p.yx *= Rotate(0.3 * PI)
    let a2 = 0.3 * pi;
    let c2 = f32::cos(a2);
    let s2 = f32::sin(a2);
    let py2 = p.y * c2 - p.x * s2;
    let px2 = p.y * s2 + p.x * c2;
    p.y = py2;
    p.x = px2;

    // p.xz *= Rotate(0.29 * PI)
    let a3 = 0.29 * pi;
    let c3 = f32::cos(a3);
    let s3 = f32::sin(a3);
    let px3 = p.x * c3 - p.z * s3;
    let pz3 = p.x * s3 + p.z * c3;
    p.x = px3;
    p.z = pz3;

    sierpinski3(p, time)
}

/// RayMarcher - exact GLSL port with safety
fn ray_march(ro: Vec3, rd: Vec3, time: f32) -> Vec4 {
    let mut steps: f32 = 0.0;
    let mut total_dist: f32 = 0.0;
    let mut min_dist: f32 = 100.0;
    let mut min_dist_pos: Vec3 = ro;
    let mut cur_pos: Vec3 = ro;
    let mut hit = false;

    while (steps as i32) < MAX_STEPS {
        let p = ro + total_dist * rd;
        let dist = distance_estimator(p, time);
        cur_pos = p;
        
        if min_dist > dist {
            min_dist = dist;
            min_dist_pos = cur_pos;
        }
        
        // Ensure minimum step to prevent infinite loop
        total_dist += dist.max(0.001);
        
        if dist < MIN_DIST {
            hit = true;
            break;
        }
        if total_dist > MAX_DIST {
            break;
        }
        
        steps += 1.0;
    }

    // Coloring from GLSL
    let iterations = steps + f32::ln(f32::ln(MAX_DIST).max(1.0)) / f32::ln(2.0)
                   - f32::ln(f32::ln(cur_pos.dot(cur_pos).max(1.0)).max(0.001)) / f32::ln(2.0);
    let iterations = iterations.max(0.1);

    let mut col: Vec3;
    
    if hit {
        // col.rgb = vec3(0.8 + (length(curPos) / 8.0), 1.0, 0.8)
        let hsv = vec3(0.8 + cur_pos.length() / 8.0, 1.0, 0.8);
        col = hsv2rgb(hsv);
    } else {
        let hsv = vec3(0.8 + min_dist_pos.length() / 8.0, 1.0, 0.8);
        col = hsv2rgb(hsv);
        // col.rgb *= 1.0 / pow(minDistToScene, 1.0)
        col = col * (1.0 / min_dist.max(0.001));
        // col.rgb /= 15.0 * map(sin(iTime * 3.0), -1.0, 1.0, 1.0, 3.0)
        let pulsate = map_range(f32::sin(time * 3.0), -1.0, 1.0, 1.0, 3.0);
        col = col / (15.0 * pulsate);
    }

    // col.rgb /= iterations / 10.0 (ambient occlusion)
    col = col / (iterations / 10.0).max(0.01);

    // col.rgb /= pow(distance(ro, minDistToScenePos), 2.0)
    let d = (ro - min_dist_pos).length();
    col = col / (d * d).max(0.01);

    // col.rgb *= 2000.0
    col = col * 2000.0;

    // Clamp
    col.x = clamp01(col.x);
    col.y = clamp01(col.y);
    col.z = clamp01(col.z);

    vec4(col.x, col.y, col.z, 1.0)
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let x = ((vert_id & 1) * 4 - 1) as f32;
    let y = ((vert_id & 2) * 2 - 1) as f32;
    *out_pos = vec4(x, y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let res = Vec2::new(constants.width as f32, constants.height as f32);
    let time = constants.time;

    // Exact GLSL UV calculation
    let mut uv = (frag_coord.truncate().truncate() - 0.5 * res) / res.y;
    uv = uv * 0.2;
    uv.y -= 0.015;

    // Exact GLSL camera
    let ro = vec3(-40.0, 30.1, -10.0);
    let rd = ray_dir(uv, ro, vec3(0.0, 1.0, 0.0), 1.0);

    let col = ray_march(ro, rd, time);

    *output = col;
}
