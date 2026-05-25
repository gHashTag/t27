pub struct Mandelbrot;

impl Mandelbrot {
    pub fn escape_time(cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut zx = 0.0f64;
        let mut zy = 0.0f64;
        let mut i = 0u32;
        while i < max_iter && zx * zx + zy * zy <= 4.0 {
            let tmp = zx * zx - zy * zy + cx;
            zy = 2.0 * zx * zy + cy;
            zx = tmp;
            i += 1;
        }
        i
    }

    pub fn is_member(cx: f64, cy: f64, max_iter: u32) -> bool {
        Self::escape_time(cx, cy, max_iter) == max_iter
    }

    pub fn render(width: usize, height: usize, max_iter: u32) -> Vec<Vec<u32>> {
        let x_min = -2.5f64;
        let x_max = 1.0f64;
        let y_min = -1.0f64;
        let y_max = 1.0f64;
        let mut grid = vec![vec![0u32; width]; height];
        for py in 0..height {
            for px in 0..width {
                let cx = x_min + (px as f64 / width as f64) * (x_max - x_min);
                let cy = y_min + (py as f64 / height as f64) * (y_max - y_min);
                grid[py][px] = Self::escape_time(cx, cy, max_iter);
            }
        }
        grid
    }

    pub fn smooth_escape(cx: f64, cy: f64, max_iter: u32) -> f64 {
        let mut zx = 0.0f64;
        let mut zy = 0.0f64;
        let mut i = 0u32;
        while i < max_iter && zx * zx + zy * zy <= 4.0 {
            let tmp = zx * zx - zy * zy + cx;
            zy = 2.0 * zx * zy + cy;
            zx = tmp;
            i += 1;
        }
        if i == max_iter { return max_iter as f64; }
        let log_zn = (zx * zx + zy * zy).ln() / 2.0;
        let nu = (log_zn / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
        i as f64 + 1.0 - nu
    }

    pub fn julia_escape(zx: f64, zy: f64, cx: f64, cy: f64, max_iter: u32) -> u32 {
        let mut zx = zx; let mut zy = zy;
        let mut i = 0u32;
        while i < max_iter && zx * zx + zy * zy <= 4.0 {
            let tmp = zx * zx - zy * zy + cx;
            zy = 2.0 * zx * zy + cy;
            zx = tmp;
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_member() { assert!(Mandelbrot::is_member(0.0, 0.0, 100)); }

    #[test]
    fn outside() { assert!(!Mandelbrot::is_member(3.0, 0.0, 100)); }

    #[test]
    fn escape_time_known() {
        assert_eq!(Mandelbrot::escape_time(0.0, 0.0, 100), 100);
        assert!(Mandelbrot::escape_time(3.0, 0.0, 100) < 5);
    }

    #[test]
    fn render_dimensions() {
        let grid = Mandelbrot::render(80, 40, 50);
        assert_eq!(grid.len(), 40);
        assert_eq!(grid[0].len(), 80);
    }

    #[test]
    fn smooth() {
        let s = Mandelbrot::smooth_escape(0.5, 0.5, 1000);
        assert!(s < 1000.0);
    }
}
