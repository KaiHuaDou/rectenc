use opencv::core::*;
use opencv::imgproc::*;
use opencv::prelude::*;
use opencv::videoio::*;
use std::array;
use std::collections::VecDeque;

pub const PREVIEW_LEN: usize = 2;

pub struct VideoMetadata {
    pub fps: f32,
    pub width: u32,
    pub height: u32,
}

pub struct VideoIterator<const N: usize> {
    video_capture: VideoCapture,
    current_index: usize,
    total_frames: usize,
    buffer: VecDeque<Mat>,
}

impl<const N: usize> VideoIterator<N> {
    pub fn new(video_capture: VideoCapture, preview: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let total_frames = video_capture.get(CAP_PROP_FRAME_COUNT)? as usize;
        println!("视频总帧数：{}", total_frames);
        if total_frames == 0 {
            return Err("视频帧数为 0".into());
        }
        let video_iterator = VideoIterator {
            video_capture,
            current_index: 0,
            total_frames: if preview { 512 } else { total_frames },
            buffer: VecDeque::with_capacity(total_frames),
        };
        Ok(video_iterator)
    }

    // fn skip_frames(&mut self, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    //     for _ in 0..count {
    //         let mut mat = Mat::default();
    //         if !self.video_capture.read(&mut mat)? {
    //             break;
    //         };
    //     }
    //     Ok(())
    // }

    #[inline(always)]
    fn read_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut mat = Mat::default();
        if !self.video_capture.read(&mut mat)? {
            return Ok(());
        };
        let mut gray = Mat::default();
        cvt_color(&mat, &mut gray, COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT)?;
        let mut binary = Mat::default();
        threshold(&gray, &mut binary, 127.0, 255.0, THRESH_BINARY)?;
        self.buffer.push_back(binary);
        Ok(())
    }
}

impl<const N: usize> Iterator for VideoIterator<N> {
    type Item = [Mat; N];

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            for _ in 0..N {
                self.read_frame().ok()?;
            }
            self.current_index += N;
        } else if self.current_index + N <= self.total_frames {
            self.buffer.pop_front();
            self.read_frame().ok()?;
            self.current_index += 1;
        } else {
            return None;
        }
        let mut result = array::from_fn(|_| Mat::default());
        for i in 0..N {
            result[i] = self.buffer[i].clone();
        }
        Some(result)
    }
}

pub fn read_video(
    path: &str,
    preview: bool,
) -> Result<(VideoMetadata, VideoIterator<PREVIEW_LEN>), Box<dyn std::error::Error>> {
    println!("正在从 {} 读取视频帧", path);
    let video_capture = VideoCapture::from_file(path, CAP_ANY)?;
    let metadata = VideoMetadata {
        fps: video_capture.get(CAP_PROP_FPS)? as f32,
        width: video_capture.get(CAP_PROP_FRAME_WIDTH)? as u32,
        height: video_capture.get(CAP_PROP_FRAME_HEIGHT)? as u32,
    };
    let video_iterator = VideoIterator::<PREVIEW_LEN>::new(video_capture, preview)?;
    Ok((metadata, video_iterator))
}

pub fn write_video(frames: &[Mat], fps: f32, path: &str, mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let fourcc = if mode { VideoWriter::fourcc('a', 'v', 'c', '1')? } else { VideoWriter::fourcc('F', 'F', 'V', '1')? };

    let mut video_writer = VideoWriter::new_with_backend(
        path,
        CAP_FFMPEG,
        fourcc,
        fps as f64,
        Size::new(frames[0].cols(), frames[0].rows()),
        false,
    )?;

    for frame in frames {
        video_writer.write(frame)?;
    }

    video_writer.release()?;
    Ok(())
}
