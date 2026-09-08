// Scales the low-resolution image up to the window.
//
// A full-screen triangle rather than a quad: three vertices instead of four,
// no vertex buffer, and no seam down the diagonal where two triangles meet.

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // Un triangle qui deborde de l'ecran : (-1,-1), (3,-1), (-1,3). La partie
    // visible couvre exactement le viewport.
    let x = f32(i32(index) / 2) * 4.0 - 1.0;
    let y = f32(i32(index) & 1) * 4.0 - 1.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // Y inverse : l'espace de clip monte, les coordonnees de texture
    // descendent.
    out.uv = vec2<f32>((x + 1.0) * 0.5, 1.0 - (y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source, source_sampler, in.uv);
}
