use glam::{DVec2, DVec3};
use rand_core::RngCore;

#[derive(Clone)]
pub struct SimplexNoise {
    perm: [u8; 1024],
}

const GRAD_X: [f64; 12] = [1., -1., 1.0, -1., 1., -1., 1.0, -1., 0., 0.0, 0.0, 0.0];
const GRAD_Y: [f64; 12] = [1., 1.0, -1., -1., 0., 0.0, 0.0, 0.0, 1., -1., 1.0, -1.];
const GRAD_Z: [f64; 12] = [0., 0.0, 0.0, 0.0, 1., 1.0, -1., -1., 1., 1.0, -1., -1.];

const F3: f64 = 1.0 / 3.0;
const G3: f64 = 1.0 / 6.0;
#[allow(clippy::excessive_precision)]
const SQRT3: f64 = 1.7320508075688772935274463415059;
const F2: f64 = 0.5 * (SQRT3 - 1.0);
const G2: f64 = (3.0 - SQRT3) / 6.0;

impl SimplexNoise {
    pub fn new<R: RngCore>(rng: &mut R) -> Self {
        let mut noise = Self { perm: [0_u8; 1024] };
        noise.reset(rng);
        noise
    }

    pub fn reset<R: RngCore>(&mut self, rng: &mut R) {
        let ptr = self.perm.as_mut_ptr();

        for i in 0..=255 {
            unsafe {
                *ptr.add(i as usize) = i;
            }
        }

        for j in 0..=255 {
            let rng = rng.next_u64() % (256 - j);
            let k = rng + j;
            unsafe {
                let l = *ptr.add(j as usize);
                *ptr.add(j as usize) = *ptr.add(k as usize);
                *ptr.add(j as usize + 256) = *ptr.add(k as usize);
                *ptr.add(k as usize) = l;
                *ptr.add(512 + j as usize) = *ptr.add(j as usize) % 12;
                *ptr.add(512 + j as usize + 256) = *ptr.add(j as usize) % 12;
            }
        }
    }

    pub fn get_3d(&self, offset: u8, pos: DVec3) -> f64 {
        let DVec3 { x, y, z } = pos;
        let perm = unsafe { &*(self.perm.as_ptr().cast::<[u8; 512]>()) };
        let perm12 = unsafe { &*(self.perm.as_ptr().add(512).cast::<[u8; 512]>()) };

        let mut t: f64 = (x + y + z) * F3;
        let i = (x + t).floor() as i32;
        let j = (y + t).floor() as i32;
        let k = (z + t).floor() as i32;

        t = (i + j + k) as f64 * G3;
        let x0 = i as f64 - t;
        let y0 = j as f64 - t;
        let z0 = k as f64 - t;

        let x0 = x - x0;
        let y0 = y - y0;
        let z0 = z - z0;

        let i1: f64;
        let j1: f64;
        let k1: f64;
        let i2: f64;
        let j2: f64;
        let k2: f64;

        if x0 >= y0 {
            if y0 >= z0 {
                i1 = 1.;
                j1 = 0.;
                k1 = 0.;
                i2 = 1.;
                j2 = 1.;
                k2 = 0.;
            } else if x0 >= z0 {
                i1 = 1.;
                j1 = 0.;
                k1 = 0.;
                i2 = 1.;
                j2 = 0.;
                k2 = 1.;
            } else {
                i1 = 0.;
                j1 = 0.;
                k1 = 1.;
                i2 = 1.;
                j2 = 0.;
                k2 = 1.;
            }
        } else if y0 < z0 {
            i1 = 0.;
            j1 = 0.;
            k1 = 1.;
            i2 = 0.;
            j2 = 1.;
            k2 = 1.;
        } else if x0 < z0 {
            i1 = 0.;
            j1 = 1.;
            k1 = 0.;
            i2 = 0.;
            j2 = 1.;
            k2 = 1.;
        } else {
            i1 = 0.;
            j1 = 1.;
            k1 = 0.;
            i2 = 1.;
            j2 = 1.;
            k2 = 0.;
        }

        let x1 = x0 - i1 + G3;
        let y1 = y0 - j1 + G3;
        let z1 = z0 - k1 + G3;
        let x2 = x0 - i2 + 2.0 * G3;
        let y2 = y0 - j2 + 2.0 * G3;
        let z2 = z0 - k2 + 2.0 * G3;
        let x3 = x0 - 1. + 3.0 * G3;
        let y3 = y0 - 1. + 3.0 * G3;
        let z3 = z0 - 1. + 3.0 * G3;

        t = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
        let n0 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t * grad_coord_3d(perm12, perm, offset, i, j, k, x0, y0, z0)
        };

