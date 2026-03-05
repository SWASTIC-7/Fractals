use mygraphics::wgpu_renderer::ShaderType;
use std::io::{self, Write};

fn select_shader() -> ShaderType {
    println!("Select a shader to run:\n");
    println!("  1. Sierpinski Triangle");
    println!("  2. Sierpinski Carpet");
    println!("  3. Koch Curve");
    println!("  4. Mandelbrot Set");
    println!("  5. Julia Set");
    println!("  6. Sierpinski Tetrahedron");
    println!("  7. Menger Sponge");
    println!("  8. Mandelbulb");
    println!("  9. Mandelbox");
    println!();

    loop {
        print!("Enter your choice (1-9): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        match input.trim() {
            "1" => return ShaderType::SierpinskiTriangle,
            "2" => return ShaderType::SierpinskiCarpet,
            "3" => return ShaderType::KochCurve,
            "4" => return ShaderType::MandelbrotSet,
            "5" => return ShaderType::JuliaSet,
            "6" => return ShaderType::SierpinskiTetrahedron,
            "7" => return ShaderType::MengerSponge,
            "8" => return ShaderType::Mandelbulb,
            "9" => return ShaderType::Mandelbox,
            _ => println!("Invalid choice. Please enter 1-9."),
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let shader_type = select_shader();
    println!("\nLaunching {}...\n", shader_type.name());
    mygraphics::wgpu_renderer::main(shader_type)
}
