pub mod stackblur {
    pub fn blur(s: &[u8], width: usize, height: usize, channels: usize, radius: usize) -> Vec<u8> {
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
                                let p = if i < r { radius - kern.len() + p } else { p };
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
                                let p = if i < r { radius - kern.len() + p } else { p };
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
}
