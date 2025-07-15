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

    // input_img = input_img.grayscale();

    input_img = input_img.crop_imm(0, 0, img_size, img_size); // Not Perfect, but fine

    let mut red: Vec<i16> = vec![0; (img_size * img_size) as usize];
    let mut green: Vec<i16> = vec![0; (img_size * img_size) as usize];
    let mut blue: Vec<i16> = vec![0; (img_size * img_size) as usize];
    for y in 0..img_size
    {
        for x in 0..img_size
        {
            let i = (y * img_size + x) as usize;
            let pixel = input_img.get_pixel(x as u32, y as u32).0;
            red[i] = pixel[0] as i16;
            green[i] = pixel[1] as i16;
            blue[i] = pixel[2] as i16;
        }
    }

    let mut red_error_sum: i32 = red.iter().map(|&v| v as i32).sum();
    let mut green_error_sum: i32 = green.iter().map(|&v| v as i32).sum();
    let mut blue_error_sum: i32 = blue.iter().map(|&v| v as i32).sum();

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


    //Red
    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Starting Pin
    let mut pin_num = rand::thread_rng().gen_range(0..pins.len());
    // let mut pin = pins[0];


    let start_time = Instant::now();
    let mut last_update = Instant::now();

    let mut last_i = 0;
    let max_iterations = 1000000;
    for i in 0..max_iterations
    {
        let (best, _) = (0..amount).into_par_iter().filter(|&pin_number| pin_number != pin_num).map(|pin_number| 
        {
            let line = &precomputed_lines[pin_num][pin_number];
            let score = penalty(&red, line, img_size) + penalty(&green, line, img_size) + penalty(&blue, line, img_size); //Average Brightness of all pixels in line
            (pin_number, score)
        }).reduce_with(|a, b| if a.1 < b.1 { a } else { b }).unwrap();

        let best_line = &precomputed_lines[pin_num][best];
        draw_line_average(&mut output_img, &mut red, &mut green, &mut blue, best_line, img_size, &mut red_error_sum, &mut green_error_sum, &mut blue_error_sum);

        pin_num = best;

        let avg_brightness = (red_error_sum as f32 + green_error_sum as f32 + blue_error_sum as f32) / (img_size*img_size) as f32;
        if avg_brightness < brightness_threshold*3.0
        {
            println!("{}", i);
            break;
        }


        if last_update.elapsed().as_secs() >= 1
        {
            let elapsed = last_update.elapsed();
            let iters = i - last_i;
            let time_per_iter = elapsed.as_secs_f32() / iters as f32;

            let remaining_iters_est = ((avg_brightness - brightness_threshold).max(0.01) / REMOVE as f32) * 2.5 * (img_size as f32);
            let progress = i as f32 / (i as f32 + remaining_iters_est as f32);

            let remaining_secs = remaining_iters_est * time_per_iter;

            let elapsed_total = start_time.elapsed();
            let remaining = Duration::from_secs_f32(remaining_secs);
    
            let elapsed_minutes = elapsed_total.as_secs() / 60;
            let elapsed_seconds = elapsed_total.as_secs() % 60;
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
            last_i = i;
        }
    }

    println!("\nOne Moment");
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
fn penalty(arr: &[i16], line: &[(u32, u32, f32)], size: u32) -> i32
{
    let mut sum = 0.0;
    for &(x, y, alpha) in line 
    {
        let i = (y * size + x) as usize;
        sum += (arr[i].abs() as f32) * alpha;
    }
    (sum / line.len() as f32) as i32
}

fn draw_line(img: &mut RgbaImage, red: &mut [i16], green: &mut [i16], blue: &mut [i16], line: &[(u32, u32, f32)], size: u32, 
    red_error_sum: &mut i32, green_error_sum: &mut i32, blue_error_sum: &mut i32)
{
    for &(x, y, alpha) in line 
    {
        let i = (y * size + x) as usize;

        // let remove_amount = (REMOVE as f32 * alpha) as i16;
        // red[i] += remove_amount;
        // *red_error_sum -= remove_amount as i32;
        // green[i] += remove_amount;
        // *green_error_sum -= remove_amount as i32;
        // blue[i] += remove_amount;
        // *blue_error_sum -= remove_amount as i32;

        let px = img.get_pixel_mut(x, y);

        // let base_red = red[i].max(0) as f32;
        // let base_green = green[i].max(0) as f32;
        // let base_blue = blue[i].max(0) as f32;
        let base_red = (255 - red[i].clamp(0, 255)) as f32;
        let base_green = (255 - green[i].clamp(0, 255)) as f32;
        let base_blue = (255 - blue[i].clamp(0, 255)) as f32;

        let total = base_red + base_green + base_blue + 1.0; // prevent division by zero

        let red_factor = base_red / total;
        let green_factor = base_green / total;
        let blue_factor = base_blue / total;

        red[i] += (REMOVE as f32 * alpha * red_factor) as i16;
        *red_error_sum -= (REMOVE as f32 * alpha * red_factor) as i32;

        green[i] += (REMOVE as f32 * alpha * green_factor) as i16;
        *green_error_sum -= (REMOVE as f32 * alpha * green_factor) as i32;
        
        blue[i] += (REMOVE as f32 * alpha * blue_factor) as i16;
        *blue_error_sum -= (REMOVE as f32 * alpha * blue_factor) as i32;

        px.0[0] = blend_channel(px.0[0], alpha * red_factor);
        px.0[1] = blend_channel(px.0[1], alpha * green_factor);
        px.0[2] = blend_channel(px.0[2], alpha * blue_factor);
    }
}


