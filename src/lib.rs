pub mod stackblur {
    // Somewhat optimized blur
    pub fn blur(
        data: &[u8],
        width: usize,
        height: usize,
        channels: Channels,
        radius: usize,
    ) -> Vec<u8> {
        match channels {
            Channels::RGBA => {
                let a = RGBAHProcessor::new(data, width, height, radius)
                    .flatten()
                    .collect::<Vec<u8>>();
                let mut out: Vec<u8> = vec![Default::default(); width * height * 4];
                RGBAVProcessor::new(&a[..], width, height, radius)
                    .enumerate()
                    .for_each(|(i, item)| {
                        out[((i % height) * width + (i / height)) * 4 + 0] = item[0];
                        out[((i % height) * width + (i / height)) * 4 + 1] = item[1];
                        out[((i % height) * width + (i / height)) * 4 + 2] = item[2];
                        out[((i % height) * width + (i / height)) * 4 + 3] = item[3];
                    });
                out
            }
            Channels::RGB => {
                let a = RGBHProcessor::new(data, width, height, radius)
                    .flatten()
                    .collect::<Vec<u8>>();
                let mut out: Vec<u8> = vec![Default::default(); width * height * 3];
                RGBVProcessor::new(&a[..], width, height, radius)
                    .enumerate()
                    .for_each(|(i, item)| {
                        out[((i % height) * width + (i / height)) * 3 + 0] = item[0];
                        out[((i % height) * width + (i / height)) * 3 + 1] = item[1];
                        out[((i % height) * width + (i / height)) * 3 + 2] = item[2];
                    });
                out
            }
        }
    }

    pub enum Channels {
        RGBA,
        RGB,
    }

    struct RGBAHProcessor<'a> {
        data: &'a [u8],
        width: usize,
        height: usize,
        radius: usize,
        current_index: usize,
        current_row: usize,
        r_store: Vec<u8>,
        g_store: Vec<u8>,
        b_store: Vec<u8>,
        a_store: Vec<u8>,
        r_sum: usize,
        g_sum: usize,
        b_sum: usize,
        a_sum: usize,
        w4: usize,
        rr12: usize,
        r_minus_1: usize,
        w_minus_1: usize,
        mul: usize,
        shg: usize,
    }

    impl<'a> RGBAHProcessor<'a> {
        fn new(data: &'a [u8], width: usize, height: usize, radius: usize) -> Self {
            Self {
                data,
                width,
                height,
                radius,
                current_index: 0,
                current_row: 0,
                r_store: vec![data[0]; radius],
                g_store: vec![data[1]; radius],
                b_store: vec![data[2]; radius],
                a_store: vec![data[3]; radius],
                r_sum: (data[0] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[4 * x + 0] as usize * (radius - 1 - x)
                    }),
                g_sum: (data[1] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[4 * x + 1] as usize * (radius - 1 - x)
                    }),
                b_sum: (data[2] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[4 * x + 2] as usize * (radius - 1 - x)
                    }),
                a_sum: (data[3] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[4 * x + 3] as usize * (radius - 1 - x)
                    }),
                w4: width * 4,
                rr12: radius * (radius + 1) / 2,
                r_minus_1: radius - 1,
                w_minus_1: width - 1,
                mul: MUL_TABLE[radius - 1],
                shg: SHG_TABLE[radius - 1],
            }
        }
    }

    impl Iterator for RGBAHProcessor<'_> {
        type Item = [u8; 4];
        fn next(&mut self) -> Option<Self::Item> {
            if self.current_index == self.width {
                self.current_row += 1;
                if self.current_row == self.height {
                    return None;
                }
                self.current_index = 0;
                let rw4 = self.current_row * self.w4;

                self.r_store = vec![self.data[rw4 + 0]; self.radius];
                self.g_store = vec![self.data[rw4 + 1]; self.radius];
                self.b_store = vec![self.data[rw4 + 2]; self.radius];
                self.a_store = vec![self.data[rw4 + 3]; self.radius];

                self.r_sum = (self.data[rw4 + 0] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw4 + 4 * x + 0] as usize * (self.r_minus_1 - x)
                    });
                self.g_sum = (self.data[rw4 + 1] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw4 + 4 * x + 1] as usize * (self.r_minus_1 - x)
                    });
                self.b_sum = (self.data[rw4 + 2] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw4 + 4 * x + 2] as usize * (self.r_minus_1 - x)
                    });
                self.a_sum = (self.data[rw4 + 3] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw4 + 4 * x + 3] as usize * (self.r_minus_1 - x)
                    });
            }

            let rw4 = self.current_row * self.w4;
            let store_len = self.r_store.len();

            self.r_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw4
                        + 4 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 0] as usize
                })
                .sum::<usize>();
            self.r_sum -= (&self.r_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.g_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw4
                        + 4 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 1] as usize
                })
                .sum::<usize>();
            self.g_sum -= (&self.g_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.b_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw4
                        + 4 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 2] as usize
                })
                .sum::<usize>();
            self.b_sum -= (&self.b_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.a_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw4
                        + 4 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 3] as usize
                })
                .sum::<usize>();
            self.a_sum -= (&self.a_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            let rw4_ci4 = rw4 + 4 * self.current_index;

            let r = self.data[rw4_ci4 + 0];
            let g = self.data[rw4_ci4 + 1];
            let b = self.data[rw4_ci4 + 2];
            let a = self.data[rw4_ci4 + 3];

            self.r_store.push(r);
            self.g_store.push(g);
            self.b_store.push(b);
            self.a_store.push(a);

            self.current_index += 1;

            Some([
                ((self.r_sum * self.mul) >> self.shg) as u8,
                ((self.g_sum * self.mul) >> self.shg) as u8,
                ((self.b_sum * self.mul) >> self.shg) as u8,
                ((self.a_sum * self.mul) >> self.shg) as u8,
            ])
        }
    }

    struct RGBAVProcessor<'a> {
        data: &'a [u8],
        width: usize,
        height: usize,
        radius: usize,
        current_index: usize,
        current_col: usize,
        r_store: Vec<u8>,
        g_store: Vec<u8>,
        b_store: Vec<u8>,
        a_store: Vec<u8>,
        r_sum: usize,
        g_sum: usize,
        b_sum: usize,
        a_sum: usize,
        w4: usize,
        rr12: usize,
        r_minus_1: usize,
        h_minus_1: usize,
        mul: usize,
        shg: usize,
    }

    impl<'a> RGBAVProcessor<'a> {
        fn new(data: &'a [u8], width: usize, height: usize, radius: usize) -> Self {
            Self {
                data,
                width,
                height,
                radius,
                current_index: 0,
                current_col: 0,
                r_store: vec![data[0]; radius],
                g_store: vec![data[1]; radius],
                b_store: vec![data[2]; radius],
                a_store: vec![data[3]; radius],
                r_sum: (data[0] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 4 * x + 0] as usize * (radius - 1 - x)
                    }),
                g_sum: (data[1] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 4 * x + 1] as usize * (radius - 1 - x)
                    }),
                b_sum: (data[2] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 4 * x + 2] as usize * (radius - 1 - x)
                    }),
                a_sum: (data[3] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 4 * x + 3] as usize * (radius - 1 - x)
                    }),
                w4: width * 4,
                rr12: radius * (radius + 1) / 2,
                r_minus_1: radius - 1,
                h_minus_1: height - 1,
                mul: MUL_TABLE[radius - 1],
                shg: SHG_TABLE[radius - 1],
            }
        }
    }

    impl Iterator for RGBAVProcessor<'_> {
        type Item = [u8; 4];
        fn next(&mut self) -> Option<Self::Item> {
            if self.current_index == self.height {
                self.current_col += 1;
                if self.current_col == self.width {
                    return None;
                }
                self.current_index = 0;
                let c4 = self.current_col * 4;

                self.r_store = vec![self.data[c4 + 0]; self.radius];
                self.g_store = vec![self.data[c4 + 1]; self.radius];
                self.b_store = vec![self.data[c4 + 2]; self.radius];
                self.a_store = vec![self.data[c4 + 3]; self.radius];

                self.r_sum = (self.data[c4 + 0] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c4 + self.w4 * x + 0] as usize * (self.r_minus_1 - x)
                    });
                self.g_sum = (self.data[c4 + 1] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c4 + self.w4 * x + 1] as usize * (self.r_minus_1 - x)
                    });
                self.b_sum = (self.data[c4 + 2] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c4 + self.w4 * x + 2] as usize * (self.r_minus_1 - x)
                    });
                self.a_sum = (self.data[c4 + 3] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c4 + self.w4 * x + 3] as usize * (self.r_minus_1 - x)
                    });
            }

            let c4 = self.current_col * 4;
            let store_len = self.r_store.len();

            self.r_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c4
                        + self.w4
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 0] as usize
                })
                .sum::<usize>();
            self.r_sum -= (&self.r_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.g_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c4
                        + self.w4
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 1] as usize
                })
                .sum::<usize>();
            self.g_sum -= (&self.g_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.b_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c4
                        + self.w4
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 2] as usize
                })
                .sum::<usize>();
            self.b_sum -= (&self.b_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.a_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c4
                        + self.w4
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 3] as usize
                })
                .sum::<usize>();
            self.a_sum -= (&self.a_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            let c4w4ci = c4 + self.w4 * self.current_index;

            let r = self.data[c4w4ci + 0];
            let g = self.data[c4w4ci + 1];
            let b = self.data[c4w4ci + 2];
            let a = self.data[c4w4ci + 3];

            self.r_store.push(r);
            self.g_store.push(g);
            self.b_store.push(b);
            self.a_store.push(a);

            self.current_index += 1;

            Some([
                ((self.r_sum * self.mul) >> self.shg) as u8,
                ((self.g_sum * self.mul) >> self.shg) as u8,
                ((self.b_sum * self.mul) >> self.shg) as u8,
                ((self.a_sum * self.mul) >> self.shg) as u8,
            ])
        }
    }

    struct RGBHProcessor<'a> {
        data: &'a [u8],
        width: usize,
        height: usize,
        radius: usize,
        current_index: usize,
        current_row: usize,
        r_store: Vec<u8>,
        g_store: Vec<u8>,
        b_store: Vec<u8>,
        r_sum: usize,
        g_sum: usize,
        b_sum: usize,
        w3: usize,
        rr12: usize,
        r_minus_1: usize,
        w_minus_1: usize,
        mul: usize,
        shg: usize,
    }

    impl<'a> RGBHProcessor<'a> {
        fn new(data: &'a [u8], width: usize, height: usize, radius: usize) -> Self {
            Self {
                data,
                width,
                height,
                radius,
                current_index: 0,
                current_row: 0,
                r_store: vec![data[0]; radius],
                g_store: vec![data[1]; radius],
                b_store: vec![data[2]; radius],
                r_sum: (data[0] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[3 * x + 0] as usize * (radius - 1 - x)
                    }),
                g_sum: (data[1] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[3 * x + 1] as usize * (radius - 1 - x)
                    }),
                b_sum: (data[2] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[3 * x + 2] as usize * (radius - 1 - x)
                    }),
                w3: width * 3,
                rr12: radius * (radius + 1) / 2,
                r_minus_1: radius - 1,
                w_minus_1: width - 1,
                mul: MUL_TABLE[radius - 1],
                shg: SHG_TABLE[radius - 1],
            }
        }
    }

    impl Iterator for RGBHProcessor<'_> {
        type Item = [u8; 3];
        fn next(&mut self) -> Option<Self::Item> {
            if self.current_index == self.width {
                self.current_row += 1;
                if self.current_row == self.height {
                    return None;
                }
                self.current_index = 0;
                let rw3 = self.current_row * self.w3;

                self.r_store = vec![self.data[rw3 + 0]; self.radius];
                self.g_store = vec![self.data[rw3 + 1]; self.radius];
                self.b_store = vec![self.data[rw3 + 2]; self.radius];

                self.r_sum = (self.data[rw3 + 0] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw3 + 3 * x + 0] as usize * (self.r_minus_1 - x)
                    });
                self.g_sum = (self.data[rw3 + 1] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw3 + 3 * x + 1] as usize * (self.r_minus_1 - x)
                    });
                self.b_sum = (self.data[rw3 + 2] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[rw3 + 3 * x + 2] as usize * (self.r_minus_1 - x)
                    });
            }

            let rw3 = self.current_row * self.w3;
            let store_len = self.r_store.len();

            self.r_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw3
                        + 3 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 0] as usize
                })
                .sum::<usize>();
            self.r_sum -= (&self.r_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.g_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw3
                        + 3 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 1] as usize
                })
                .sum::<usize>();
            self.g_sum -= (&self.g_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.b_sum += (0..self.radius)
                .map(|rad| {
                    self.data[rw3
                        + 3 * (if self.current_index + rad < self.width {
                            self.current_index + rad
                        } else {
                            self.w_minus_1
                        })
                        + 2] as usize
                })
                .sum::<usize>();
            self.b_sum -= (&self.b_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            let rw3_ci3 = rw3 + 3 * self.current_index;

            let r = self.data[rw3_ci3 + 0];
            let g = self.data[rw3_ci3 + 1];
            let b = self.data[rw3_ci3 + 2];

            self.r_store.push(r);
            self.g_store.push(g);
            self.b_store.push(b);

            self.current_index += 1;

            Some([
                ((self.r_sum * self.mul) >> self.shg) as u8,
                ((self.g_sum * self.mul) >> self.shg) as u8,
                ((self.b_sum * self.mul) >> self.shg) as u8,
            ])
        }
    }

    struct RGBVProcessor<'a> {
        data: &'a [u8],
        width: usize,
        height: usize,
        radius: usize,
        current_index: usize,
        current_col: usize,
        r_store: Vec<u8>,
        g_store: Vec<u8>,
        b_store: Vec<u8>,
        r_sum: usize,
        g_sum: usize,
        b_sum: usize,
        w3: usize,
        rr12: usize,
        r_minus_1: usize,
        h_minus_1: usize,
        mul: usize,
        shg: usize,
    }

    impl<'a> RGBVProcessor<'a> {
        fn new(data: &'a [u8], width: usize, height: usize, radius: usize) -> Self {
            Self {
                data,
                width,
                height,
                radius,
                current_index: 0,
                current_col: 0,
                r_store: vec![data[0]; radius],
                g_store: vec![data[1]; radius],
                b_store: vec![data[2]; radius],
                r_sum: (data[0] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 3 * x + 0] as usize * (radius - 1 - x)
                    }),
                g_sum: (data[1] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 3 * x + 1] as usize * (radius - 1 - x)
                    }),
                b_sum: (data[2] as usize * radius * (radius + 1) / 2)
                    + (0..(radius - 1)).fold(0, |acc, x| {
                        acc + data[width * 3 * x + 2] as usize * (radius - 1 - x)
                    }),
                w3: width * 3,
                rr12: radius * (radius + 1) / 2,
                r_minus_1: radius - 1,
                h_minus_1: height - 1,
                mul: MUL_TABLE[radius - 1],
                shg: SHG_TABLE[radius - 1],
            }
        }
    }

    impl Iterator for RGBVProcessor<'_> {
        type Item = [u8; 3];
        fn next(&mut self) -> Option<Self::Item> {
            if self.current_index == self.height {
                self.current_col += 1;
                if self.current_col == self.width {
                    return None;
                }
                self.current_index = 0;
                let c3 = self.current_col * 3;

                self.r_store = vec![self.data[c3 + 0]; self.radius];
                self.g_store = vec![self.data[c3 + 1]; self.radius];
                self.b_store = vec![self.data[c3 + 2]; self.radius];

                self.r_sum = (self.data[c3 + 0] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c3 + self.w3 * x + 0] as usize * (self.r_minus_1 - x)
                    });
                self.g_sum = (self.data[c3 + 1] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c3 + self.w3 * x + 1] as usize * (self.r_minus_1 - x)
                    });
                self.b_sum = (self.data[c3 + 2] as usize * self.rr12)
                    + (0..(self.r_minus_1)).fold(0, |acc, x| {
                        acc + self.data[c3 + self.w3 * x + 2] as usize * (self.r_minus_1 - x)
                    });
            }

            let c3 = self.current_col * 3;
            let store_len = self.r_store.len();

            self.r_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c3
                        + self.w3
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 0] as usize
                })
                .sum::<usize>();
            self.r_sum -= (&self.r_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.g_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c3
                        + self.w3
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 1] as usize
                })
                .sum::<usize>();
            self.g_sum -= (&self.g_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            self.b_sum += (0..self.radius)
                .map(|rad| {
                    self.data[c3
                        + self.w3
                            * (if self.current_index + rad < self.height {
                                self.current_index + rad
                            } else {
                                self.h_minus_1
                            })
                        + 2] as usize
                })
                .sum::<usize>();
            self.b_sum -= (&self.b_store[(store_len - self.radius)..])
                .iter()
                .map(|x| *x as usize)
                .sum::<usize>();

            let c3w3ci = c3 + self.w3 * self.current_index;

            let r = self.data[c3w3ci + 0];
            let g = self.data[c3w3ci + 1];
            let b = self.data[c3w3ci + 2];

            self.r_store.push(r);
            self.g_store.push(g);
            self.b_store.push(b);

            self.current_index += 1;

            Some([
                ((self.r_sum * self.mul) >> self.shg) as u8,
                ((self.g_sum * self.mul) >> self.shg) as u8,
                ((self.b_sum * self.mul) >> self.shg) as u8,
            ])
        }
    }

    pub fn unoptimized_blur(
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        r: u8,
    ) -> Vec<u8> {
        let r = r as usize;
        let radius = (r * 2) + 1;
        let mul = MUL_TABLE[r];
        let shg = SHG_TABLE[r];

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

    pub fn unoptimized_blur_2(
        data: &[u8],
        width: usize,
        height: usize,
        channels: usize,
        r: u8,
    ) -> Vec<u8> {
        let r = r as usize;
        let radius = (r * 2) + 1;
        let mul = MUL_TABLE[r];
        let shg = SHG_TABLE[r];

        let hor_out_vec: Vec<u8> = (0..height)
            .map(|row| {
                let rwc = row * width * channels;

                (0..width)
                    .map(move |i| {
                        let ic = i * channels;

                        (0..channels).map(move |channel| {
                            let sum = (0..radius).fold(0usize, |acc, idx| {
                                if r > i + idx {
                                    if idx < r {
                                        acc + data[rwc + ic + channel] as usize * (idx + 1)
                                    } else {
                                        acc + data[rwc + ic + channel] as usize * (radius - idx)
                                    }
                                } else if i + idx - r < width {
                                    if idx < r {
                                        acc + data[rwc + (i + idx - r) * channels + channel]
                                            as usize
                                            * (idx + 1)
                                    } else {
                                        acc + data[rwc + (i + idx - r) * channels + channel]
                                            as usize
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

                            ((sum * mul) >> shg) as u8
                        })
                    })
                    .flatten()
            })
            .flatten()
            .collect();
        let hor_out = &hor_out_vec[..];

        let ver_out: Vec<u8> = (0..width)
            .map(|col| {
                let cc = col * channels;

                (0..height)
                    .map(move |i| {
                        let iwc = i * width * channels;

                        (0..channels).map(move |channel| {
                            let sum = (0..radius).fold(0usize, |acc, idx| {
                                if r > i + idx {
                                    if idx < r {
                                        acc + hor_out[cc + iwc + channel] as usize * (idx + 1)
                                    } else {
                                        acc + hor_out[cc + iwc + channel] as usize * (radius - idx)
                                    }
                                } else if i + idx - r < height {
                                    if idx < r {
                                        acc + hor_out
                                            [cc + (i + idx - r) * width * channels + channel]
                                            as usize
                                            * (idx + 1)
                                    } else {
                                        acc + hor_out
                                            [cc + (i + idx - r) * width * channels + channel]
                                            as usize
                                            * (radius - idx)
                                    }
                                } else {
                                    if idx < r {
                                        acc + hor_out
                                            [cc + (height - 1) * width * channels + channel]
                                            as usize
                                            * (idx + 1)
                                    } else {
                                        acc + hor_out
                                            [cc + (height - 1) * width * channels + channel]
                                            as usize
                                            * (radius - idx)
                                    }
                                }
                            });
                            ((sum * mul) >> shg) as u8
                        })
                    })
                    .flatten()
            })
            .flatten()
            .collect();

        ver_out
    }

    #[rustfmt::skip]
    const MUL_TABLE: [usize; 256] = [
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
    const SHG_TABLE: [usize; 256] = [
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
}
