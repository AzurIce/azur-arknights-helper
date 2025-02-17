use image::DynamicImage;
use ocrs::ImageSource;

use crate::{controller::DEFAULT_HEIGHT, vision::utils::draw_box, AAH};

use super::Analyzer;

pub struct OcrAnalyzerOutput {
    pub screen: Box<DynamicImage>,
    // pub annotated_screen: Box<DynamicImage>,
}

pub struct OcrAnalyzer {}

impl OcrAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analyzer for OcrAnalyzer {
    type Output = OcrAnalyzerOutput;
    fn analyze(&mut self, core: &AAH) -> Result<Self::Output, String> {
        // Make sure that we are in the operation-start page
        println!("[OcrAnalyzer]: analyzing");

        // TODO: 并不是一个好主意，缩放大图消耗时间更多，且误差更大
        // TODO: 然而测试了一下，发现缩放模板有时也会导致误差较大 (333.9063)
        // let image = aah
        //     .controller
        //     .screencap_scaled()
        //     .map_err(|err| format!("{:?}", err))?;

        // Get image
        let screen = core
            .controller
            .screencap()
            .map_err(|err| format!("{:?}", err))?;

        let image = screen.to_rgb8();
        let img_source = ImageSource::from_bytes(&image.as_raw(), image.dimensions())
            .map_err(|err| format!("prepare image source error: {err}"))?;
        let ocr_input = core
            .ocr_engine
            .prepare_input(img_source)
            .map_err(|err| format!("prepare image error: {err}"))?;

        // Detect and recognize text. If you only need the text and don't need any
        // layout information, you can also use `engine.get_text(&ocr_input)`,
        // which returns all the text in an image as a single string.

        // Get oriented bounding boxes of text words in input image.
        let word_rects = core.ocr_engine.detect_words(&ocr_input).unwrap();

        // Group words into lines. Each line is represented by a list of word
        // bounding boxes.
        let line_rects = core.ocr_engine.find_text_lines(&ocr_input, &word_rects);

        // Recognize the characters in each line.
        let line_texts = core
            .ocr_engine
            .recognize_text(&ocr_input, &line_rects)
            .unwrap();

        for line in line_texts
            .iter()
            .flatten()
            // Filter likely spurious detections. With future model improvements
            // this should become unnecessary.
            .filter(|l| l.to_string().len() > 1)
        {
            println!("{}", line);
        }

        // Match
        // let res = SingleMatcher::Template {
        //     image: screen.to_luma32f(),
        //     template: template.to_luma32f(),
        //     threshold: None,
        // }
        // .result();

        // Annotated
        // let mut annotated_screen = screen.clone();
        // if let Some(rect) = &res.rect {
        //     draw_box(
        //         &mut annotated_screen,
        //         rect.x as i32,
        //         rect.y as i32,
        //         rect.width,
        //         rect.height,
        //         [255, 0, 0, 255],
        //     );
        // }

        let screen = Box::new(screen);
        // let annotated_screen = Box::new(annotated_screen);
        Ok(Self::Output {
            screen,
            // res,
            // annotated_screen,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{vision::analyzer::Analyzer, AAH};

    use super::OcrAnalyzer;

    #[test]
    fn test_ocr() {
        let aah = AAH::connect("127.0.0.1:16384", "../../resources", |_| {}).unwrap();
        let res = OcrAnalyzer::new().analyze(&aah);
    }
}
