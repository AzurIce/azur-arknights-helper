```rust
pub fn m_match_template(image: &GrayImage, template: &GrayImage) -> Image<Luma<f32>> {
    use image::GenericImageView;

    let (image_width, image_height) = image.dimensions();
    let (template_width, template_height) = template.dimensions();

    assert!(
        image_width >= template_width,
        "image width must be greater than or equal to template width"
    );
    assert!(
        image_height >= template_height,
        "image height must be greater than or equal to template height"
    );

    let should_normalize = true;
    let image_squared_integral = if should_normalize {
        Some(integral_squared_image::<_, u64>(image))
    } else {
        None
    };
    let template_squared_sum = if should_normalize {
        Some(sum_squares(template))
    } else {
        None
    };

    let template = template.ref_ndarray2();
    println!("{:?}", image.dimensions());
    let image = image.ref_ndarray2();
    println!("{:?}", image.shape());

    let mut result: ImageBuffer<Luma<f32>, Vec<f32>> = Image::new(
        image_width - template_width + 1,
        image_height - template_height + 1,
    );

    result
        .mut_ndarray2()
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(y, mut row)| {
            for x in 0..row.len() {
                let mut score = 0f32;

                for dy in 0..template_height as usize {
                    for dx in 0..template_width as usize {
                        let image_value =
                            *image.get((y + dy, x + dx)).unwrap() as f32;
                        let template_value = *template.get((dy, dx)).unwrap() as f32;

                        score += image_value * template_value;
                    }
                }

                if let (&Some(ref i), &Some(t)) = (&image_squared_integral, &template_squared_sum) {
                    let region = imageproc::rect::Rect::at(x as i32, y as i32)
                        .of_size(template_width, template_height);
                    let norm = normalization_term(i, t, region);
                    if norm > 0.0 {
                        score /= norm;
                    }
                }
                row[x] = score;
            }
        });
    result
}
```

```rust
    result
        .mut_ndarray2()
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(y, mut row)| {
            for x in 0..row.len() {
                let mut score = template
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .enumerate()
                    .map(|(dy, row)| {
                        let mut score = 0f32;
                        for dx in 0..row.len() {
                            let image_value: f32 =
                                image.get((y + dy, x + dx)).unwrap().clone() as f32;
                            let template_value: f32 = row.get(dx).unwrap().clone() as f32;
                            score += image_value * template_value
                        }
                        score
                    })
                    .sum::<f32>();

                let mut score = 0f32; // 忘删了但是不影响测试时间
                
                if let (&Some(ref i), &Some(t)) = (&image_squared_integral, &template_squared_sum) {
                    let region = imageproc::rect::Rect::at(x as i32, y as i32)
                        .of_size(template_width, template_height);
                    let norm = normalization_term(i, t, region);
                    if norm > 0.0 {
                        score /= norm;
                    }
                }
                row[x] = score;
            }
        });

    result
}
```



```
#### testing device MUMU ####
testing EnterMissionMistCity.png on main.png...
[Matcher::TemplateMatcher]: image: 2560x1440, template: 159x158, template: CrossCorrelationNormalized, matching...
(2560, 1440)
[1440, 2560]
test vision::matcher::test::test_device_match has been running for over 60 seconds
finding_extremes...
[Matcher::TemplateMatcher]: cost: 468.95822s, min: 0.42804277, max: 0.9999335, loc: (865, 753)
[Matcher::TemplateMatcher]: success!
test vision::matcher::test::test_device_match ... ok
```



```
#### testing device MUMU ####
testing EnterMissionMistCity.png on main.png...
[Matcher::TemplateMatcher]: image: 2560x1440, template: 159x158, template: CrossCorrelationNormalized, matching...
(2560, 1440)
[1440, 2560]
test vision::matcher::test::test_device_match has been running for over 60 seconds
finding_extremes...
[Matcher::TemplateMatcher]: cost: 464.04495s, min: 0.0, max: 0.0, loc: (0, 0)
[Matcher::TemplateMatcher]: failed
test vision::matcher::test::test_device_match ... ok
```

