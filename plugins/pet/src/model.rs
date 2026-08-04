use glam::Mat4;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

impl ModelVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct ModelMaterial {
    pub bind_group: wgpu::BindGroup,
}

pub struct Submesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material_index: Option<usize>,
    #[allow(dead_code)]
    pub skin_index: Option<usize>,
}

pub struct Node {
    pub name: Option<String>,
    pub local_transform: Mat4,
    pub base_transform: Mat4,
    #[allow(dead_code)]
    pub global_transform: Mat4,
    pub children: Vec<usize>,
}

pub struct Skin {
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

pub struct Model {
    pub submeshes: Vec<Submesh>,
    pub materials: Vec<ModelMaterial>,
    pub nodes: Vec<Node>,
    pub skins: Vec<Skin>,
    pub roots: Vec<usize>,
}

pub fn load_model_from_path(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    path: &str,
) -> Option<Model> {
    if !std::path::Path::new(path).exists() {
        println!("Model file not found: {}", path);
        return None;
    }
    let (gltf, buffers, images) = gltf::import(path).ok()?;

    let mut materials = Vec::new();

    for image_data in images {
        let rgba_pixels = match image_data.format {
            gltf::image::Format::R8G8B8 => {
                let mut rgba =
                    Vec::with_capacity(image_data.width as usize * image_data.height as usize * 4);
                for rgb in image_data.pixels.chunks(3) {
                    rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
                rgba
            }
            gltf::image::Format::R8G8B8A8 => image_data.pixels.clone(),
            _ => vec![255; image_data.width as usize * image_data.height as usize * 4],
        };

        let texture_size = wgpu::Extent3d {
            width: image_data.width,
            height: image_data.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("GLTF Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba_pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_data.width),
                rows_per_image: Some(image_data.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Material Bind Group"),
        });

        materials.push(ModelMaterial { bind_group });
    }

    // Nodes
    let mut nodes = Vec::new();
    for node in gltf.nodes() {
        let (t, r, s) = node.transform().decomposed();
        let translation = glam::Vec3::from(t);
        let rotation = glam::Quat::from_array(r);
        let scale = glam::Vec3::from(s);
        let local_transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);

        let children: Vec<usize> = node.children().map(|c| c.index()).collect();

        nodes.push(Node {
            name: node.name().map(|s| s.to_string()),
            local_transform,
            base_transform: local_transform,
            global_transform: Mat4::IDENTITY,
            children,
        });
    }

    // Roots
    let mut roots = Vec::new();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            roots.push(node.index());
        }
    }

    // Skins
    let mut skins = Vec::new();
    for skin in gltf.skins() {
        let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
        let mut inverse_bind_matrices = vec![Mat4::IDENTITY; joints.len()];

        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(iter) = reader.read_inverse_bind_matrices() {
            for (i, matrix) in iter.enumerate() {
                inverse_bind_matrices[i] = Mat4::from_cols_array_2d(&matrix);
            }
        }

        skins.push(Skin {
            joints,
            inverse_bind_matrices,
        });
    }

    let mut submeshes = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let mut vertices = Vec::new();

            let mut positions = Vec::new();
            if let Some(iter) = reader.read_positions() {
                positions.extend(iter);
            }

            let mut normals = Vec::new();
            if let Some(iter) = reader.read_normals() {
                normals.extend(iter);
            }

            let mut tex_coords = Vec::new();
            if let Some(read_tex_coords) = reader.read_tex_coords(0) {
                tex_coords.extend(read_tex_coords.into_f32());
            }

            let mut joints = Vec::new();
            if let Some(iter) = reader.read_joints(0) {
                joints.extend(iter.into_u16());
            }

            let mut weights = Vec::new();
            if let Some(iter) = reader.read_weights(0) {
                weights.extend(iter.into_f32());
            }

            let vertex_count = positions.len();
            for i in 0..vertex_count {
                let j = joints.get(i).copied().unwrap_or([0, 0, 0, 0]);
                let w = weights.get(i).copied().unwrap_or([1.0, 0.0, 0.0, 0.0]);
                vertices.push(ModelVertex {
                    position: positions[i],
                    normal: normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                    tex_coords: tex_coords.get(i).copied().unwrap_or([0.0, 0.0]),
                    joints: [j[0] as u32, j[1] as u32, j[2] as u32, j[3] as u32],
                    weights: w,
                });
            }

            let mut indices = Vec::new();
            if let Some(iter) = reader.read_indices() {
                indices.extend(iter.into_u32());
            } else {
                indices.extend(0..vertex_count as u32);
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let material_index = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|t| t.texture().source().index());

            // Node holding this mesh has the skin index.
            // We need to find which node points to this mesh to get the skin.
            let mut skin_index = None;
            for node in gltf.nodes() {
                if let Some(m) = node.mesh() {
                    if m.index() == mesh.index() {
                        if let Some(s) = node.skin() {
                            skin_index = Some(s.index());
                        }
                        break;
                    }
                }
            }

            submeshes.push(Submesh {
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
                material_index,
                skin_index,
            });
        }
    }

    Some(Model {
        submeshes,
        materials,
        nodes,
        skins,
        roots,
    })
}
