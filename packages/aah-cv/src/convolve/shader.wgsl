struct Uniforms {
    image_width: u32,
    image_height: u32,
    kernel_width: u32,
    kernel_height: u32,
};

@group(0)
@binding(0)
var<storage, read_write> result_buf: array<f32>;

@group(0)
@binding(1)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(2)
var<storage, read> image_buf: array<f32>;

@group(0)
@binding(3)
var<storage, read> kernel_buf: array<f32>;

@compute
@workgroup_size(16, 16, 1)
// global_invocation_id:  workgroup_id * workgroup_size + local_invocation_id.
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var image_width = uniforms.image_width;
    var image_height = uniforms.image_height;

    var kernel_width = uniforms.kernel_width;
    var kernel_height = uniforms.kernel_height;

    var result_width = image_width - kernel_width + 1u;
    var result_height = image_height - kernel_height + 1u;

    if x >= result_width || y >= result_height {
        return;
    }

    var result_idx = y * result_width + x;
    result_buf[result_idx] = 0.0;
    for (var k = 0u; k < kernel_width; k++) {
        for (var l = 0u; l < kernel_height; l++) {
            var image_idx = (y + l) * image_width + (x + k);
            var kernel_idx = l * kernel_width + k;
            result_buf[result_idx] += image_buf[image_idx] * kernel_buf[kernel_idx];
        }
    }
}
