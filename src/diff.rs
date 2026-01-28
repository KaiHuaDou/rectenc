use crate::rect::Rect;
use rayon::prelude::*;

const ACCURACY_PENALTY: f32 = 1.0;
const AREA_WEIGHT: f32 = 1.0;

pub fn best_rect(weights: &[i8], rows: usize, cols: usize) -> Rect {
    let (weights_corrected, weights_invert) = weights
        .iter()
        .map(|&w| {
            if w != 0 {
                let w_shift = (w as f32 * ACCURACY_PENALTY) as i8;
                if w_shift < 0 { (w_shift - 1, -(w_shift - 1)) } else { (w_shift, -w_shift) }
            } else {
                (-1, -1)
            }
        })
        .collect::<(Vec<i8>, Vec<i8>)>();
    let (pos_rect, pos_sum) = kadane_2d(&weights_corrected, rows, cols, true);
    let (neg_rect, neg_sum) = kadane_2d(&weights_invert, rows, cols, false);

    if pos_sum >= neg_sum { pos_rect } else { neg_rect }
}

#[inline(always)]
pub fn kadane_2d(weights: &[i8], rows: usize, cols: usize, sign: bool) -> (Rect, i32) {
    let init_rect = Rect { x: 0, y: 0, width: 0, height: 0, sign };
    let init_pair = (init_rect, i32::MIN);

    let (best_rect, max_score) = (0..rows)
        .into_par_iter()
        .map(|top| {
            // 如果本行和为 0，自动跳转到下一行
            // 删除会增加碎片但是同时增加细节度
            let row_start = top * cols;
            let mut row_sum = 0i32;
            for j in 0..cols {
                row_sum += weights[row_start + j] as i32;
            }
            if row_sum == 0 {
                return (init_rect, i32::MIN);
            }

            let mut col_sums = vec![0i32; cols];
            let mut local_score = i32::MIN;
            let mut local_rect = init_rect;

            for bottom in top..rows {
                let row_start = bottom * cols;
                for j in 0..cols {
                    col_sums[j] += weights[row_start + j] as i32;
                }

                let (current_sum, left, right) = kadane_1d(&col_sums);

                let area = (right - left + 1) * (bottom - top + 1);
                let score = current_sum + (area as f32 * AREA_WEIGHT) as i32;

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
        .reduce(|| init_pair, |a, b| if a.1 >= b.1 { a } else { b });

    if best_rect.width == 1 || best_rect.height == 1 {
        (Rect { x: 0, y: 0, width: 0, height: 0, sign }, 0)
    } else {
        (best_rect, max_score)
    }
}

#[inline(always)]
fn kadane_1d(arr: &[i32]) -> (i32, usize, usize) {
    let mut max_sum = arr[0];
    let mut current_sum = arr[0];
    let mut start = 0;
    let mut end = 0;
    let mut temp_start = 0;

    for i in 1..arr.len() {
        if current_sum < 0 {
            current_sum = arr[i];
            temp_start = i;
        } else {
            current_sum += arr[i];
        }

        if current_sum > max_sum {
            max_sum = current_sum;
            start = temp_start;
            end = i;
        }
    }

    (max_sum, start, end)
}
