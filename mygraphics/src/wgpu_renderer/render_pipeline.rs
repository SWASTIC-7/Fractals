use crate::wgpu_renderer::ShaderType;
use crate::wgpu_renderer::renderer::{GlobalBindGroup, GlobalBindGroupLayout};
use sierpinskie_triangle::ShaderConstants;
use wgpu::{
    ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, PushConstantRange,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState,
    include_spirv,
};

#[derive(Debug, Clone)]
pub struct MyRenderPipeline {
    pipeline: RenderPipeline,
}

impl MyRenderPipeline {
    pub fn new(
        device: &Device,
        global_bind_group_layout: &GlobalBindGroupLayout,
        out_format: TextureFormat,
        shader_type: ShaderType,
    ) -> anyhow::Result<Self> {
        // Load the appropriate shader based on selection
        let module = match shader_type {
            ShaderType::SierpinskiTriangle => {
                device.create_shader_module(include_spirv!(env!("SHADER_TRIANGLE_SPV")))
            }
            ShaderType::SierpinskiCarpet => {
                device.create_shader_module(include_spirv!(env!("SHADER_CARPET_SPV")))
            }
            ShaderType::KochCurve => {
                device.create_shader_module(include_spirv!(env!("SHADER_KOCH_SPV")))
            }
            ShaderType::MandelbrotSet => {
                device.create_shader_module(include_spirv!(env!("SHADER_MANDELBROT_SPV")))
            }
            ShaderType::JuliaSet => {
                device.create_shader_module(include_spirv!(env!("SHADER_JULIA_SPV")))
            }
        };

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("MyRenderPipeline layout"),
            bind_group_layouts: &[&global_bind_group_layout.0],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX_FRAGMENT,
                range: 0..size_of::<ShaderConstants>() as u32,
            }],
        });

        Ok(Self {
            pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("MyRenderPipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: Some("main_vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: Some("main_fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: out_format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            }),
        })
    }

    pub fn draw(&self, rpass: &mut RenderPass<'_>, global_bind_group: &GlobalBindGroup) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &global_bind_group.0, &[]);
        rpass.draw(0..6, 0..1);
    }
}
