struct Uniforms {
    input_width: u32,
    input_height: u32,
    template_width: u32,
    template_height: u32,
};

@group(0)
@binding(0)
var<storage, read> input_buf: array<f32>;

@group(0)
@binding(1)
var<storage, read> template_buf: array<f32>;

@group(0)
@binding(2)
var<storage, read_write> result_buf: array<f32>;

@group(0)
@binding(3)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(8, 8, 1)
// Sum of squared errors
fn main_sqdiff(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var total_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var sqdiff = pow(input_val - template_val, 2.0);

            total_sum += sqdiff;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum;
}

@compute
@workgroup_size(8, 8, 1)
// Sum of squared errors normed
fn main_sqdiff_normed(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var total_sum = 0.0;
    var input_sq_sum = 0.0;
    var template_sq_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var sqdiff = pow(input_val - template_val, 2.0);

            total_sum += sqdiff;
            input_sq_sum += input_val * input_val;
            template_sq_sum += template_val * template_val;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum / sqrt(template_sq_sum * input_sq_sum); // TODO: make this normalization correct
}

@compute
@workgroup_size(8, 8, 1)
// Cross Correleation
fn main_ccorr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var total_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var cc = input_val * template_val;

            total_sum += cc;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum;
}

@compute
@workgroup_size(8, 8, 1)
// Cross Correleation normed
fn main_ccorr_normed(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var total_sum = 0.0;
    var input_sq_sum = 0.0;
    var template_sq_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var cc = input_val * template_val;
            input_sq_sum += input_val * input_val;
            template_sq_sum += template_val * template_val;

            total_sum += cc;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum / sqrt(input_sq_sum * template_sq_sum);
}

@compute
@workgroup_size(8, 8, 1)
// CCOEFF
fn main_ccoeff(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var total_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var cc = input_val * template_val;

            total_sum += cc;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum;
}

@compute
@workgroup_size(8, 8, 1)
// CCOEFF_NORMED
fn main_ccoeff_normed(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x = global_id.x;
    var y = global_id.y;

    var input_width = uniforms.input_width;
    var input_height = uniforms.input_height;

    var template_width = uniforms.template_width;
    var template_height = uniforms.template_height;

    var match_width = min(template_width, input_width - x);
    var match_height = min(template_height, input_height - y);

    var sum_squared_i = 0.0;
    var sum_squared_template = 0.0;
    var total_sum = 0.0;
    for (var i = 0u; i < match_width; i++) {
        for (var j = 0u; j < match_height; j++) {
            var input_idx = (y + j) * input_width + (i + x);
            var template_idx = j * template_width + i;

            var input_val = input_buf[input_idx];
            var template_val = template_buf[template_idx];

            var cc = input_val * template_val;

            total_sum += cc;
            sum_squared_i += input_val * input_val;
            sum_squared_template += template_val * template_val;
        }
    }

    var result_idx = y * (input_width - template_width + 1u) + x;
    result_buf[result_idx] = total_sum / sqrt(sum_squared_template * sum_squared_i);
}