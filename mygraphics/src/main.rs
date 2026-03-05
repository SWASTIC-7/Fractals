use mygraphics::wgpu_renderer::ShaderType;
use std::io::{self, Write};

fn select_shader() -> ShaderType {
    println!("\n=== Rust GPU Fractal Shaders ===\n");
    println!("Select a shader to run:\n");
    println!("  1. Sierpinski Triangle");
    println!("  2. Sierpinski Carpet");
    println!();

    loop {
        print!("Enter your choice (1-2): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        match input.trim() {
            "1" => return ShaderType::SierpinskiTriangle,
            "2" => return ShaderType::SierpinskiCarpet,
            _ => println!("Invalid choice. Please enter 1 or 2."),
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let shader_type = select_shader();
    println!("\nLaunching {}...\n", shader_type.name());
    mygraphics::wgpu_renderer::main(shader_type)
}
