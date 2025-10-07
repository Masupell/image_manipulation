use rand::Rng;
use std::f32::consts::PI;
use std::i32;
use std::io::Write;
use std::time::{Duration, Instant};

use image::{Rgba, RgbaImage};
use image::GenericImageView;
use rayon::prelude::*;

type Line = Vec<(u32, u32, f32)>; //x, y, blend_alpha

pub static DRAW_OPACITY: u8 = 100; //130
pub static REMOVE: i16 = 100; //50

pub fn run(path: &str, brightness_threshold: f32, output_path: &str)
{
    let beginning = Instant::now();
    println!("Preparing Image...");
    let mut input_img = image::open(path).unwrap();

    let width = input_img.width() as f32;
    let height = input_img.height() as f32;

    let radius = (width/2.0).min(height/2.0);
    let img_size = radius as u32 * 2;

    input_img = input_img.crop_imm(0, 0, img_size, img_size); // Not Perfect, but fine

    let mut cyan: Vec<f32> = vec![0.0; (img_size * img_size) as usize];
    let mut magenta: Vec<f32> = vec![0.0; (img_size * img_size) as usize];
    let mut yellow: Vec<f32> = vec![0.0; (img_size * img_size) as usize];
    let mut black: Vec<f32> = vec![0.0; (img_size * img_size) as usize];

    let mut canvas: Vec<[f32; 4]> = vec![[0.0; 4]; (img_size * img_size) as usize];

    for y in 0..img_size
    {
        for x in 0..img_size
        {
            let idx = (y * img_size + x) as usize;
            let pixel = input_img.get_pixel(x as u32, y as u32);
            let red = pixel[0] as f32 / 255.0;
            let green = pixel[1] as f32 / 255.0;
            let blue = pixel[2] as f32 / 255.0;
            (cyan[idx], magenta[idx], yellow[idx], black[idx]) = rgb_to_cmyk(red, green, blue);
        }
    }

    // Pins
    let mut pins: Vec<(u32, u32)> = Vec::new();

    let center = radius;

    let amount = 360;

    let angle = 360/amount;

    for i in 0..360/angle
    {
        let x = (center + (radius - 10.0) * (i as f32 * angle as f32 * PI / 180.0).cos()).round() as u32;
        let y = (center + (radius - 10.0) * (i as f32 * angle as f32 * PI / 180.0).sin()).round() as u32;

        pins.push((x, y));
    }

    //Output Image + Make Background White
    let mut output_img = RgbaImage::new(img_size, img_size);

    for y in 0..output_img.height()
    {
        for x in 0..output_img.width()
        {
            output_img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }


    //Precompute lines (to not calculate lines during loop, faster)
    let mut precomputed_lines: Vec<Vec<Line>> = vec![vec![Vec::new(); amount]; amount];
    for x in 0..amount
    {
        for y in 0..amount
        {
            if x == y { continue; }
            let (x0, y0) = pins[x];
            let (x1, y1) = pins[y];
            precomputed_lines[x][y] = get_anitaliased_line(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        }
    }


    // Starting Pin
    let mut pin_num = rand::thread_rng().gen_range(0..pins.len());
    // let mut pin = pins[0];


    let start_time = Instant::now();
    let mut last_update = Instant::now();

    let mut last_i = 0;

    // let mut min_vals: [f32; 4] = [1.0; 4];
    let mut max_vals: [f32; 4] = [0.0; 4];

    // let max_iterations = 1000000;
    for (i, a) in [&mut cyan, &mut magenta, &mut yellow, &mut black].iter_mut().enumerate()
    {
        // let iterations = if i == 3 { 75000 } else { 25000 };
        let strength = if i == 3 { 0.8 } else { 0.3 };
        for _ in 0..25000
        {
            let (best, _) = (0..amount).into_par_iter().filter(|&pin_number| pin_number != pin_num).map(|pin_number| 
            {
                let line = &precomputed_lines[pin_num][pin_number];
                let score = average_brightness(*a, line, img_size);
                (pin_number, score)
            }).reduce_with(|a, b| if a.1 > b.1 { a } else { b }).unwrap();

            let best_line = &precomputed_lines[pin_num][best];

            for (x, y, alpha) in best_line.iter()
            {
                let idx = (y * img_size + x) as usize;
                let diff = a[idx] - canvas[idx][i]; 
                let delta = diff * strength * alpha;
                a[idx] = (a[idx] - delta).max(0.0); // 0.05 temporary, thats just testing I guess?
                canvas[idx][i] = (canvas[idx][i] + delta).min(1.0);
                // min_vals[i] = min_vals[i].min(delta);
                max_vals[i] = max_vals[i].max(delta);
            }

            pin_num = best;
        }
        println!("{}. Loop finished (Off four)", i+1);
    }
    println!("\nOne Moment");

    // for [c, m, y, k] in &mut canvas 
    // {
    //     *c /= max_vals[0].max(1e-6);
    //     *m /= max_vals[1].max(1e-6);
    //     *y /= max_vals[2].max(1e-6);
    //     *k /= max_vals[3].max(1e-6);
    // }



    for y in 0..img_size
    {
        for x in 0..img_size
        {
            let idx = (y * img_size + x) as usize;
            let [c, m, yellow, k] = canvas[idx];
            let (r, g, b) = cmyk_to_rgb(c, m, yellow, k);
            output_img.put_pixel(x, y, Rgba([(r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, 255]));
        }
    }


    output_img.save(output_path).unwrap();

    std::io::stdout().flush().unwrap();
    println!("Total Time: {:02}:{:02}", beginning.elapsed().as_secs() / 60, beginning.elapsed().as_secs() % 60);
}


fn get_anitaliased_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Line // Returns a Line of antialiased calculation (just position and alpha value)
{
    let mut result = Vec::new();
    
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    let (x0, y0, x1, y1) = if steep { (y0, x0, y1, x1) } else { (x0, y0, x1, y1) };
    let (x0, y0, x1, y1) = if x0 > x1 { (x1, y1, x0, y0) } else { (x0, y0, x1, y1) };

    let dx = x1 - x0;
    let dy = y1 - y0;
    let gradient = if dx == 0.0 { 1.0 } else { dy / dx };

    let mut x = x0.round();
    let mut y = y0 + (x - x0) * gradient;

    while x <= x1 
    {
        let base_x = x as i32;
        let base_y = y.floor() as i32;
        let weight = y.fract();

        let mut push = |ix: i32, iy: i32, alpha: f32| 
        {
            if ix >= 0 && iy >= 0 
            {
                result.push((ix as u32, iy as u32, alpha));
            }
        };

        if steep 
        {
            push(base_y, base_x, 1.0 - weight);
            push(base_y + 1, base_x, weight);
        } 
        else 
        {
            push(base_x, base_y, 1.0 - weight);
            push(base_x, base_y + 1, weight);
        }

        x += 1.0;
        y += gradient;
    }

    result
}


#[inline(always)]
fn average_brightness(arr: &[f32], line: &[(u32, u32, f32)], size: u32) -> f32
{
    let mut sum = 0.0;
    for &(x, y, alpha) in line 
    {
        let i = (y * size + x) as usize;
        sum += (arr[i].abs()) * alpha;
    }
    sum / line.len() as f32
}


pub fn rgb_to_cmyk(r: f32, g: f32, b: f32) -> (f32, f32, f32, f32) // Cyan, magenta, yellow, key(black)
{
    let c = 1.0 - r;
    let m = 1.0 - g;
    let y = 1.0 - b;

    let k = c.min(m).min(y);

    if k >= 1.0 { return (0.0, 0.0, 0.0, 1.0); }

    let c = (c - k) / (1.0 - k);
    let m = (m - k) / (1.0 - k);
    let y = (y - k) / (1.0 - k);

    (c, m, y, k)
}

pub fn cmyk_to_rgb(c: f32, m: f32, y: f32, k: f32) -> (f32, f32, f32) 
{
    let r = (1.0 - c) * (1.0 - k);
    let g = (1.0 - m) * (1.0 - k);
    let b = (1.0 - y) * (1.0 - k);
    (r, g, b)
}