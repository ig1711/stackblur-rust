pub mod stackblur {
    pub fn blur(
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        r: u8,
    ) -> Vec<u8> {
        #[rustfmt::skip]
        let mul_table = [
            512,512,456,512,328,456,335,512,405,328,271,456,388,335,292,512,
            454,405,364,328,298,271,496,456,420,388,360,335,312,292,273,512,
            482,454,428,405,383,364,345,328,312,298,284,271,259,496,475,456,
            437,420,404,388,374,360,347,335,323,312,302,292,282,273,265,512,
            497,482,468,454,441,428,417,405,394,383,373,364,354,345,337,328,
            320,312,305,298,291,284,278,271,265,259,507,496,485,475,465,456,
            446,437,428,420,412,404,396,388,381,374,367,360,354,347,341,335,
            329,323,318,312,307,302,297,292,287,282,278,273,269,265,261,512,
            505,497,489,482,475,468,461,454,447,441,435,428,422,417,411,405,
            399,394,389,383,378,373,368,364,359,354,350,345,341,337,332,328,
            324,320,316,312,309,305,301,298,294,291,287,284,281,278,274,271,
            268,265,262,259,257,507,501,496,491,485,480,475,470,465,460,456,
            451,446,442,437,433,428,424,420,416,412,408,404,400,396,392,388,
            385,381,377,374,370,367,363,360,357,354,350,347,344,341,338,335,
            332,329,326,323,320,318,315,312,310,307,304,302,299,297,294,292,
            289,287,285,282,280,278,275,273,271,269,267,265,263,261,259,257];

        #[rustfmt::skip]
        let shg_table = [
             9, 11, 12, 13, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 17, 
            17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 18, 19, 
            19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20,
            20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21,
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22, 
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
            22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
            23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 
            23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
            24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24 ];

        let r = r as usize;
        let radius = (r * 2) + 1;
        let mul = mul_table[r];
        let shg = shg_table[r];

        let mut hor_out = vec![0u8; width * height * channels];

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
                                acc + data[rwc + (width - 1) * channels + channel] as usize
                                    * (idx + 1)
                            } else {
                                acc + data[rwc + (width - 1) * channels + channel] as usize
                                    * (radius - idx)
                            }
                        }
                    });

                    hor_out[rwc + ic + channel] = ((sum * mul) >> shg) as u8;
                }
            }
        }

        let mut ver_out = vec![0u8; width * height * channels];

        for col in 0..width {
            let cc = col * channels;

            for i in 0..height {
                let iwc = i * width * channels;

                for channel in 0..channels {
                    let sum = (0..radius).fold(0usize, |acc, idx| {
                        if r > i + idx {
                            if idx < r {
                                acc + hor_out[cc + iwc + channel] as usize * (idx + 1)
                            } else {
                                acc + hor_out[cc + iwc + channel] as usize * (radius - idx)
                            }
                        } else if i + idx - r < height {
                            if idx < r {
                                acc + hor_out[cc + (i + idx - r) * width * channels + channel]
                                    as usize
                                    * (idx + 1)
                            } else {
                                acc + hor_out[cc + (i + idx - r) * width * channels + channel]
                                    as usize
                                    * (radius - idx)
                            }
                        } else {
                            if idx < r {
                                acc + hor_out[cc + (height - 1) * width * channels + channel]
                                    as usize
                                    * (idx + 1)
                            } else {
                                acc + hor_out[cc + (height - 1) * width * channels + channel]
                                    as usize
                                    * (radius - idx)
                            }
                        }
                    });

                    ver_out[cc + iwc + channel] = ((sum * mul) >> shg) as u8;
                }
            }
        }

        ver_out
    }
}
