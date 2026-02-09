use crate::diff::*;
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

pub fn frames2rect(frames: &[Mat; 2]) -> Result<(Rect, Mat), Box<dyn std::error::Error>> {
    let rows = frames[0].rows() as usize;
    let cols = frames[0].cols() as usize;
    let mut weights = vec![Weight::BB; rows * cols];
    for row in 0..rows {
        for col in 0..cols {
            let orig = *frames[0].at_2d::<u8>(row as i32, col as i32)? > 0;
            let curr = *frames[1].at_2d::<u8>(row as i32, col as i32)? > 0;
            weights[row * cols + col] = match (orig, curr) {
                (false, false) => Weight::BB,
                (false, true) => Weight::BW,
                (true, false) => Weight::WB,
                (true, true) => Weight::WW,
            }
        }
    }
    let rectangle = best_rect(&weights, rows, cols);
    let weight_mat = weight2mat(&weights, rows, cols)?;
    Ok((rectangle, weight_mat))
}

// This is a performance HACK.
static mut CURR_FRAME: u32 = 0;

fn weight2mat(weights: &[Weight], rows: usize, cols: usize) -> Result<Mat, Box<dyn std::error::Error>> {
    let mut frame = Mat::new_rows_cols_with_default(rows as i32, cols as i32, core::CV_8UC1, core::Scalar::all(128.0))?;
    for (idx, &weight) in weights.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let pixel_value = match weight {
            Weight::BB | Weight::WW => 128,
            Weight::BW => 255,
            Weight::WB => 0,
        };
        *frame.at_2d_mut::<u8>(row as i32, col as i32)? = pixel_value;
    }
    unsafe {
        CURR_FRAME += 1;
    }
    imgproc::put_text(
        &mut frame,
        &format!("{}", unsafe { CURR_FRAME }),
        core::Point_::new(10, 50),
        imgproc::FONT_HERSHEY_SIMPLEX,
        1.0,
        core::Scalar::new(0.0, 0.0, 255.0, 0.0),
        2,
        imgproc::LINE_AA,
        false,
    )?;
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
