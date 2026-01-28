use crate::diff;
use opencv::core;
use opencv::imgproc;
use opencv::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub sign: bool,
}

pub fn write_rects(rectangles: &[Rect], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_data = rmp_serde::to_vec(rectangles)?;
    fs::write(path, serialized_data)?;
    Ok(())
}

pub fn read_rects(path: &str) -> Result<Vec<Rect>, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let rectangles = rmp_serde::from_read(&file)?;
    Ok(rectangles)
}

pub fn frames2rect<const N: usize>(frames: &[Mat; N]) -> Result<(Rect, Mat), Box<dyn std::error::Error>> {
    let rows = frames[0].rows() as usize;
    let cols = frames[0].cols() as usize;
    let mut weights = vec![0i8; rows * cols];
    for row in 0..rows {
        for col in 0..cols {
            let orig_pixel = *frames[0].at_2d::<u8>(row as i32, col as i32)? as i16;
            for frame in &frames[1..] {
                let curr_pixel = *frame.at_2d::<u8>(row as i32, col as i32)? as i16;
                let diff = curr_pixel - orig_pixel;
                if diff > 0 {
                    weights[row * cols + col] += 1;
                } else if diff < 0 {
                    weights[row * cols + col] -= 1;
                }
            }
        }
    }
    let rectangle = diff::best_rect(&weights, rows, cols);
    let weight_mat = weight2mat::<N>(&weights, rows, cols)?;
    Ok((rectangle, weight_mat))
}

fn weight2mat<const N: usize>(weights: &[i8], rows: usize, cols: usize) -> Result<Mat, Box<dyn std::error::Error>> {
    let mut frame = Mat::new_rows_cols_with_default(rows as i32, cols as i32, core::CV_8UC1, core::Scalar::all(128.0))?;
    for (idx, &weight) in weights.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let pixel_value = ((weight as i64 + (N - 1) as i64) * 255 / (2 * (N - 1)) as i64) as u8;
        *frame.at_2d_mut::<u8>(row as i32, col as i32)? = pixel_value;
    }
    Ok(frame)
}

pub fn rect2frame(frame: &mut Mat, rectangle: &Rect) -> Result<(), Box<dyn std::error::Error>> {
    imgproc::rectangle(
        frame,
        core::Rect::new(rectangle.x as i32, rectangle.y as i32, rectangle.width as i32, rectangle.height as i32),
        core::Scalar::all(if rectangle.sign { 255.0 } else { 0.0 }),
        -1,
        imgproc::LineTypes::LINE_4.into(),
        0,
    )?;
    Ok(())
}
