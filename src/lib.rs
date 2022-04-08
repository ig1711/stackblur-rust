pub mod stackblur {
    pub fn bad_blur(
        s: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        radius: usize,
    ) -> Vec<u8> {
        let radius = if radius % 2 == 0 { radius + 1 } else { radius };
        let r = radius / 2;
        let stack_total = ((radius + 1) * (radius + 1)) / 4;
        let hor: Vec<u8> = s
            .chunks(width * channels)
            .map(|rowd| {
                let row: Vec<&[u8]> = rowd.chunks(channels).collect();
                row.iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let kerne = vec![0; radius];
                        let kern: Vec<&[u8]> = kerne
                            .iter()
                            .enumerate()
                            .map(|(ki, _)| {
                                if r > i + ki {
                                    row[i]
                                } else if i + ki - r < row.len() {
                                    row[i + ki - r]
                                } else {
                                    row[row.len() - 1]
                                }
                            })
                            .collect();
                        kern.iter()
                            .enumerate()
                            .fold(vec![0usize; channels], |acc, (p, x)| {
                                x.iter()
                                    .enumerate()
                                    .map(|(j, &it)| {
                                        if p < r {
                                            acc[j] + (it as usize * (p + 1))
                                        } else {
                                            acc[j] + (it as usize * (radius - p))
                                        }
                                    })
                                    .collect()
                            })
                    })
                    .flatten()
                    .map(|k| (k / stack_total) as u8)
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect();

        let d: Vec<&[u8]> = hor.chunks(channels).collect();
        let vv: Vec<&u8> = d
            .iter()
            .enumerate()
            .map(|(i, _)| d[(i % height) * width + (i / height)])
            .flatten()
            .collect();

        let ver: Vec<u8> = vv
            .chunks(height * channels)
            .map(|rowd| {
                let row: Vec<&[&u8]> = rowd.chunks(channels).collect();
                row.iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let kerne = vec![0; radius];
                        let kern: Vec<&[&u8]> = kerne
                            .iter()
                            .enumerate()
                            .map(|(ki, _)| {
                                if r > i + ki {
                                    row[i]
                                } else if i + ki - r < row.len() {
                                    row[i + ki - r]
                                } else {
                                    row[row.len() - 1]
                                }
                            })
                            .collect();
                        kern.iter()
                            .enumerate()
                            .fold(vec![0usize; channels], |acc, (p, x)| {
                                x.iter()
                                    .enumerate()
                                    .map(|(j, &&it)| {
                                        if p < r {
                                            acc[j] + (it as usize * (p + 1))
                                        } else {
                                            acc[j] + (it as usize * (radius - p))
                                        }
                                    })
                                    .collect()
                            })
                    })
                    .flatten()
                    .map(|k| (k / stack_total) as u8)
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect();

        let dd: Vec<&[u8]> = ver.chunks(channels).collect();
        dd.iter()
            .enumerate()
            .map(|(i, _)| dd[(i % width) * height + (i / width)].to_owned())
            .flatten()
            .collect()
    }

    use rayon::prelude::*;
    pub fn mt_bad_blur(
        s: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        radius: usize,
    ) -> Vec<u8> {
        let radius = if radius % 2 == 0 { radius + 1 } else { radius };
        let r = radius / 2;
        let stack_total = ((radius + 1) * (radius + 1)) / 4;
        let hor: Vec<u8> = s
            .par_chunks(width * channels)
            .map(|rowd| {
                let row: Vec<&[u8]> = rowd.par_chunks(channels).collect();
                row.par_iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let kerne = vec![0; radius];
                        let kern: Vec<&[u8]> = kerne
                            .par_iter()
                            .enumerate()
                            .map(|(ki, _)| {
                                if r > i + ki {
                                    row[i]
                                } else if i + ki - r < row.len() {
                                    row[i + ki - r]
                                } else {
                                    row[row.len() - 1]
                                }
                            })
                            .collect();
                        kern.into_par_iter()
                            .enumerate()
                            .fold(
                                || vec![0usize; channels],
                                |acc, (p, x)| {
                                    x.par_iter()
                                        .enumerate()
                                        .map(|(j, &it)| {
                                            if p < r {
                                                acc[j] + (it as usize * (p + 1))
                                            } else {
                                                acc[j] + (it as usize * (radius - p))
                                            }
                                        })
                                        .collect()
                                },
                            )
                            .flatten()
                    })
                    .flatten()
                    .map(|k| (k / stack_total) as u8)
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect();

        let d: Vec<&[u8]> = hor.par_chunks(channels).collect();
        let vv: Vec<&u8> = d
            .par_iter()
            .enumerate()
            .map(|(i, _)| d[(i % height) * width + (i / height)])
            .flatten()
            .collect();

        let ver: Vec<u8> = vv
            .par_chunks(height * channels)
            .map(|rowd| {
                let row: Vec<&[&u8]> = rowd.par_chunks(channels).collect();
                row.par_iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let kerne = vec![0; radius];
                        let kern: Vec<&[&u8]> = kerne
                            .par_iter()
                            .enumerate()
                            .map(|(ki, _)| {
                                if r > i + ki {
                                    row[i]
                                } else if i + ki - r < row.len() {
                                    row[i + ki - r]
                                } else {
                                    row[row.len() - 1]
                                }
                            })
                            .collect();
                        kern.into_par_iter()
                            .enumerate()
                            .fold(
                                || vec![0usize; channels],
                                |acc, (p, x)| {
                                    x.par_iter()
                                        .enumerate()
                                        .map(|(j, &&it)| {
                                            if p < r {
                                                acc[j] + (it as usize * (p + 1))
                                            } else {
                                                acc[j] + (it as usize * (radius - p))
                                            }
                                        })
                                        .collect()
                                },
                            )
                            .flatten()
                    })
                    .flatten()
                    .map(|k| (k / stack_total) as u8)
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect();

        let dd: Vec<&[u8]> = ver.par_chunks(channels).collect();
        dd.par_iter()
            .enumerate()
            .map(|(i, _)| dd[(i % width) * height + (i / width)].to_owned())
            .flatten()
            .collect()
    }
    pub fn blur(
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        radius: usize,
    ) -> Vec<u8> {
        let mul_table: [usize; 255] = [
            512, 512, 456, 512, 328, 456, 335, 512, 405, 328, 271, 456, 388, 335, 292, 512, 454,
            405, 364, 328, 298, 271, 496, 456, 420, 388, 360, 335, 312, 292, 273, 512, 482, 454,
            428, 405, 383, 364, 345, 328, 312, 298, 284, 271, 259, 496, 475, 456, 437, 420, 404,
            388, 374, 360, 347, 335, 323, 312, 302, 292, 282, 273, 265, 512, 497, 482, 468, 454,
            441, 428, 417, 405, 394, 383, 373, 364, 354, 345, 337, 328, 320, 312, 305, 298, 291,
            284, 278, 271, 265, 259, 507, 496, 485, 475, 465, 456, 446, 437, 428, 420, 412, 404,
            396, 388, 381, 374, 367, 360, 354, 347, 341, 335, 329, 323, 318, 312, 307, 302, 297,
            292, 287, 282, 278, 273, 269, 265, 261, 512, 505, 497, 489, 482, 475, 468, 461, 454,
            447, 441, 435, 428, 422, 417, 411, 405, 399, 394, 389, 383, 378, 373, 368, 364, 359,
            354, 350, 345, 341, 337, 332, 328, 324, 320, 316, 312, 309, 305, 301, 298, 294, 291,
            287, 284, 281, 278, 274, 271, 268, 265, 262, 259, 257, 507, 501, 496, 491, 485, 480,
            475, 470, 465, 460, 456, 451, 446, 442, 437, 433, 428, 424, 420, 416, 412, 408, 404,
            400, 396, 392, 388, 385, 381, 377, 374, 370, 367, 363, 360, 357, 354, 350, 347, 344,
            341, 338, 335, 332, 329, 326, 323, 320, 318, 315, 312, 310, 307, 304, 302, 299, 297,
            294, 292, 289, 287, 285, 282, 280, 278, 275, 273, 271, 269, 267, 265, 263, 261, 259,
        ];

        let shg_table: [usize; 255] = [
            9, 11, 12, 13, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 17, 17, 17,
            18, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19,
            19, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21,
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
            21, 21, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
            23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        ];

        let radius = if radius % 2 == 0 { radius + 1 } else { radius };
        let r = radius / 2;
        let mul = mul_table[r];
        let shg = shg_table[r];

        let mut out = vec![0u8; width * height * channels];
        for row in 0..height {
            let rwc = row * width * channels;
            for i in 0..width {
                let ic = i * channels;
                for channel in 0..channels {
                    let sum = (0..radius).fold(0usize, |acc, idx| {
                        if r > i + idx {
                            if idx < r {
                                acc + data[rwc + ic + channel] as usize * (idx + 1)
                            } else {
                                acc + data[rwc + ic + channel] as usize * (radius - idx)
                            }
                        } else if i + idx - r < width {
                            if idx < r {
                                acc + data[rwc + (i + idx - r) * channels + channel] as usize
                                    * (idx + 1)
                            } else {
                                acc + data[rwc + (i + idx - r) * channels + channel] as usize
                                    * (radius - idx)
                            }
                        } else {
                            if idx < r {
                                acc + data[rwc + width - channels + channel] as usize * (idx + 1)
                            } else {
                                acc + data[rwc + width - channels + channel] as usize
                                    * (radius - idx)
                            }
                        }
                    });
                    out[rwc + ic + channel] = ((sum * mul) >> shg) as u8;
                }
            }
        }

        let mut out_2 = vec![0u8; width * height * channels];
        for col in 0..width {
            let cc = col * channels;
            for i in 0..height {
                let iwc = i * width * channels;
                for channel in 0..channels {
                    let sum = (0..radius).fold(0usize, |acc, idx| {
                        if r > i + idx {
                            if idx < r {
                                acc + out[cc + iwc + channel] as usize * (idx + 1)
                            } else {
                                acc + out[cc + iwc + channel] as usize * (radius - idx)
                            }
                        } else if i + idx - r < height {
                            if idx < r {
                                acc + out[cc + (i + idx - r) * width * channels + channel] as usize
                                    * (idx + 1)
                            } else {
                                acc + out[cc + (i + idx - r) * width * channels + channel] as usize
                                    * (radius - idx)
                            }
                        } else {
                            if idx < r {
                                acc + out[cc + height - channels + channel] as usize * (idx + 1)
                            } else {
                                acc + out[cc + height - channels + channel] as usize
                                    * (radius - idx)
                            }
                        }
                    });
                    out_2[cc + iwc + channel] = ((sum * mul) >> shg) as u8;
                }
            }
        }
        out_2
    }
}
