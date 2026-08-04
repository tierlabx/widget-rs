struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct BoneUniform {
    matrices: array<mat4x4<f32>, 256>,
};
@group(2) @binding(0)
var<uniform> bones: BoneUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) joints: vec4<u32>,
    @location(4) weights: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // Skinning matrix computation
    var skin_mat = 
        model.weights.x * bones.matrices[model.joints.x] +
        model.weights.y * bones.matrices[model.joints.y] +
        model.weights.z * bones.matrices[model.joints.z] +
        model.weights.w * bones.matrices[model.joints.w];
        
    var skinned_position = skin_mat * vec4<f32>(model.position, 1.0);
    // for normals, we need the inverse transpose, but assuming uniform scaling, we can just cast to mat3
    var skinned_normal = (skin_mat * vec4<f32>(model.normal, 0.0)).xyz;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal = skinned_normal;
    out.clip_position = camera.view_proj * vec4<f32>(skinned_position.xyz, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if tex_color.a < 0.5 {
        discard;
    }
    
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.5));
    let normal = normalize(in.normal);
    
    // Half-lambert for a softer base
    let NdotL = dot(normal, light_dir) * 0.5 + 0.5;
    
    // Soft toon shading: instead of a hard if-statement, use smoothstep
    // It creates a smooth but tight transition between shadow and light
    let diffuse_step = mix(0.5, 1.0, smoothstep(0.4, 0.6, NdotL));
    
    let final_color = tex_color.rgb * diffuse_step;

    return vec4<f32>(final_color, tex_color.a);
}
