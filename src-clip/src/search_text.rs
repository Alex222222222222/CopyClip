use std::{fmt, path};

use log::debug;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use once_cell::sync::OnceCell;
use rtf_parser::document;

use crate::ClipType;

const MAX_SEARCH_TEXT_LENGTH: u64 = 1000;
const HTML_READ_WIDTH: usize = 80;
static OCR_ENGINE: OnceCell<OcrEngine> = OnceCell::new();

#[derive(Debug, Clone, Copy)]
pub struct OcrEngineFullError;
impl fmt::Display for OcrEngineFullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OCR engine is already initialized")
    }
}
impl std::error::Error for OcrEngineFullError {}

#[derive(Debug, Clone, Copy)]
pub struct OcrEngineNotInitialisedError;
impl fmt::Display for OcrEngineNotInitialisedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OCR engine not initialized")
    }
}
impl std::error::Error for OcrEngineNotInitialisedError {}

/// Get the model in the given path, and feed the model into the ocr engine,
/// will return `OcrEngineFullError` if the model is already set.
pub fn init_search<T>(
    detection_model_path: T,
    recognition_model_path: T,
) -> Result<(), anyhow::Error>
where
    T: AsRef<path::Path>,
{
    let detection_model = rten::Model::load_file(detection_model_path)?;
    let recognition_model = rten::Model::load_file(recognition_model_path)?;
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    if OCR_ENGINE.set(engine).is_err() {
        let err = OcrEngineFullError {};
        return Err(err.into());
    }

    Ok(())
}

fn image_to_search_text(image: &str) -> Result<String, anyhow::Error> {
    let engine = match OCR_ENGINE.get() {
        Some(engine) => engine,
        None => return Err(OcrEngineNotInitialisedError {}.into()),
    };

    // see https://github.com/robertknight/ocrs/blob/main/ocrs/examples/hello_ocr.rs
    let image = image::open(image)?.into_rgb8();
    let image = ImageSource::from_bytes(image.as_raw(), image.dimensions())?;
    let image = engine.prepare_input(image)?;
    let word_rectangles = engine.detect_words(&image)?;
    let line_rectangles = engine.find_text_lines(&image, &word_rectangles);
    let line_texts = engine.recognize_text(&image, &line_rectangles)?;
    let mut text = String::new();
    for line_text in line_texts.into_iter().flatten() {
        text.push_str(format!("{}\n", line_text).as_str());
    }

    Ok(crate::trimming_clip_text(&text, MAX_SEARCH_TEXT_LENGTH))
}

fn rtf_to_search_text(rtf: &str) -> Result<String, anyhow::Error> {
    match document::RtfDocument::try_from(rtf) {
        Ok(document) => {
            let text = document.get_text();
            Ok(crate::trimming_clip_text(&text, MAX_SEARCH_TEXT_LENGTH))
        }
        Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}

fn html_to_search_text(html: &str) -> String {
    crate::trimming_clip_text(
        &html2text::from_read(html.as_bytes(), HTML_READ_WIDTH),
        MAX_SEARCH_TEXT_LENGTH,
    )
}

pub fn convert_text_to_search_text<T: Into<ClipType>>(
    clip_type: T,
    text: &str,
) -> Result<String, anyhow::Error> {
    debug!("Tring to covert to search text");
    match clip_type.into() {
        crate::ClipType::Text => Ok(crate::trimming_clip_text(text, MAX_SEARCH_TEXT_LENGTH)),
        crate::ClipType::Image => image_to_search_text(text),
        crate::ClipType::File => Ok(crate::trimming_clip_text(text, MAX_SEARCH_TEXT_LENGTH)),
        crate::ClipType::Html => Ok(html_to_search_text(text)),
        crate::ClipType::Rtf => rtf_to_search_text(text),
    }
}
