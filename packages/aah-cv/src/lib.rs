//! aah-cv is a computer vision library for aah.
//! 
//! # Template matching
//! The [`template_matching`] module implements GPU accelerated template
//! matching algorithm.
//! 
//! The implemented matching methods include Sum of Squared Differences (SSD),
//! Cross Correlation (CC), Correlation Coefficient (CCoeff), and the normalized version of them.
//! 
//! ## Example
//! 
//! ```rust
//! use aah_cv::template_matching::{match_template, MatchTemplateMethod};
//! 
//! let image = image::open("path/to/image").unwrap();
//! let template = image::open("path/to/template").unwrap();
//! let res = match_template(image, template, MatchTemplateMethod::CorrelationCoefficientNormed);
//! let matches = find_matches(res, template.width, template.height, MatchTemplateMethod::CorrelationCoefficientNormed, 0.9)
//! ```
pub mod convolve;
pub mod gpu;
pub mod template_matching;
pub mod utils;
