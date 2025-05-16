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

@group(0) @binding(0) var img_texture: texture_2d<f32>;
@group(0) @binding(1) var img_sampler: sampler;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> 
{
    let tex_size = vec2<f32>(textureDimensions(img_texture));
    let uv = frag_coord.xy / tex_size;
    let input_texture = textureSample(img_texture, img_sampler, uv);
    
    // Sobel kernels
    let gx = array<array<i32, 3>, 3>
    (
        array(-1,  0,  1),
        array(-2,  0,  2),
        array(-1,  0,  1)
    );
    
    let gy = array<array<i32, 3>, 3>
    (
        array(-1, -2, -1),
        array( 0,  0,  0),
        array( 1,  2,  1)
    );

    var gx_sum: f32 = 0.0;
    var gy_sum: f32 = 0.0;
    
    // Surrounding pixels
    for (var y: i32 = -1; y <= 1; y = y + 1) 
    {
        for (var x: i32 = -1; x <= 1; x = x + 1) 
        {
            let offset = vec2<f32>(f32(x), f32(y)) / tex_size;
            let sample_color = textureSample(img_texture, img_sampler, uv + offset);
            let brightness = dot(sample_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
            
            gx_sum = gx_sum + f32(gx[y + 1][x + 1]) * brightness;
            gy_sum = gy_sum + f32(gy[y + 1][x + 1]) * brightness;
        }
    }
    
    // Gradient magnitude
    let edge_strength = gx_sum * gx_sum + gy_sum * gy_sum; // Actually needs square toot for gradient magnitude, but just use the cube in threshold instead
    
    // Normalize (assuming a max expected value)
    let threshold: f32 = 0.01;
    // let edge = select(vec4(0.0, 0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), edge_strength > threshold);
    let edge = select(vec4(0.0, 0.0, 0.0, 1.0), vec4(input_texture.rgb, 1.0), edge_strength > threshold);
    
    return edge;
}