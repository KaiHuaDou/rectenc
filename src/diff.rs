#![allow(dead_code)]

use crate::rect::Rect;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Weight {
    BB,
    WW,
    BW,
    WB,
}

struct Params {
    /// 放大差异权重/控制矩形内允许包含的与主差异方向相反的像素比例
    ///
    /// **典型值范围：** 1 ~ 5
    pub scale: i8,

    /// 对无差异区域（权重为 0）的惩罚强度
    ///
    /// **典型值范围：** 1 ~ 5
    pub zero_weight: i8,

    /// 控制对正方形矩形的偏好程度，惩罚极端的宽或高。
    ///
    /// **典型值范围：** 0.0 ~ 5.0
    pub aspect_weight: f32,

    /// 矩形的最小宽度和高度，过滤过小的差异区域。
    ///
    /// **典型值范围：** 2 ~ 20（设置为 0 表示无最小尺寸限制）
    pub min_size: u32,
}

const BAS_0: Params = Params { scale: 2, zero_weight: -1, aspect_weight: 0.0, min_size: 0 };
const PARAMS: Params = BAS_0;

pub fn best_rect(weights: &[Weight], rows: usize, cols: usize) -> Rect {
    let weights_corrected = weights;
    let (weights_pos, weights_neg) = weights_corrected
        .iter()
        .map(|&w| match w {
            Weight::BB => (PARAMS.zero_weight, 0),
            Weight::WW => (0, PARAMS.zero_weight),
            Weight::BW => (PARAMS.scale, -PARAMS.scale),
            Weight::WB => (-PARAMS.scale, PARAMS.scale),
        })
        .collect::<(Vec<i8>, Vec<i8>)>();
    let (pos_rect, pos_sum) = kadane_2d(&weights_pos, rows, cols, true);
    let (neg_rect, neg_sum) = kadane_2d(&weights_neg, rows, cols, false);

    if pos_sum >= neg_sum { pos_rect } else { neg_rect }
}

#[inline(always)]
pub fn kadane_2d(weights: &[i8], rows: usize, cols: usize, sign: bool) -> (Rect, i32) {
    let init_rect = Rect { x: 0, y: 0, width: 0, height: 0, sign };

    let candidates: Vec<(Rect, i32)> = (0..rows)
        .into_par_iter()
        .map(|top| {
            let mut col_sums = vec![0i32; cols];
            let mut local_score = i32::MIN;
            let mut local_rect = init_rect;

            for bottom in top..rows {
                let row_start = bottom * cols;
                for j in 0..cols {
                    col_sums[j] += weights[row_start + j] as i32;
                }

                let (current_sum, left, right) = kadane_1d(&col_sums);

                let width = (right - left + 1) as f32;
                let height = (bottom - top + 1) as f32;

                let aspect_penalty = if width > 0.0 && height > 0.0 {
                    let aspect_ratio = width.max(height) / width.min(height);
                    (aspect_ratio - 1.0) * PARAMS.aspect_weight
                } else {
                    0.0
                };

                let score = current_sum - aspect_penalty as i32;

                if score > local_score {
                    local_score = score;
                    local_rect = Rect {
                        x: left as u32,
                        y: top as u32,
                        width: (right - left + 1) as u32,
                        height: (bottom - top + 1) as u32,
                        sign,
                    };
                }
            }

            (local_rect, local_score)
        })
        .collect();

    candidates
        .into_iter()
        .filter(|(rect, _)| rect.width >= PARAMS.min_size && rect.height >= PARAMS.min_size)
        .max_by_key(|(_, score)| *score)
        .unwrap_or((Rect { x: 0, y: 0, width: 0, height: 0, sign }, 0))
}

#[inline(always)]
fn kadane_1d(arr: &[i32]) -> (i32, usize, usize) {
    let mut max_sum = arr[0];
    let mut current_sum = arr[0];
    let mut start = 0;
    let mut end = 0;
    let mut temp_start = 0;

    for (i, &val) in arr.iter().enumerate().skip(1) {
        if current_sum < 0 {
            current_sum = val;
            temp_start = i;
        } else {
            current_sum += val;
        }

        if current_sum > max_sum {
            max_sum = current_sum;
            start = temp_start;
            end = i;
        }
    }

    (max_sum, start, end)
}
