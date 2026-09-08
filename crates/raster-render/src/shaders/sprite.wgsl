// Sprite rendering.
//
// One instance per sprite: the quad's four corners come from the vertex index,
// so no vertex buffer is needed — only the per-instance data below.

struct Camera {
    // Maps world space to clip space. Column-major, padded to 16-byte columns
    // because that is how WGSL aligns a mat3x3.
    view_projection: mat3x3<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var atlas: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct Instance {
    // Top-left corner in world space, already snapped to the pixel grid.
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    // The sprite's rectangle within the atlas, in normalised coordinates.
    @location(2) uv_min: vec2<f32>,
    @location(3) uv_max: vec2<f32>,
    @location(4) tint: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: Instance,
) -> VertexOutput {
    // Two triangles as a strip: (0,0) (1,0) (0,1) (1,1). Deriving the corner
    // from the index avoids a vertex buffer entirely.
    let corner = vec2<f32>(
        f32(vertex_index & 1u),
        f32((vertex_index >> 1u) & 1u),
    );

    let world = instance.position + corner * instance.size;
    let clip = camera.view_projection * vec3<f32>(world, 1.0);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip.xy, 0.0, 1.0);
    out.uv = mix(instance.uv_min, instance.uv_max, corner);
    out.tint = instance.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas, atlas_sampler, in.uv);
    let colour = sampled * in.tint;

    // A fully transparent pixel must not write to the depth or blend as if it
    // were there; discarding keeps sprite edges clean at any scale.
    if colour.a < 0.001 {
        discard;
    }
    return colour;
}
