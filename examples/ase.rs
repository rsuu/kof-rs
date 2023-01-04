use asefile::AsepriteFile;




use minifb::{Window};

use std::mem;
use std::num::NonZeroU32;
use std::path::Path;

fn main() {
    let (width, height) = (625_usize, 625_usize);

    let windowoptions = minifb::WindowOptions {
        borderless: false,
        transparency: false,
        title: true,
        resize: false,
        topmost: false,
        none: true,
        scale_mode: minifb::ScaleMode::Center,
        scale: minifb::Scale::X1,
    };

    let mut window = Window::new("rmg", width, height, windowoptions).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_millis(60)));
    window.update();

    let _buffer = vec![789; width * height];

    let frames = open_ase("walk");
    let mut idx = 0;

    'l1: while window.is_open() {
        window
            .update_with_buffer(&frames[idx], width, height)
            .unwrap();

        if idx + 1 < frames.len() {
            idx += 1;
        } else {
            idx = 0;
        }
    }
    // ==========================================
}

fn open_ase(id: &str) -> Vec<Vec<u32>> {
    let ase = AsepriteFile::read_file(Path::new(&format!("./tests/{}.ase", id))).unwrap();

    let head = 0;
    let tail = ase.num_frames();

    let mut frames = Vec::with_capacity(tail as usize);

    for idx in head..tail {
        frames.push(mem::take(&mut ase.frame(idx).image().to_vec()));
        *frames.last_mut().unwrap().last_mut().unwrap() = 0;
    }

    let mut tmp: Vec<u32> = vec![];
    let mut res: Vec<Vec<u32>> = vec![];

    for idx in 0..frames.len() {
        let mut resize = resize_rgba8(frames[idx].clone());

        // let mut resize = vec![];
        // for idx in frames[idx].iter().step_by(4) {
        //     let mut fg = Rgba::new([idx + 0, idx + 1, idx + 2, idx + 3]);
        //     let bg = Rgba::new([0, 0, 0, 0]);
        //     fg.blend(&bg);
        //
        //     resize.extend_from_slice(&fg.0);
        // }

        argb_u32(&mut tmp, &mem::take(&mut resize));
        res.push(tmp.clone());
    }

    res
}

#[inline(always)]
pub fn argb_u32(buffer: &mut Vec<u32>, bytes: &[u8]) {
    *buffer = vec![0; bytes.len() / 4];

    for (idx, f) in (0..bytes.len()).step_by(4).enumerate() {
        buffer[idx] = rgba_as_argb_u32(&bytes[f], &bytes[f + 1], &bytes[f + 2], &bytes[f + 3]);
    }
}

#[inline(always)]
pub fn rgba_as_argb_u32(r: &u8, g: &u8, b: &u8, _a: &u8) -> u32 {
    // (r, g, b, a) -> (a, r, g, b) -> u32
    //  3  2  1  0      3  2  1  0
    u32::from_be_bytes([0, *r, *g, *b])
}

#[inline(always)]
pub fn resize_rgba8(bytes: Vec<u8>) -> Vec<u8> {
    let mut src_image = fir::Image::from_vec_u8(
        NonZeroU32::new(625).ok_or(()).unwrap(),
        NonZeroU32::new(625).ok_or(()).unwrap(),
        bytes,
        fir::PixelType::U8x4,
    )
    .unwrap();
    let dst_width = NonZeroU32::new(625).ok_or(()).unwrap();
    let dst_height = NonZeroU32::new(625).ok_or(()).unwrap();

    // FIXED: https://github.com/Cykooz/fast_image_resize/issues/9
    let mut dst_image = fir::Image::new(dst_width, dst_height, src_image.pixel_type());
    let mut dst_view = dst_image.view_mut();
    let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Box));

    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    // rgba
    let alpha_mul_div = fir::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    //

    dst_image.buffer().to_vec()
}

