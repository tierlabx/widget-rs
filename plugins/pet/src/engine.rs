use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

use crate::model::{load_model_from_path, ModelVertex, Node};

pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update_view_proj(&mut self, camera_pos: glam::Vec3, camera_target: glam::Vec3, aspect: f32) {
        let view = glam::Mat4::look_at_rh(camera_pos, camera_target, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);
        self.view_proj = (proj * view).to_cols_array_2d();
    }
}

pub enum EngineCommand {
    LoadModel(String),
    SetFps(u32),
    UpdateVelocity(f32, f32),
    Touch(f32, f32),
}

pub struct PetEngine;

impl PetEngine {
    pub fn start(
        sender: SyncSender<Vec<u8>>,
        command_rx: Receiver<EngineCommand>,
        initial_model: String,
        initial_fps: u32,
    ) {
        std::thread::spawn(move || {
            pollster::block_on(run_engine(sender, command_rx, initial_model, initial_fps));
        });
    }
}

async fn run_engine(
    sender: SyncSender<Vec<u8>>,
    command_rx: Receiver<EngineCommand>,
    initial_model: String,
    initial_fps: u32,
) {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("MSAA Texture"),
        view_formats: &[],
    });
    let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("Depth Texture"),
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let u32_size = std::mem::size_of::<u32>() as u32;
    let output_buffer_size = (u32_size * WIDTH * HEIGHT) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(
        glam::vec3(0.0, 1.0, 3.0),
        glam::vec3(0.0, 1.0, 0.0),
        WIDTH as f32 / HEIGHT as f32,
    );

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
            label: Some("texture_bind_group_layout"),
        });

    let bone_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bone_bind_group_layout"),
        });

    let bone_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Bone Buffer"),
        size: (256 * std::mem::size_of::<[[f32; 4]; 4]>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bone_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bone_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: bone_buffer.as_entire_binding(),
        }],
        label: Some("bone_bind_group"),
    });

    let default_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("Default Texture"),
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &default_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[255, 255, 255, 255],
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let default_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let default_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&default_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&default_sampler),
            },
        ],
        label: Some("Default Bind Group"),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &camera_bind_group_layout,
            &texture_bind_group_layout,
            &bone_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[ModelVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_desc.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let mut model =
        load_model_from_path(&device, &queue, &texture_bind_group_layout, &initial_model);
    let program_start = Instant::now();
    let mut target_frametime = if initial_fps == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f32(1.0 / initial_fps as f32)
    };

    let mut velocity_x = 0.0;
    let mut velocity_y = 0.0;
    let mut drag_tilt_x = 0.0;
    let mut drag_tilt_z = 0.0;

    #[derive(PartialEq)]
    enum PetState {
        Idle,
        TouchHead,
        TouchBody,
    }
    let mut current_state = PetState::Idle;
    let mut state_timer = 0.0;

    let mut current_nod = 0.0;
    let mut current_shake = 0.0;

    loop {
        while let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                EngineCommand::LoadModel(path) => {
                    model =
                        load_model_from_path(&device, &queue, &texture_bind_group_layout, &path);
                }
                EngineCommand::SetFps(fps) => {
                    target_frametime = if fps == 0 {
                        Duration::ZERO
                    } else {
                        Duration::from_secs_f32(1.0 / fps as f32)
                    };
                }
                EngineCommand::UpdateVelocity(dx, dy) => {
                    velocity_x = dx;
                    velocity_y = dy;
                }
                EngineCommand::Touch(_x, y) => {
                    if y < 100.0 {
                        current_state = PetState::TouchHead;
                    } else {
                        current_state = PetState::TouchBody;
                    }
                    state_timer = 1.5; // touch animation lasts 1.5s
                }
            }
        }

        let start = Instant::now();
        let time = program_start.elapsed().as_secs_f32();

        let radius = -2.5;
        camera_uniform.update_view_proj(
            glam::vec3(0.0, 0.8, radius), // Camera directly in front
            glam::vec3(0.0, 0.8, 0.0),
            WIDTH as f32 / HEIGHT as f32,
        );
        queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        let target_tilt_z = (velocity_x * -0.015).clamp(-0.5, 0.5);
        let target_tilt_x = (velocity_y * -0.015).clamp(-0.5, 0.5);
        drag_tilt_z += (target_tilt_z - drag_tilt_z) * 0.15;
        drag_tilt_x += (target_tilt_x - drag_tilt_x) * 0.15;

        // Skeletal animation
        if let Some(m) = model.as_mut() {
            if let Some(spine_idx) = m.nodes.iter().position(|n| {
                n.name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains("spine")
            }) {
                let base = m.nodes[spine_idx].base_transform;
                // Breathing / swaying motion + drag physics
                let rot = glam::Mat4::from_rotation_z((time * 2.0).sin() * 0.05 + drag_tilt_z)
                    * glam::Mat4::from_rotation_x((time * 1.5).sin() * 0.05 + drag_tilt_x);
                m.nodes[spine_idx].local_transform = base * rot;
            }

            if let Some(head_idx) = m.nodes.iter().position(|n| {
                n.name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains("head")
                    || n.name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains("neck")
            }) {
                let base = m.nodes[head_idx].base_transform;

                let mut head_rot = glam::Mat4::from_rotation_y((time * 0.8).sin() * 0.3)
                    * glam::Mat4::from_rotation_x((time * 1.2).cos() * 0.15);

                let mut target_nod = 0.0;
                let mut target_shake = 0.0;

                if current_state == PetState::TouchHead {
                    // Nodding animation
                    target_nod = (state_timer * 10.0_f32).sin() * 0.3;
                } else if current_state == PetState::TouchBody {
                    // Shake head
                    target_shake = (state_timer * 15.0_f32).sin() * 0.4;
                }

                current_nod += (target_nod - current_nod) * 0.15;
                current_shake += (target_shake - current_shake) * 0.15;

                head_rot = glam::Mat4::from_rotation_x(current_nod)
                    * glam::Mat4::from_rotation_y(current_shake)
                    * head_rot;

                m.nodes[head_idx].local_transform = base * head_rot;
            }

            // Fix T-pose by putting arms down
            // Node names usually vary, we'll check common ones
            let arms = [
                (
                    vec!["upper arm_l", "leftupperarm", "arm_l"],
                    glam::Mat4::from_rotation_z(1.2),
                ),
                (
                    vec!["upper arm_r", "rightupperarm", "arm_r"],
                    glam::Mat4::from_rotation_z(-1.2),
                ),
            ];
            for (names, rot) in arms {
                if let Some(idx) = m.nodes.iter().position(|n| {
                    let node_name = n.name.as_deref().unwrap_or("").to_lowercase();
                    names.iter().any(|&name| node_name.contains(name))
                }) {
                    let base = m.nodes[idx].base_transform;
                    m.nodes[idx].local_transform = base * rot;
                }
            }

            let mut global_transforms = vec![glam::Mat4::IDENTITY; m.nodes.len()];

            fn compute_global(
                nodes: &[Node],
                idx: usize,
                parent_global: glam::Mat4,
                global_transforms: &mut [glam::Mat4],
            ) {
                let global = parent_global * nodes[idx].local_transform;
                global_transforms[idx] = global;
                for &child in &nodes[idx].children {
                    compute_global(nodes, child, global, global_transforms);
                }
            }

            for &root in &m.roots {
                compute_global(&m.nodes, root, glam::Mat4::IDENTITY, &mut global_transforms);
            }

            let mut joint_matrices = [[[0.0; 4]; 4]; 256];
            if !m.skins.is_empty() {
                let skin = &m.skins[0]; // use the first skin
                for (i, &joint_idx) in skin.joints.iter().enumerate() {
                    if i < 256 {
                        let global = global_transforms[joint_idx];
                        let ibm = skin.inverse_bind_matrices[i];
                        joint_matrices[i] = (global * ibm).to_cols_array_2d();
                    }
                }
            } else {
                for i in 0..256 {
                    joint_matrices[i] = glam::Mat4::IDENTITY.to_cols_array_2d();
                }
            }

            queue.write_buffer(&bone_buffer, 0, bytemuck::cast_slice(&joint_matrices));
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&texture_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(2, &bone_bind_group, &[]);

            if let Some(m) = &model {
                for submesh in &m.submeshes {
                    render_pass.set_vertex_buffer(0, submesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        submesh.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    let bg = if let Some(mat_idx) = submesh.material_index {
                        if mat_idx < m.materials.len() {
                            &m.materials[mat_idx].bind_group
                        } else {
                            &default_bind_group
                        }
                    } else {
                        &default_bind_group
                    };

                    render_pass.set_bind_group(1, bg, &[]);
                    render_pass.draw_indexed(0..submesh.num_elements, 0, 0..1);
                }
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * WIDTH),
                    rows_per_image: Some(HEIGHT),
                },
            },
            texture_desc.size,
        );

        queue.submit(Some(encoder.finish()));

        {
            let buffer_slice = output_buffer.slice(..);
            let (tx_map, rx_map) = futures::channel::oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx_map.send(result).unwrap();
            });
            device.poll(wgpu::Maintain::Wait);
            rx_map.await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();
            let result = data.to_vec();
            drop(data);
            output_buffer.unmap();

            if sender.send(result).is_err() {
                break;
            }
        }

        let elapsed = start.elapsed();
        let dt = elapsed.as_secs_f32().max(0.016);
        if state_timer > 0.0 {
            state_timer -= dt;
            if state_timer <= 0.0 {
                current_state = PetState::Idle;
            }
        }

        velocity_x *= 0.8; // friction/decay
        velocity_y *= 0.8;

        if elapsed < target_frametime {
            std::thread::sleep(target_frametime - elapsed);
        }
    }
}