fn draw_line_average(img: &mut RgbaImage, red: &mut [i16], green: &mut [i16], blue: &mut [i16], line: &[(u32, u32, f32)], size: u32, 
    red_error_sum: &mut i32, green_error_sum: &mut i32, blue_error_sum: &mut i32)
{
    let mut total_red = 0.0;
    let mut total_green = 0.0;
    let mut total_blue = 0.0;
    let mut total_weight = 0.0;

    for &(x, y, alpha) in line 
    {
        let i = (y * size + x) as usize;

        let weight = alpha;

        total_red += (255 - red[i].clamp(0, 255)) as f32 * weight;
        total_green += (255 - green[i].clamp(0, 255)) as f32 * weight;
        total_blue += (255 - blue[i].clamp(0, 255)) as f32 * weight;

        total_weight += weight;
    }

    if total_weight == 0.0 
    {
        total_weight += 0.1
    }

    let avg_red = total_red / total_weight;
    let avg_green = total_green / total_weight;
    let avg_blue = total_blue / total_weight;

    let total = avg_red + avg_green + avg_blue + 1.0;

    let red_factor = avg_red / total;
    let green_factor = avg_green / total;
    let blue_factor = avg_blue / total;

    let red_inc = (REMOVE as f32 * red_factor) as i16;
    let green_inc = (REMOVE as f32 * green_factor) as i16;
    let blue_inc = (REMOVE as f32 * blue_factor) as i16;

    let red_err = red_inc as i32;
    let green_err = green_inc as i32;
    let blue_err = blue_inc as i32;

    for &(x, y, alpha) in line 
    {
        let i = (y * size + x) as usize;

        red[i] += red_inc;
        *red_error_sum -= red_err;

        green[i] += green_inc;
        *green_error_sum -= green_err;

        blue[i] += blue_inc;
        *blue_error_sum -= blue_err;

        let px = img.get_pixel_mut(x, y);

        px.0[0] = blend_channel(px.0[0], alpha * red_factor);
        px.0[1] = blend_channel(px.0[1], alpha * green_factor);
        px.0[2] = blend_channel(px.0[2], alpha * blue_factor);
    }
}


fn blend_channel(base: u8, alpha: f32) -> u8
{
    let darkening = alpha * (DRAW_OPACITY as f32/255.0);
    (base as f32 * (1.0 - darkening)).max(0.0) as u8
}



// fn draw_line_dark(img: &mut RgbaImage, red: &mut [i16], green: &mut [i16], blue: &mut [i16], line: &[(u32, u32, f32)], size: u32, 
//     red_error_sum: &mut i32, green_error_sum: &mut i32, blue_error_sum: &mut i32)
// {
//     for &(x, y, alpha) in line 
//     {
//         let i = (y * size + x) as usize;

//         let px = img.get_pixel_mut(x, y);

//         let base_red = (255 - red[i].clamp(0, 255)) as f32;
//         let base_green = (255 - green[i].clamp(0, 255)) as f32;
//         let base_blue = (255 - blue[i].clamp(0, 255)) as f32;

//         let total = base_red + base_green + base_blue + 1.0; // prevent division by zero

//         let red_factor = base_red / total;
//         let green_factor = base_green / total;
//         let blue_factor = base_blue / total;

//         red[i] += (REMOVE as f32 * alpha * red_factor) as i16;
//         *red_error_sum -= (REMOVE as f32 * alpha * red_factor) as i32;

//         green[i] += (REMOVE as f32 * alpha * green_factor) as i16;
//         *green_error_sum -= (REMOVE as f32 * alpha * green_factor) as i32;
        
//         blue[i] += (REMOVE as f32 * alpha * blue_factor) as i16;
//         *blue_error_sum -= (REMOVE as f32 * alpha * blue_factor) as i32;

//         px.0[0] = blend_channel_dark(px.0[0], alpha * red_factor);
//         px.0[1] = blend_channel_dark(px.0[1], alpha * green_factor);
//         px.0[2] = blend_channel_dark(px.0[2], alpha * blue_factor);
//     }
// }

// fn blend_channel_dark(base: u8, alpha: f32) -> u8 
// {
//     let brightening = alpha * (DRAW_OPACITY as f32 / 255.0);
//     ((base as f32 * (1.0 - brightening)) + 255.0 * brightening).min(255.0) as u8
// }