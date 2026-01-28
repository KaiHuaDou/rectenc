#![allow(clippy::all)]

mod diff;
mod rect;
mod video;

use clap::{Parser, Subcommand};
use opencv::core::{CV_8UC1, Scalar};
use opencv::prelude::*;
use rayon::ThreadPoolBuilder;
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

const RECT_COUNT: u8 = 1;

fn main() {
    let cli = Cli::parse();
    ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    match &cli.command {
        Commands::Encode { input, output, preview } => {
            if let Err(e) = encode_video(input, output, *preview) {
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

fn encode_video(input: &str, output: &str, preview: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("编码 {} 到 {}", input, output);
    let mut rectangles = Vec::new();
    let (metadata, iterator) = read_video(input, preview)?;
    rectangles.push(Rect {
        x: metadata.fps.to_bits(),
        y: RECT_COUNT as u32,
        width: metadata.width,
        height: metadata.height,
        sign: true,
    });

    let mut ref_frame =
        Mat::new_rows_cols_with_default(metadata.height as i32, metadata.width as i32, CV_8UC1, Scalar::all(0.0))?;
    let mut weight_mats = Vec::new();

    for mut frames in iterator {
        for _ in 0..RECT_COUNT {
            frames[0] = ref_frame.clone();
            let (rect, weight_mat) = frames2rect(&frames)?;
            print!("*");
            rect2frame(&mut ref_frame, &rect)?;
            rectangles.push(rect);
            weight_mats.push(weight_mat);
        }
    }

    write_rects(&rectangles, output)?;
    write_video(&weight_mats, metadata.fps * RECT_COUNT as f32, "weights.mkv", false)?;

    Ok(())
}

fn decode_video(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("解码 {} 到 {}", input, output);
    let rectangles = read_rects(input)?;
    let metadata = &rectangles[0];
    let empty_mat =
        Mat::new_rows_cols_with_default(metadata.height as i32, metadata.width as i32, CV_8UC1, Scalar::all(0.0))?;
    let mut frames = vec![empty_mat; rectangles.len() - 1];
    let rect_count = metadata.y as usize;
    rect2frame(&mut frames[0], &rectangles[1])?;
    for i in 2..(rectangles.len() - 1) / rect_count {
        frames[i - 1] = frames[i - 2].clone();
        for j in 0..rect_count {
            rect2frame(&mut frames[i - 1], &rectangles[rect_count * (i - 1) + j])?;
        }
    }
    write_video(&frames, f32::from_bits(metadata.x), output, true)?;
    Ok(())
}