// #[inline(always)]
// fn blend(fg: &[u8; 4], bg: &[u8; 4]) -> [u8; 4] {
// // blend alpha
// // https://doc.servo.org/src/image/color.rs.html#689
//     let alpha = fg[3] as i32 + 1;
//     let inv = 256 - fg[3] as i32;
//
//     [
//         (alpha as i32 * fg[0] as i32 + inv * bg[0] as i32) as u8,
//         (alpha as i32 * fg[1] as i32 + inv * bg[1] as i32) as u8,
//         (alpha as i32 * fg[2] as i32 + inv * bg[2] as i32) as u8,
//         255,
//     ]
// }

// #[derive(Clone, Copy)]
// pub struct Rgba([u8; 4]);
//
// impl Rgba {
//     fn new(rgba: [u8; 4]) -> Self {
//         Self(rgba)
//     }
//
//     #[inline(always)]
//     fn blend(&mut self, other: &Rgba) {
//         // http://stackoverflow.com/questions/7438263/alpha-compositing-algorithm-blend-modes#answer-11163848
//
//         if other.0[3] == 0 {
//             return;
//         }
//
//         if other.0[3] == u8::MAX {
//             *self = *other;
//
//             return;
//         }
//
//         // First, as we don't know what type our pixel is, we have to convert to floats between 0.0 and 1.0
//         let max_t = u8::MAX as f32;
//
//         let (bg_r, bg_g, bg_b, bg_a) = (self.0[0], self.0[1], self.0[2], self.0[3]);
//         let (fg_r, fg_g, fg_b, fg_a) = (other.0[0], other.0[1], other.0[2], other.0[3]);
//         let (bg_r, bg_g, bg_b, bg_a) = (
//             bg_r as f32 / max_t,
//             bg_g as f32 / max_t,
//             bg_b as f32 / max_t,
//             bg_a as f32 / max_t,
//         );
//         let (fg_r, fg_g, fg_b, fg_a) = (
//             fg_r as f32 / max_t,
//             fg_g as f32 / max_t,
//             fg_b as f32 / max_t,
//             fg_a as f32 / max_t,
//         );
//
//         // Work out what the final alpha level will be
//         let alpha_final = bg_a + fg_a - bg_a * fg_a;
//
//         if alpha_final == 0.0 {
//             return;
//         };
//
//         // We premultiply our channels by their alpha, as this makes it easier to calculate
//         let (bg_r_a, bg_g_a, bg_b_a) = (bg_r * bg_a, bg_g * bg_a, bg_b * bg_a);
//         let (fg_r_a, fg_g_a, fg_b_a) = (fg_r * fg_a, fg_g * fg_a, fg_b * fg_a);
//
//         // Standard formula for src-over alpha compositing
//         let (out_r_a, out_g_a, out_b_a) = (
//             fg_r_a + bg_r_a * (1.0 - fg_a),
//             fg_g_a + bg_g_a * (1.0 - fg_a),
//             fg_b_a + bg_b_a * (1.0 - fg_a),
//         );
//
//         // Unmultiply the channels by our resultant alpha channel
//         let (out_r, out_g, out_b) = (
//             out_r_a / alpha_final,
//             out_g_a / alpha_final,
//             out_b_a / alpha_final,
//         );
//
//         // Cast back to our initial type on return
//         *self = Rgba([
//             if (max_t * out_r) > 255.0 {
//                 255_u8
//             } else if (max_t * out_r) < 0.0 {
//                 0_u8
//             } else {
//                 (max_t * out_r) as u8
//             },
//             if (max_t * out_g) > 255.0 {
//                 255_u8
//             } else if (max_t * out_g) < 0.0 {
//                 0_u8
//             } else {
//                 (max_t * out_g) as u8
//             },
//             if (max_t * out_b) > 255.0 {
//                 255_u8
//             } else if (max_t * out_b) < 0.0 {
//                 0_u8
//             } else {
//                 (max_t * out_b) as u8
//             },
//             if (max_t * alpha_final) > 255.0 {
//                 255_u8
//             } else if (max_t * alpha_final) < 0.0 {
//                 0_u8
//             } else {
//                 (max_t * alpha_final) as u8
//             },
//         ]);
//
//     }
// }