        t = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
        let n1 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t
                * grad_coord_3d(
                    perm12,
                    perm,
                    offset,
                    i + i1 as i32,
                    j + j1 as i32,
                    k + k1 as i32,
                    x1,
                    y1,
                    z1,
                )
        };

        t = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
        let n2 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t
                * grad_coord_3d(
                    perm12,
                    perm,
                    offset,
                    i + i2 as i32,
                    j + j2 as i32,
                    k + k2 as i32,
                    x2,
                    y2,
                    z2,
                )
        };

        t = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
        let n3 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t * grad_coord_3d(perm12, perm, offset, i + 1, j + 1, k + 1, x3, y3, z3)
        };

        32.0 * (n0 + n1 + n2 + n3)
    }

    pub fn get(&self, offset: u8, pos: DVec2) -> f64 {
        let DVec2 { x, y } = pos;
        let perm = unsafe { &*(self.perm.as_ptr().cast::<[u8; 512]>()) };
        let perm12 = unsafe { &*(self.perm.as_ptr().add(512).cast::<[u8; 512]>()) };

        let mut t: f64 = (x + y) * F2;
        let i = (x + t).floor() as i32;
        let j = (y + t).floor() as i32;

        t = (i + j) as f64 * G2;
        let x0 = i as f64 - t;
        let y0 = j as f64 - t;

        let x0 = x - x0;
        let y0 = y - y0;

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f64 + G2;
        let y1 = y0 - j1 as f64 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        t = 0.5 - x0 * x0 - y0 * y0;
        let n0 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t * grad_coord_2d(perm12, perm, offset, i, j, x0, y0)
        };

        t = 0.5 - x1 * x1 - y1 * y1;
        let n1 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t * grad_coord_2d(perm12, perm, offset, i + i1, j + j1, x1, y1)
        };

        t = 0.5 - x2 * x2 - y2 * y2;
        let n2 = if t < 0. {
            0.
        } else {
            t *= t;
            t * t * grad_coord_2d(perm12, perm, offset, i + 1, j + 1, x2, y2)
        };

        70.0 * (n0 + n1 + n2)
    }
}

fn index2d_12(perm12: &[u8; 512], perm: &[u8; 512], offset: u8, x: i32, y: i32) -> u8 {
    unsafe {
        let a = *perm.get_unchecked((y & 0xff) as usize + offset as usize) as usize;
        *perm12.get_unchecked((x & 0xff) as usize + a)
    }
}

fn index3d_12(perm12: &[u8; 512], perm: &[u8; 512], offset: u8, x: i32, y: i32, z: i32) -> u8 {
    unsafe {
        let a = *perm.get_unchecked((z & 0xff) as usize + offset as usize) as usize;
        let b = *perm.get_unchecked((y & 0xff) as usize + a) as usize;
        *perm12.get_unchecked((x & 0xff) as usize + b)
    }
}

fn grad_coord_2d(
    perm12: &[u8; 512],
    perm: &[u8; 512],
    offset: u8,
    x: i32,
    y: i32,
    xd: f64,
    yd: f64,
) -> f64 {
    let lut_pos = index2d_12(perm12, perm, offset, x, y) as usize;
    unsafe { xd * *GRAD_X.get_unchecked(lut_pos) + yd * *GRAD_Y.get_unchecked(lut_pos) }
}

#[allow(clippy::too_many_arguments)]
fn grad_coord_3d(
    perm12: &[u8; 512],
    perm: &[u8; 512],
    offset: u8,
    x: i32,
    y: i32,
    z: i32,
    xd: f64,
    yd: f64,
    zd: f64,
) -> f64 {
    let lut_pos = index3d_12(perm12, perm, offset, x, y, z) as usize;
    unsafe {
        xd * *GRAD_X.get_unchecked(lut_pos)
            + yd * *GRAD_Y.get_unchecked(lut_pos)
            + zd * *GRAD_Z.get_unchecked(lut_pos)
    }
}
