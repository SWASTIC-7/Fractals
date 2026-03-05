# Fractal Shaders with Rust-GPU

A project for rendering fractal shapes using shaders written entirely in Rust via [Rust-GPU](https://github.com/Rust-GPU/rust-gpu).

## Overview

This project uses:
- **Rust-GPU** - Write GPU shaders in Rust, compiled to SPIR-V
- **wgpu** - Cross-platform graphics API for rendering
- **winit** - Window creation and event handling

## Project Structure

```
fractals/
├── mygraphics/          # Main application & wgpu renderer
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       └── wgpu_renderer/
├── sierpinskie-shaders/  # Rust-GPU shader code
│   └── src/
│       └── lib.rs       # Shader entry points (vertex/fragment)
├── sierpinskie_shaders.spv  # Compiled SPIR-V output
└── manifest.json        # Shader manifest
```

## Prerequisites

- Rust (Edition 2024)
- Vulkan-compatible GPU and drivers

## How to Build & Run

```bash
# Build the shaders (compiles Rust to SPIR-V)
cargo gpu build

# Run the application
cargo run -p mygraphics
```

## Fractals to Implement

### 2D Fractals

| Fractal | Hausdorff Dimension | Status |
|---------|---------------------|--------|
| Sierpinski Triangle | $D = \log_{2}{3} \approx 1.585$ | Planned |
| Sierpinski Carpet | $D = \log_{3}{8} \approx 1.893$ | Planned |
| Koch Curve | $D = \log_{3}{4} \approx 1.262$ | Planned |
| Mandelbrot Set | $D = 2$ | Planned |
| Julia Set | $D = 2$ | Planned |

### 3D Fractals (Ray Marching)

| Fractal | Hausdorff Dimension | Status |
|---------|---------------------|--------|
| Sierpinski Tetrahedron | $D = 2$ | Planned |
| Menger Sponge | $D = \log_{3}{20} \approx 2.727$ | Planned |
| Mandelbulb | $D = 3$ (conjectured) | Planned |
| Mandelbox | Varies | Planned |

## Writing Shaders in Rust

Shaders are written in [sierpinskie-shaders/src/lib.rs](sierpinskie-shaders/src/lib.rs) using `spirv-std`:

```rust
#![no_std]
use spirv_std::spirv;
use glam::{Vec3, Vec4};

#[spirv(fragment)]
pub fn main_fs(vtx_color: Vec3, output: &mut Vec4) {
    *output = Vec4::from((vtx_color, 1.0));
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] vtx_pos: &mut Vec4,
    vtx_color: &mut Vec3,
) {
    // Vertex shader logic
}
```

## Resources

- [Rust-GPU](https://github.com/Rust-GPU/rust-gpu) - Compile Rust to SPIR-V
- [wgpu](https://wgpu.rs/) - Safe Rust graphics API


## License

MIT