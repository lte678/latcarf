mod glium_sdl2;
mod text_rendering;

use image::{ImageBuffer, Rgb};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::gfx::primitives::DrawRenderer;
use glium::VertexBuffer;
use glium::Surface;
use clap::Parser;
use itertools::Itertools;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::process;
use env_logger;

use crate::glium_sdl2::DisplayBuild;
use crate::text_rendering::{load_default_fonts, generate_atlas};

#[macro_use]
extern crate glium;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    device: Option<String>,
    #[arg(short, long)]
    debug: bool,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);


type FracFloat = f64;
const MAX_ITERATIONS: u32 = 200;


fn main() {
    env_logger::init();
    let cli = Args::parse();

    // Prepare logging
    let mut log_builder = env_logger::builder();
    if cli.debug {
        log_builder.filter_level(log::LevelFilter::Debug);
    }
    log_builder.init();

    // Pick device to run on
    if let Some(device) = cli.device {
        if device == "cpu" {
            cpu_mode();
        } else if device == "gpu" {
            gpu_mode();
        } else {
            println!("Invalid device '{}'", device);
        }
    } else {
        gpu_mode();
    }
}


fn print_glsl_error(err: glium::ProgramCreationError, frag_shader: &str, vert_shader: &str) {
    println!("Failed to compile shader:");
    if let glium::CompilationError(compile_err, shader_type) = err {
        let line_number: usize = (&compile_err)
            .split(":").nth(1).unwrap()
            .split("(").nth(0).unwrap()
            .parse().unwrap();
        print!("{}", &compile_err);
        println!("In line {line_number}:");
        match shader_type {
            glium::program::ShaderType::Fragment => println!("{}", frag_shader.split("\n").nth(line_number-1).unwrap()),
            glium::program::ShaderType::Vertex => println!("{}", vert_shader.split("\n").nth(line_number-1).unwrap()),
            _ => (),
        }
    } else {
        println!("{}", err);
    }
}


fn gpu_mode() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl = video_subsystem.window("latcarf", 1920, 1080)
        .position_centered()
        .build_glium()
        .unwrap();
    // video_subsystem.gl_set_swap_interval(SwapInterval::Immediate).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    println!("Initialized GPU context.");

    let font = load_default_fonts(); 
    let font_atlas: font_kit::canvas::Canvas = generate_atlas(&font, 32.0, None);
    let font_atlas_img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        font_atlas.size.x() as u32, font_atlas.size.y() as u32, font_atlas.pixels
    ).unwrap();
    font_atlas_img.save("font_atlas.png").unwrap();



    let demo_rectangle = vec![
        Vertex{ position: [-1.0, -1.0] },
        Vertex{ position: [ 1.0, -1.0] },
        Vertex{ position: [-1.0,  1.0] },
        Vertex{ position: [ 1.0, -1.0] },
        Vertex{ position: [ 1.0,  1.0] },
        Vertex{ position: [-1.0,  1.0] },
    ];

    let vert_shader = String::from_utf8_lossy(include_bytes!("../res/mandelbrot.vert"));
    let frag_shader_preamble = "#version 330";
    let frag_shader_colormap = String::from_utf8_lossy(include_bytes!("../res/IDL_Waves.frag"));
    let frag_shader_main = String::from_utf8_lossy(include_bytes!("../res/mandelbrot.frag"));
    let frag_shader = [frag_shader_preamble, &frag_shader_colormap, &frag_shader_main].join("\n");

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let shader = match glium::Program::from_source(&gl, &vert_shader, &frag_shader, None) {
        Ok(s) => s,
        Err(shader_err) => { print_glsl_error(shader_err, &frag_shader, &vert_shader); process::exit(1) }
    };
    let vbo = VertexBuffer::new(&gl, &demo_rectangle).unwrap();
    let (w, h) = gl.get_framebuffer_dimensions();
    let mut scale = 2.0 / u32::min(w, h) as f32;
    let mut offset = (0.0, 0.0);

    let mut frametimes: VecDeque<u64> = VecDeque::new();
    loop {
        let (w, h) = gl.get_framebuffer_dimensions();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => return,
                Event::MouseWheel {precise_y, ..} => scale *= (-0.1 * precise_y).exp(),
                Event::MouseMotion {mousestate, xrel, yrel, ..} if mousestate.left() => {offset.0 -= (xrel as f32)*scale; offset.1 += (yrel as f32)*scale},
                _ => ()
            }
        }
    
        let render_start_t = Instant::now();
        // RENDER START
        let mut render_tgt = gl.draw();
        render_tgt.clear_color(0.0, 0.0, 0.0, 1.0);
        render_tgt.draw(
            &vbo,
            &indices,
            &shader,
            &uniform!{offset: offset, scale: scale, window_size: (w as f32, h as f32)},
            &Default::default()
        ).unwrap();
        render_tgt.finish().unwrap();
        // RENDER END
        frametimes.push_back(render_start_t.elapsed().as_nanos() as u64);
        if frametimes.len() > 10 {
            let avg_frametime: u64 = (frametimes.iter().sum::<u64>() / frametimes.len() as u64) / 1000;
            println!("Time per frame: {avg_frametime}us");
            frametimes.pop_front();
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}


fn cpu_mode() {
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
        render_cpu(&mut canvas, (0.0, 0.0)).unwrap();
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


fn render_cpu<T: RenderTarget>(canvas: &mut Canvas<T>, offset: (FracFloat, FracFloat)) -> Result<(), String> {
    let (w, h) = canvas.output_size()?;
    let (real_offset, imag_offset) = offset;
    let scale: FracFloat = 4.0 / (w as FracFloat);
    for (x, y) in (0..w).cartesian_product(0..h) {
        let c_real = (x as FracFloat - 0.5 * w as FracFloat) * scale + real_offset;
        let c_imag = (y as FracFloat - 0.5 * h as FracFloat) * scale + imag_offset;
        if let Some((_, dist)) = mandelbrot_depth(c_real, c_imag) {
            if dist > scale*0.25 {
                canvas.pixel(x as i16, y as i16, Color::RGB(255, 255, 255))?;
            }
        }
    }
    Ok(())
}


/// Calculates the depth of the mandelbrot fractal for given C real and imaginary part.
/// Returns tuple of depth and distance to set if outside of the set.
fn mandelbrot_depth(c_real: FracFloat, c_imag: FracFloat) -> Option<(u32, FracFloat)> {
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
    let mut z_prime_real: FracFloat = 1.0;
    let mut z_prime_imag: FracFloat = 0.0;
    let mut i: u32 = 0;
    while (z_real_sq + z_imag_sq) < 4.0 {
        let z_prime_rtmp = z_prime_real;
        z_prime_real = 2.0*(z_real*z_prime_real - z_imag*z_prime_imag) + 1.0;
        z_prime_imag = 2.0*(z_real*z_prime_imag + z_imag*z_prime_rtmp);
        z_imag = 2.0*z_real*z_imag + c_imag;
        z_real = z_real_sq - z_imag_sq + c_real;
        z_real_sq = z_real * z_real;
        z_imag_sq = z_imag * z_imag;

        i += 1;
        if i >= MAX_ITERATIONS {
            return None
        }
    }
    let z_mag = (z_real_sq + z_imag_sq).sqrt();
    let z_prime_mag = (z_prime_real*z_prime_real + z_prime_imag*z_prime_imag).sqrt();
    let dist = z_mag*z_mag.ln()/z_prime_mag;
    Some((i, dist))
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
