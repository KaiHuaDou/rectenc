#![allow(clippy::all)]

mod diff;
mod rect;
mod video;

use clap::{Parser, Subcommand};
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
        Commands::Encode { input, output, rect_count, preview } => {
            if let Err(e) = encode_video(input, output, *rect_count, *preview) {
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

fn encode_video(input: &str, output: &str, rect_count: u8, preview: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("编码 {} 到 {}", input, output);
    let mut rectangles = Vec::new();
    let (metadata, iterator) = read_video(input, preview)?;
    rectangles.push(Rect {
        x: metadata.fps.to_bits(),
        y: rect_count as u32,
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
            frames[0] = ref_frame.clone();
            let (rect, weight_mat) = frames2rect(&frames)?;
            print!("*");
            rect2frame(&mut ref_frame, &rect)?;
            rectangles.push(rect);
            weight_mats.push(weight_mat);
        }
    }

    write_rects(&rectangles, output)?;
    write_video(&weight_mats, metadata.fps * rect_count as f32, &format!("{input}-weights.mkv"), false)?;

    Ok(())
}

fn decode_video(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("解码 {} 到 {}", input, output);
    let rectangles = read_rects(input)?;
    let metadata = &rectangles[0];
    let empty_mat = Mat::new_rows_cols_with_default(
        metadata.height as i32,
        metadata.width as i32,
        core::CV_8UC1,
        core::Scalar::all(0.0),
    )?;
    let rect_count = metadata.y as usize;
    let frame_count = (rectangles.len() - 1) / rect_count;
    let mut frames = vec![empty_mat.clone(); frame_count];

    for i in 0..frame_count {
        frames[i] = if i == 0 { empty_mat.clone() } else { frames[i - 1].clone() };
        for j in 0..rect_count {
            let rect_index = 1 + i * rect_count + j;
            rect2frame(&mut frames[i], &rectangles[rect_index])?;
        }
    }

    write_video(&frames, f32::from_bits(metadata.x), output, true)?;
    Ok(())
}
