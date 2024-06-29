use nalgebra as na;

pub fn world_to_screen(
    camera_pos: na::Vector3<f32>,
    camera_euler: na::Vector3<f32>,
    world_pos: na::Vector3<f32>,
	screen_width: f32,
	screen_height: f32,
) -> (f32, f32) {
    let matrix = camera_matrix_from_trans(
        &camera_pos,
        &camera_euler,
        screen_height / screen_width,
        20.0_f32.to_radians(),
        1000.0,
        0.3,
    );
    let world_pos_homogeneous = na::Vector4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let result = matrix * world_pos_homogeneous;
    let mut result = result / result.w;
	result.x = -result.x;
	

	// println!("{:?}", result);
    let result = (result + na::Vector4::new(1.0, 1.0, 1.0, 1.0)) / 2.0;

    (
        (result.x * screen_width).round(),
        (result.y * screen_height).round(),
    )
	// (result.x, result.y)
}

// ? yxz
pub(crate) fn camera_euler_angles_xyz(side: bool) -> na::Vector3<f32> {
    if side {
        na::Vector3::new(30.0_f32.to_radians(), 10.0_f32.to_radians(), 0.0)
    } else {
        na::Vector3::new(30.0_f32.to_radians(), 0.0, 0.0)
    }
}

pub(crate) fn camera_matrix_from_trans(
    pos: &na::Vector3<f32>,
    euler: &na::Vector3<f32>,
    ratio: f32,
    fov_2_y: f32,
    far: f32,
    near: f32,
) -> na::Matrix4<f32> {
    let cos_y = euler.y.cos();
    let sin_y = euler.y.sin();
    let cos_x = euler.x.cos();
    let sin_x = euler.x.sin();
    let tan_f = fov_2_y.tan();

	#[rustfmt::skip]
    let translate = na::Matrix4::new(
        1.0, 0.0, 0.0, -pos.x,
		0.0, 1.0, 0.0, -pos.y,
		0.0, 0.0, 1.0, -pos.z,
		0.0, 0.0, 0.0, 1.0,
    );

	#[rustfmt::skip]
    let matrix_y = na::Matrix4::new(
        cos_y, 0.0, sin_y, 0.0,
		0.0, 1.0, 0.0, 0.0,
		-sin_y, 0.0, cos_y, 0.0,
		0.0, 0.0, 0.0, 1.0,
    );

	#[rustfmt::skip]
    let matrix_x = na::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
		0.0, cos_x, -sin_x, 0.0,
		0.0, sin_x, cos_x, 0.0,
		0.0, 0.0, 0.0, 1.0,
    );

	#[rustfmt::skip]
    let proj = na::Matrix4::new(
        ratio / tan_f, 0.0, 0.0, 0.0,
        0.0, 1.0 / tan_f, 0.0, 0.0,
        0.0, 0.0, -(far + near) / (far - near), -(2.0 * far * near) / (far - near),
        0.0, 0.0, -1.0, 0.0,
    );

    proj * matrix_x * matrix_y * translate
}
