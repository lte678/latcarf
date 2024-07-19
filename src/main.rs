extern crate sdl2;
extern crate itertools;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::gfx::primitives::DrawRenderer;
use itertools::Itertools;

use std::time::{Duration, Instant};
use std::collections::VecDeque;


type FracFloat = f64;
const MAX_ITERATIONS: u32 = 200;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("latcarf", 1920, 1080)
        .position_centered()
        .build()
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    println!("Initialized window manager.");
    
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    
    let mut frametimes: VecDeque<u64> = VecDeque::new();
    loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => return,
                _ => ()
            }
        }
    
        let render_start_t = Instant::now();
        render(&mut canvas, (0.0, 0.0)).unwrap();
        frametimes.push_back(render_start_t.elapsed().as_nanos() as u64);
        if frametimes.len() > 10 {
            let avg_frametime: u64 = (frametimes.iter().sum::<u64>() / frametimes.len() as u64) / 1000;
            println!("Time per frame: {avg_frametime}us");
            frametimes.pop_front();
        }
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}


fn render<T: RenderTarget>(canvas: &mut Canvas<T>, offset: (FracFloat, FracFloat)) -> Result<(), String> {
    let (w, h) = canvas.output_size()?;
    let (real_offset, imag_offset) = offset;
    let scale: FracFloat = 4.0 / (w as FracFloat);
    for (x, y) in (0..w).cartesian_product(0..h) {
        let c_real = (x as FracFloat - 0.5 * w as FracFloat) * scale + real_offset;
        let c_imag = (y as FracFloat - 0.5 * h as FracFloat) * scale + imag_offset;
        match mandelbrot_depth(c_real, c_imag) {
            Some(depth) => canvas.pixel(x as i16, y as i16, Color::RGB(255, 255, 255))?,
            None => (),
        }
    }
    Ok(())
}

/// Calculates the depth of the mandelbrot fractal for given C real and imaginary part.
fn mandelbrot_depth(c_real: FracFloat, c_imag: FracFloat) -> Option<u32> {
    // z_n+1 = z_n^2 + c
    // Translated from complex into real operations (indices omitted):
    // "next iteration" = (z_real + z_imag*i)^2 + c_real + c_imag*i
    // "next iteration" = z_real^2 - z_imag^2 + 2*z_real*z_imag*i + c_real + c_imag*i
    // z_n+1_real = z_n_real^2 - z_n_imag^2 + c_real
    // z_n+1_imag = 2*z_n_real*z_n_imag + c_imag
    let mut z_real_sq: FracFloat = 0.0;
    let mut z_imag_sq: FracFloat = 0.0;
    let mut z_real: FracFloat = 0.0;
    let mut z_imag: FracFloat = 0.0;
    for i in 0..MAX_ITERATIONS {
        z_imag = 2.0*z_real*z_imag + c_imag;
        z_real = z_real_sq - z_imag_sq + c_real;
        z_real_sq = z_real * z_real;
        z_imag_sq = z_imag * z_imag;
        if (z_real_sq + z_imag_sq) > 2.0 {
            return Some(i)
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandelbrot_depth() {
        assert!(mandelbrot_depth(0.0, 0.0).is_none());
        assert!(mandelbrot_depth(2.1, 0.0).is_some());
        assert!(mandelbrot_depth(0.0, 2.1).is_some());
        
    }
}