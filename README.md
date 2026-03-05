# Fractal Shaders with Rust-GPU

A project for rendering fractal shapes using shaders written entirely in Rust via [Rust-GPU](https://github.com/Rust-GPU/rust-gpu).

## Overview

This project uses:
- **Rust-GPU** - Write GPU shaders in Rust, compiled to SPIR-V
- **wgpu** - Cross-platform graphics API for rendering
- **winit** - Window creation and event handling


## Prerequisites

- Rust (Edition 2024)
- Vulkan-compatible GPU and drivers

## How to Build & Run

```bash
# Build the shaders (compiles Rust to SPIR-V)
cargo gpu build

# Run the application
cargo run -p mygraphics

# Select a shader to run:

#   1. Sierpinski Triangle
#   2. Sierpinski Carpet
#   3. Koch Curve
#   4. Mandelbrot Set
#   5. Julia Set

# Enter your choice (1-5):
```

## Fractals to Implement

### 2D Fractals

| Fractal | Hausdorff Dimension | Status |
|---------|---------------------|--------|
| Sierpinski Triangle | $D = \log_{2}{3} \approx 1.585$ | Done |
| Sierpinski Carpet | $D = \log_{3}{8} \approx 1.893$ | Done |
| Koch Curve | $D = \log_{3}{4} \approx 1.262$ | Done |
| Mandelbrot Set | $D = 2$ | Done |
| Julia Set | $D = 2$ | Done |

### 3D Fractals (Ray Marching)

| Fractal | Hausdorff Dimension | Status |
|---------|---------------------|--------|
| Sierpinski Tetrahedron | $D = 2$ | Done |
| Menger Sponge | $D = \log_{3}{20} \approx 2.727$ | Done |
| Mandelbulb | $D = 3$ (conjectured) | Done |
| Mandelbox | Varies | Done |


## Credit
- [pedrotrschneider](https://github.com/pedrotrschneider/shader-fractals) has written fractal shaders in glsl
## Resources

- [Rust-GPU](https://github.com/Rust-GPU/rust-gpu) - Compile Rust to SPIR-V
- [wgpu](https://wgpu.rs/) - Safe Rust graphics API


## License

MIT