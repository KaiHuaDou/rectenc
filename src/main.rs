#![allow(clippy::all)]

mod diff;
mod error;
mod rect;
mod video;

use clap::{Parser, Subcommand};
use error::*;
use opencv::core;
use opencv::prelude::*;
use rect::*;
use video::*;

#[derive(Parser)]
#[command(name = "rectenc", author, version, about = "A video encoding/decoding utility using rectangles", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        #[arg(value_parser)]
        input: String,
        #[arg(value_parser)]
        output: String,
        #[arg(short, long, default_value_t = 1)]
        rect_count: u8,
        #[arg(short, long, default_value_t = false)]
        preview: bool,
        #[arg(short, long, default_value_t = false)]
        adaptive: bool,
    },
    Decode {
        #[arg(value_parser)]
        input: String,
        #[arg(value_parser)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode { input, output, rect_count, preview, adaptive } => {
            if let Err(e) = encode_video(input, output, *rect_count, *preview, *adaptive) {
                eprintln!("Error encoding video: {}", e);
            };
        }
        Commands::Decode { input, output } => {
            if let Err(e) = decode_video(input, output) {
                eprintln!("Error decoding video: {}", e);
            };
        }
    }
}

fn encode_video(input: &str, output: &str, rect_count: u8, preview: bool, adaptive: bool) -> Result<(), RectEncError> {
    println!("编码 {} 到 {}", input, output);
    let mut rectangles = Vec::new();
    let (metadata, iterator) = read_video(input, preview)?;
    let adaption = if adaptive { Some(10) } else { None };

    rectangles.push(Rect {
        x: metadata.fps.to_bits(),
        y: if adaptive { u32::MAX } else { rect_count as u32 },
        width: metadata.width,
        height: metadata.height,
        sign: true,
    });

    let mut ref_frame = Mat::new_rows_cols_with_default(
        metadata.height as i32,
        metadata.width as i32,
        core::CV_8UC1,
        core::Scalar::all(0.0),
    )?;
    let mut weight_mats = Vec::new();

    for mut frames in iterator {
        for _ in 0..rect_count {
            print!("*");
            frames[0] = ref_frame.clone();
            let (rect, weight_mat) = frames2rect(&frames, adaption)?;
            rectangles.push(rect);
            if rect == Rect::INVALID {
                break;
            }
            rect2frame(&mut ref_frame, &rect)?;
            weight_mats.push(weight_mat);
        }
    }

    write_rects(&rectangles, output)?;
    let weight_fps = metadata.fps * rect_count as f32;
    write_video(&weight_mats, weight_fps, &format!("{input}-{rect_count}-weights.mkv"), false)?;

    Ok(())
}

fn decode_video(input: &str, output: &str) -> Result<(), RectEncError> {
    println!("解码 {} 到 {}", input, output);
    let rectangles = read_rects(input)?;
    let metadata = &rectangles[0];
    let empty_mat = Mat::new_rows_cols_with_default(
        metadata.height as i32,
        metadata.width as i32,
        core::CV_8UC1,
        core::Scalar::all(0.0),
    )?;

    let adaptive = metadata.y == u32::MAX;
    if adaptive {
        let mut frames = vec![empty_mat.clone()];
        let mut idx = 0;
        for i in 1..rectangles.len() {
            if rectangles[i] != Rect::INVALID {
                rect2frame(&mut frames[idx], &rectangles[i])?;
            } else {
                frames.push(frames[idx].clone());
                idx += 1;
            }
        }
        write_video(&frames, f32::from_bits(metadata.x), output, true)?;
    } else {
        let rect_count = metadata.y as usize;
        let frame_count = (rectangles.len() - 1) / rect_count;
        let mut frames = vec![empty_mat.clone(); frame_count];
        for i in 0..frame_count {
            frames[i] = if i == 0 { empty_mat.clone() } else { frames[i - 1].clone() };
            for j in 0..rect_count {
                let idx = 1 + i * rect_count + j;
                rect2frame(&mut frames[i], &rectangles[idx])?;
            }
        }
        write_video(&frames, f32::from_bits(metadata.x), output, true)?;
    };
    Ok(())
}
