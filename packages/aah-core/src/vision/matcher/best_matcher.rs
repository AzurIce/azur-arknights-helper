use image::DynamicImage;

pub struct BestMatcherResult {

}

pub struct BestMatcher {
    images: Vec<DynamicImage>,
    template: DynamicImage,
}

impl BestMatcher {
    pub fn new(images: Vec<DynamicImage>, template: DynamicImage) -> Self {
        Self { images, template }
    }
}