use rand::Rng;
use std::f32::consts::PI;
use std::i32;
use std::io::Write;
use std::time::{Duration, Instant};

use image::{Pixel, Rgba, RgbaImage};
use image::GenericImageView;
use rayon::prelude::*;

type Line = Vec<(u32, u32, f32)>; //x, y, blend_alpha

pub static DRAW_OPACITY: u8 = 130; //130
pub static REMOVE: i16 = 50; //50

pub fn run(path: &str)
{
    let beginning = Instant::now();
    println!("Preparing Image...");
    let mut input_img = image::open(path).unwrap();

    let width = input_img.width() as f32;
    let height = input_img.height() as f32;

    let radius = (width/2.0).min(height/2.0);
    let img_size = radius as u32 * 2;

    input_img = input_img.grayscale();

    input_img = input_img.crop_imm(0, 0, img_size, img_size); // Not Perfect, but fine

    let mut pixel_value: Vec<Vec<i16>> = vec![vec![0; input_img.width() as usize]; input_img.height() as usize];
    pixel_value.par_iter_mut().enumerate().for_each(|(y, row)| //Faster with threading, just saving the image pixels to this array
    {
        row.par_iter_mut().enumerate().for_each(|(x, val)| 
        {
            *val = input_img.get_pixel(x as u32, y as u32).0[0] as i16;
        });
    });

    let mut weight: Vec<Vec<f32>> = vec![vec![0.0; input_img.width() as usize]; input_img.height() as usize];
    for (x, y, pixel) in input_img.pixels()
    {
        weight[y as usize][x as usize] = (1.0 - ((pixel.0[0] as f32)/255.0)).abs();
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
            output_img.put_pixel(x, y, Rgba([200, 200, 200, 255]));
        }
    }


    //Precompute lines (to not calculate lines during loop, faster)
    let mut precomputed_lines: Vec<Vec<Line>> = vec![vec![Vec::new(); amount]; amount];
    for x in 0..amount
    {
        for y in 0..amount
        {
            if x == y { continue; }
            let (x0, x1) = pins[x];
            let (y0, y1) = pins[y];
            precomputed_lines[x][y] = get_anitaliased_line(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        }
    }

    // Starting Pin
    let mut pin_num = rand::thread_rng().gen_range(0..pins.len());
    // let mut pin = pins[0];


    let start_time = Instant::now();
    let mut last_update = Instant::now();

    for i in 0..6000
    {
        let mut darkest = i32::MAX;
        let mut best = 0;
        for pin_number in 0..amount
        {
            if pin_number == pin_num { continue; }
            
            let line = &precomputed_lines[pin_num][pin_number];
            let score = penalty(&pixel_value, line); //Average Brightness of all pixels in line

            if score < darkest
            {
                darkest = score;
                best = pin_number;
            }
        }

        let best_line = &precomputed_lines[pin_num][best];
        draw_line(&mut output_img, &mut pixel_value, best_line);

        pin_num = best;




        if last_update.elapsed().as_secs() >= 1
        {
            let progress = i as f32 / 6000 as f32;
            let elapsed = start_time.elapsed();
            let remaining = if progress > 0.0
            {
                Duration::from_secs_f32(elapsed.as_secs_f32() * (1.0 - progress) / progress)
            }
            else
            {
                Duration::ZERO
            };
    
            let elapsed_minutes = elapsed.as_secs() / 60;
            let elapsed_seconds = elapsed.as_secs() % 60;
            let remaining_minutes = remaining.as_secs() / 60;
            let remaining_seconds = remaining.as_secs() % 60;
    
            print!(
                "\rProgress: {:.0}% | Elapsed: {:02}:{:02} | Remaining: {:02}:{:02}",
                progress * 100.0,
                elapsed_minutes, elapsed_seconds,
                remaining_minutes, remaining_seconds
            );
            std::io::stdout().flush().unwrap();

            last_update = Instant::now();
        }
    }

    output_img.save("tests/result02.png").unwrap();

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


fn penalty(arr: &Vec<Vec<i16>>, line: &[(u32, u32, f32)]) -> i32
{
    let mut sum = 0.0;
    for &(x, y, alpha) in line 
    {
        sum += (arr[y as usize][x as usize].abs() as f32) * alpha;
    }
    (sum / line.len() as f32) as i32
}

fn draw_line(img: &mut RgbaImage, arr: &mut Vec<Vec<i16>>, line: &[(u32, u32, f32)])
{
    for &(x, y, alpha) in line 
    {
        let blend = Rgba([0, 0, 0, (DRAW_OPACITY as f32 * alpha) as u8]);
        img.get_pixel_mut(x, y).blend(&blend);
        arr[y as usize][x as usize] += (REMOVE as f32 * alpha) as i16;
    }
}