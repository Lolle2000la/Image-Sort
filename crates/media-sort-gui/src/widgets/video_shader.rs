use iced::advanced::graphics::Viewport;
use iced::advanced::mouse;
use iced::{Element, Length, Rectangle};
use iced_wgpu::primitive::{Pipeline, Primitive};
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;

const SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    return out;
}

@group(0) @binding(0)
var r_texture: texture_2d<f32>;
@group(0) @binding(1)
var r_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(r_texture, r_sampler, in.tex_coords);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] },
];

pub struct VideoPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    width: u32,
    height: u32,
}

impl Pipeline for VideoPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("video_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_WGSL)),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("video_quad_vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("video_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("video_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            cache: None,
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            vertex_buffer,
            sampler,
            bind_group_layout,
            texture: None,
            bind_group: None,
            width: 0,
            height: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoPrimitive {
    pub width: u32,
    pub height: u32,
    pub rgba: Option<std::sync::Arc<Vec<u8>>>,
}

impl Primitive for VideoPrimitive {
    type Pipeline = VideoPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let size_changed = pipeline.width != self.width || pipeline.height != self.height || pipeline.texture.is_none();

        if size_changed {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("video_frame_texture"),
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("video_bind_group"),
                layout: &pipeline.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                    },
                ],
            });

            pipeline.texture = Some(texture);
            pipeline.bind_group = Some(bind_group);
            pipeline.width = self.width;
            pipeline.height = self.height;
        }

        if let Some(texture) = &pipeline.texture {
            if let Some(rgba) = &self.rgba {
                if !rgba.is_empty() {
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        rgba,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(self.width * 4),
                            rows_per_image: Some(self.height),
                        },
                        wgpu::Extent3d {
                            width: self.width,
                            height: self.height,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        if let Some(bind_group) = &pipeline.bind_group {
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..4, 0..1);
            true
        } else {
            false
        }
    }
}

pub struct VideoProgram {
    pub width: u32,
    pub height: u32,
    pub rgba: Option<std::sync::Arc<Vec<u8>>>,
}

impl<Message> iced::widget::shader::Program<Message> for VideoProgram {
    type State = ();
    type Primitive = VideoPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        VideoPrimitive {
            width: self.width,
            height: self.height,
            rgba: self.rgba.clone(),
        }
    }
}

pub fn video_shader_view<'a, Message: 'a, Theme: 'a, Renderer: 'a>(
    width: u32,
    height: u32,
    rgba: Option<std::sync::Arc<Vec<u8>>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: iced_wgpu::primitive::Renderer,
{
    iced::widget::Shader::new(VideoProgram {
        width,
        height,
        rgba,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
