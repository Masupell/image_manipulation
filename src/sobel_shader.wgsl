// Vertex

struct VertexOutput 
{
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput 
{
    let positions = array<vec2<f32>, 3>
    (
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return out;
}



// Fragment

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> 
{
    let size = textureDimensions(input_texture);
    let uv = frag_coord.xy / vec2<f32>(size);
    let color = textureSample(input_texture, my_sampler, uv);
    return color;
}