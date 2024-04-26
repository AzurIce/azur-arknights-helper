

OpenCV 中的 **模板匹配** 位于 `imgproc` 模块下：[opencv/modules/imgproc/src/templmatch.cpp at 4.x · opencv/opencv (github.com)](https://github.com/opencv/opencv/blob/4.x/modules/imgproc/src/templmatch.cpp)。

OpenCV 会优先尝试加速实现，直接返回：

- `CV_OCL_RUN` OpenCL 加速实现
- `CV_IPP_RUN_FAST` Intel® Integrated Performance Primitives 加速实现

如果没有加速实现，就执行一个朴素实现。



模板匹配函数如下：

```cpp
void cv::matchTemplate( InputArray _img, InputArray _templ, OutputArray _result, int method, InputArray _mask )
```

要求图像深度为 `CV_8U` 或 `CV_32F`，且维数小于等于 2。

匹配方式 `method` 有如下六种：

- `TM_SQDIFF`
- `TM_SQDIFF_NORMED`
- `TM_CCORR`
- `TM_CCORR_NORMED`
- `TM_CCOEFF`
- `TM_CCOEFF_NORMED`

会首先执行 `crossCorr(img, templ, result, Point(0,0), 0, 0);`

然后执行 `common_matchTemplate(img, templ, result, method, cn);`



corrSize 为 imgSize - templSize + 1



### 1. crossCorr

1. 计算 `blockSize`

    首先计算 `templ * blockScale` 四舍五入，

    然后再与 `minBlockSize - templSize + 1` 取最大。

```cpp
void crossCorr( const Mat& img, const Mat& _templ, Mat& corr,
                Point anchor, double delta, int borderType )
{
    const double blockScale = 4.5;
    const int minBlockSize = 256;
    std::vector<uchar> buf;

    Mat templ = _templ;
    int depth = img.depth(), cn = img.channels();
    int tdepth = templ.depth(), tcn = templ.channels();
    int cdepth = corr.depth(), ccn = corr.channels();

    CV_Assert( img.dims <= 2 && templ.dims <= 2 && corr.dims <= 2 );

    if( depth != tdepth && tdepth != std::max(CV_32F, depth) )
    {
        _templ.convertTo(templ, std::max(CV_32F, depth));
        tdepth = templ.depth();
    }

    CV_Assert( depth == tdepth || tdepth == CV_32F);
    CV_Assert( corr.rows <= img.rows + templ.rows - 1 &&
               corr.cols <= img.cols + templ.cols - 1 );

    CV_Assert( ccn == 1 || delta == 0 );

    int maxDepth = depth > CV_8S ? CV_64F : std::max(std::max(CV_32F, tdepth), cdepth);
    Size blocksize, dftsize;

    blocksize.width = cvRound(templ.cols*blockScale);
    blocksize.width = std::max( blocksize.width, minBlockSize - templ.cols + 1 );
    blocksize.width = std::min( blocksize.width, corr.cols );
    blocksize.height = cvRound(templ.rows*blockScale);
    blocksize.height = std::max( blocksize.height, minBlockSize - templ.rows + 1 );
    blocksize.height = std::min( blocksize.height, corr.rows );

    dftsize.width = std::max(getOptimalDFTSize(blocksize.width + templ.cols - 1), 2);
    dftsize.height = getOptimalDFTSize(blocksize.height + templ.rows - 1);
    if( dftsize.width <= 0 || dftsize.height <= 0 )
        CV_Error( cv::Error::StsOutOfRange, "the input arrays are too big" );

    // recompute block size
    blocksize.width = dftsize.width - templ.cols + 1;
    blocksize.width = MIN( blocksize.width, corr.cols );
    blocksize.height = dftsize.height - templ.rows + 1;
    blocksize.height = MIN( blocksize.height, corr.rows );

    Mat dftTempl( dftsize.height*tcn, dftsize.width, maxDepth );
    Mat dftImg( dftsize, maxDepth );

    int i, k, bufSize = 0;
    if( tcn > 1 && tdepth != maxDepth )
        bufSize = templ.cols*templ.rows*CV_ELEM_SIZE(tdepth);

    if( cn > 1 && depth != maxDepth )
        bufSize = std::max( bufSize, (blocksize.width + templ.cols - 1)*
            (blocksize.height + templ.rows - 1)*CV_ELEM_SIZE(depth));

    if( (ccn > 1 || cn > 1) && cdepth != maxDepth )
        bufSize = std::max( bufSize, blocksize.width*blocksize.height*CV_ELEM_SIZE(cdepth));

    buf.resize(bufSize);

    Ptr<hal::DFT2D> c = hal::DFT2D::create(dftsize.width, dftsize.height, dftTempl.depth(), 1, 1, CV_HAL_DFT_IS_INPLACE, templ.rows);

    // compute DFT of each template plane
    for( k = 0; k < tcn; k++ )
    {
        int yofs = k*dftsize.height;
        Mat src = templ;
        Mat dst(dftTempl, Rect(0, yofs, dftsize.width, dftsize.height));
        Mat dst1(dftTempl, Rect(0, yofs, templ.cols, templ.rows));

        if( tcn > 1 )
        {
            src = tdepth == maxDepth ? dst1 : Mat(templ.size(), tdepth, &buf[0]);
            int pairs[] = {k, 0};
            mixChannels(&templ, 1, &src, 1, pairs, 1);
        }

        if( dst1.data != src.data )
            src.convertTo(dst1, dst1.depth());

        if( dst.cols > templ.cols )
        {
            Mat part(dst, Range(0, templ.rows), Range(templ.cols, dst.cols));
            part = Scalar::all(0);
        }
        c->apply(dst.data, (int)dst.step, dst.data, (int)dst.step);
    }

    int tileCountX = (corr.cols + blocksize.width - 1)/blocksize.width;
    int tileCountY = (corr.rows + blocksize.height - 1)/blocksize.height;
    int tileCount = tileCountX * tileCountY;

    Size wholeSize = img.size();
    Point roiofs(0,0);
    Mat img0 = img;

    if( !(borderType & BORDER_ISOLATED) )
    {
        img.locateROI(wholeSize, roiofs);
        img0.adjustROI(roiofs.y, wholeSize.height-img.rows-roiofs.y,
                       roiofs.x, wholeSize.width-img.cols-roiofs.x);
    }
    borderType |= BORDER_ISOLATED;

    Ptr<hal::DFT2D> cF, cR;
    int f = CV_HAL_DFT_IS_INPLACE;
    int f_inv = f | CV_HAL_DFT_INVERSE | CV_HAL_DFT_SCALE;
    cF = hal::DFT2D::create(dftsize.width, dftsize.height, maxDepth, 1, 1, f, blocksize.height + templ.rows - 1);
    cR = hal::DFT2D::create(dftsize.width, dftsize.height, maxDepth, 1, 1, f_inv, blocksize.height);

    // calculate correlation by blocks
    for( i = 0; i < tileCount; i++ )
    {
        int x = (i%tileCountX)*blocksize.width;
        int y = (i/tileCountX)*blocksize.height;

        Size bsz(std::min(blocksize.width, corr.cols - x),
                 std::min(blocksize.height, corr.rows - y));
        Size dsz(bsz.width + templ.cols - 1, bsz.height + templ.rows - 1);
        int x0 = x - anchor.x + roiofs.x, y0 = y - anchor.y + roiofs.y;
        int x1 = std::max(0, x0), y1 = std::max(0, y0);
        int x2 = std::min(img0.cols, x0 + dsz.width);
        int y2 = std::min(img0.rows, y0 + dsz.height);
        Mat src0(img0, Range(y1, y2), Range(x1, x2));
        Mat dst(dftImg, Rect(0, 0, dsz.width, dsz.height));
        Mat dst1(dftImg, Rect(x1-x0, y1-y0, x2-x1, y2-y1));
        Mat cdst(corr, Rect(x, y, bsz.width, bsz.height));

        for( k = 0; k < cn; k++ )
        {
            Mat src = src0;
            dftImg = Scalar::all(0);

            if( cn > 1 )
            {
                src = depth == maxDepth ? dst1 : Mat(y2-y1, x2-x1, depth, &buf[0]);
                int pairs[] = {k, 0};
                mixChannels(&src0, 1, &src, 1, pairs, 1);
            }

            if( dst1.data != src.data )
                src.convertTo(dst1, dst1.depth());

            if( x2 - x1 < dsz.width || y2 - y1 < dsz.height )
                copyMakeBorder(dst1, dst, y1-y0, dst.rows-dst1.rows-(y1-y0),
                               x1-x0, dst.cols-dst1.cols-(x1-x0), borderType);

            if (bsz.height == blocksize.height)
                cF->apply(dftImg.data, (int)dftImg.step, dftImg.data, (int)dftImg.step);
            else
                dft( dftImg, dftImg, 0, dsz.height );

            Mat dftTempl1(dftTempl, Rect(0, tcn > 1 ? k*dftsize.height : 0,
                                         dftsize.width, dftsize.height));
            mulSpectrums(dftImg, dftTempl1, dftImg, 0, true);

            if (bsz.height == blocksize.height)
                cR->apply(dftImg.data, (int)dftImg.step, dftImg.data, (int)dftImg.step);
            else
                dft( dftImg, dftImg, DFT_INVERSE + DFT_SCALE, bsz.height );

            src = dftImg(Rect(0, 0, bsz.width, bsz.height));

            if( ccn > 1 )
            {
                if( cdepth != maxDepth )
                {
                    Mat plane(bsz, cdepth, &buf[0]);
                    src.convertTo(plane, cdepth, 1, delta);
                    src = plane;
                }
                int pairs[] = {0, k};
                mixChannels(&src, 1, &cdst, 1, pairs, 1);
            }
            else
            {
                if( k == 0 )
                    src.convertTo(cdst, cdepth, 1, delta);
                else
                {
                    if( maxDepth != cdepth )
                    {
                        Mat plane(bsz, cdepth, &buf[0]);
                        src.convertTo(plane, cdepth);
                        src = plane;
                    }
                    add(src, cdst, cdst);
                }
            }
        }
    }
}
```



### 2. common_matchTemplate











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

