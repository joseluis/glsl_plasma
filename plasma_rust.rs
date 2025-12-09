#!/usr/bin/env -S rust-script -c
//! ```cargo
//! [package]
//! name = "plasma_rust"
//! ```
// $ ./plasma_rust.rs
// $ ffmpeg -i out-rust-%03d.ppm -r 60 out-rust.mp4

#![allow(non_camel_case_types)]

const FRAMES: usize = 30; // original 240

use std::{fs::File, io::Write};

fn main() {
    generate_frames(|frame, w, data| {
        write_ppm(frame, w, data);
    });
}

fn write_ppm(frame: usize, w: usize, data: &[u8]) {
    let h = data.len() / (w * 3);

    let path = format!("out-rust-{:03}.ppm", frame);
    let mut f = File::create(&path).expect("create ppm file");

    writeln!(f, "P6").unwrap();
    writeln!(f, "{} {}", w, h).unwrap();
    writeln!(f, "255").unwrap();

    f.write_all(data).unwrap();

    println!("Generated {} ({:3}/{FRAMES})", path, frame + 1);
}

pub fn generate_frames<F>(mut write_frame: F)
where
    F: FnMut(usize, usize, &[u8]), // (frame_index, width, raw_rgb_bytes)
{
    let w = 16 * 60;
    let h = 9 * 60;
    let r = vec2::new(w as f32, h as f32);

    // temporary RGB buffer reused every frame
    let mut buf = vec![0u8; w * h * 3];

    for frame in 0..FRAMES {
        let t = (frame as f32 / FRAMES as f32) * 2.0 * core::f32::consts::PI;
        let mut offset = 0;
        for y in 0..h {
            for x in 0..w {
                let fc = vec2::new(x as f32, y as f32);
                let p = (fc * 2.0 - r) / r.y;
                let mut l = vec2::default();
                l = l + (4.0 - 4.0 * (0.7 - p.dot(p)).abs());

                let mut v = p * l;
                let mut o = vec4::default();
                for iy in 1..=8 {
                    let iyf = iy as f32;
                    let tmp0 = (v.yx() * iyf + vec2::new(0.0, iyf) + t).cos() / iyf + 0.7;
                    v += tmp0;
                    let tmp1 = (v.xyyx().sin() + 1.0) * (v.x - v.y).abs();
                    o += tmp1;
                }

                let exp_term = (l.x - 4.0 - p.y * vec4::new(-1.0, 1.0, 2.0, 0.0)).exp() * 5.0;
                o = (exp_term / o).tanh();

                // write to buffer
                buf[offset] = (o.x.clamp(0.0, 1.0) * 255.0) as u8;
                buf[offset + 1] = (o.y.clamp(0.0, 1.0) * 255.0) as u8;
                buf[offset + 2] = (o.z.clamp(0.0, 1.0) * 255.0) as u8;
                offset += 3;
            }
        }
        write_frame(frame, w, &buf);
    }
}

pub use impl_math::*;
#[rustfmt::skip]
mod impl_math {
    use std::ops::{Mul, Add, Sub, Div, AddAssign};

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct vec2 { pub x: f32, pub y: f32 }
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

    impl vec2 {
        pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
        pub const fn yx(self) -> vec2 { Self::new(self.y, self.x) }
        pub const fn xyyx(self) -> vec4 { vec4::new(self.x, self.y, self.y, self.x) }
        pub fn dot(self, rhs: Self) -> f32 { self.x * rhs.x + self.y * rhs.y }
        pub fn abs(self) -> Self { Self::new(self.x.abs(), self.y.abs()) }
        pub fn cos(self) -> Self { Self::new(self.x.cos(), self.y.cos()) }
    }
    impl Add<f32> for vec2 {
        type Output = vec2;
        fn add(self, rhs: f32) -> Self::Output { vec2::new(self.x+rhs, self.y+rhs) }
    }
    impl Mul<f32> for vec2 {
        type Output = vec2;
        fn mul(self, rhs: f32) -> Self::Output { vec2::new(self.x*rhs, self.y*rhs) }
    }
    impl Div<f32> for vec2 {
        type Output = vec2;
        fn div(self, rhs: f32) -> Self::Output { vec2::new(self.x/rhs, self.y/rhs) }
    }

    impl Add<vec2> for vec2 {
        type Output = vec2;
        fn add(self, rhs: vec2) -> Self::Output { vec2::new(self.x+rhs.x, self.y+rhs.y) }
    }
    impl AddAssign<vec2> for vec2 {
        fn add_assign(&mut self, rhs: vec2) { self.x += rhs.x; self.y += rhs.y; }
    }
    impl Sub<vec2> for vec2 {
        type Output = vec2;
        fn sub(self, rhs: vec2) -> Self::Output { vec2::new(self.x-rhs.x, self.y-rhs.y) }
    }
    impl Mul<vec2> for vec2 {
        type Output = vec2;
        fn mul(self, rhs: vec2) -> Self::Output { vec2::new(self.x*rhs.x, self.y*rhs.y) }
    }

    impl vec4 {
        pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
        pub fn sin(&self) -> Self {
            Self::new(self.x.sin(), self.y.sin(), self.z.sin(), self.w.sin())
        }
        pub fn exp(&self) -> Self {
            Self::new(self.x.exp(), self.y.exp(), self.z.exp(), self.w.exp())
        }
        pub fn tanh(&self) -> Self {
            Self::new(self.x.tanh(), self.y.tanh(), self.z.tanh(), self.w.tanh())
        }
    }
    impl Add<f32> for vec4 {
        type Output = vec4;
        fn add(self, rhs: f32) -> Self::Output {
            vec4::new(self.x+rhs, self.y+rhs, self.z+rhs, self.w+rhs)
        }
    }
    impl Sub<vec4> for f32 {
        type Output = vec4;
        fn sub(self, rhs: vec4) -> Self::Output {
            vec4::new(self-rhs.x, self-rhs.y, self-rhs.z, self-rhs.w)
        }
    }
    impl Mul<f32> for vec4 {
        type Output = vec4;
        fn mul(self, rhs: f32) -> Self::Output {
            vec4::new(self.x*rhs, self.y*rhs, self.z*rhs, self.w*rhs)
        }
    }
    impl Mul<vec4> for f32 {
        type Output = vec4;
        fn mul(self, rhs: vec4) -> Self::Output {
            vec4::new(self*rhs.x, self*rhs.y, self*rhs.z, self*rhs.w)
        }
    }
    impl AddAssign<vec4> for vec4 {
        fn add_assign(&mut self, rhs: vec4) {
            self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; self.w += rhs.w;
        }
    }
    impl Div<vec4> for vec4 {
        type Output = vec4;
        fn div(self, rhs: vec4) -> Self::Output {
            vec4::new(self.x/rhs.x, self.y/rhs.y, self.z/rhs.z, self.w/rhs.w)
        }
    }
}
