use std::{
    borrow::Cow,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Clone, Debug)]
pub struct Image<'a> {
    pub data: Cow<'a, [f32]>,
    pub width: u32,
    pub height: u32,
}

impl<'a> Image<'a> {
    pub fn new(data: impl Into<Cow<'a, [f32]>>, width: u32, height: u32) -> Self {
        Self {
            data: data.into(),
            width,
            height,
        }
    }

    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    pub fn mean(&self) -> f32 {
        self.sum() / (self.width * self.height) as f32
    }

    pub fn replace_zero(&self, value: f32) -> Image<'_> {
        let data = self
            .data
            .iter()
            .map(|&a| if a == 0.0 { value } else { a })
            .collect::<Vec<f32>>()
            .into();
        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn min_value(&self, value: f32) -> Image<'_> {
        let data = self
            .data
            .iter()
            .map(|&a| a.min(value))
            .collect::<Vec<f32>>()
            .into();
        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn min(&self, other: Image<'_>) -> Image<'_> {
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a.min(b))
            .collect::<Vec<f32>>()
            .into();
        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn square(&self) -> Self {
        let data = self.data.iter().map(|v| v * v).collect::<Vec<f32>>().into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn sqrt(&self) -> Self {
        let data = self
            .data
            .iter()
            .map(|v| v.sqrt())
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl<'a> From<&'a image::ImageBuffer<image::Luma<f32>, Vec<f32>>> for Image<'a> {
    fn from(img: &'a image::ImageBuffer<image::Luma<f32>, Vec<f32>>) -> Self {
        Self {
            data: Cow::Borrowed(img),
            width: img.width(),
            height: img.height(),
        }
    }
}

// With f32
impl Add<f32> for Image<'_> {
    type Output = Image<'static>;

    fn add(self, rhs: f32) -> Self::Output {
        let data = self
            .data
            .iter()
            .map(|v| v + rhs)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl Sub<f32> for Image<'_> {
    type Output = Image<'static>;

    fn sub(self, rhs: f32) -> Self::Output {
        self.add(-rhs)
    }
}

impl Mul<f32> for Image<'_> {
    type Output = Image<'static>;

    fn mul(self, rhs: f32) -> Self::Output {
        let data = self
            .data
            .iter()
            .map(|v| v * rhs)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl Div<f32> for Image<'_> {
    type Output = Image<'static>;

    fn div(self, rhs: f32) -> Self::Output {
        self.mul(1.0 / rhs)
    }
}

impl Mul<Image<'_>> for f32 {
    type Output = Image<'static>;

    fn mul(self, rhs: Image<'_>) -> Self::Output {
        rhs * self
    }
}

// With Image
impl Mul<Image<'_>> for Image<'_> {
    type Output = Image<'static>;

    fn mul(self, rhs: Image<'_>) -> Self::Output {
        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| a * b)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl Div<Image<'_>> for Image<'_> {
    type Output = Image<'static>;

    fn div(self, rhs: Image<'_>) -> Self::Output {
        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| a / b)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl<'a> Add for Image<'a> {
    type Output = Image<'a>;

    fn add(self, other: Image<'a>) -> Self::Output {
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}

impl<'a> Sub for Image<'a> {
    type Output = Image<'a>;

    fn sub(self, other: Image<'a>) -> Self::Output {
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<f32>>()
            .into();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }
}
