use std::f32::consts::PI;
use std::io;
use std::io::Write;
use rand::Rng;
use std::time::{Duration, Instant};

use image::{GenericImage, Rgb, Rgba, RgbaImage};
use image::GenericImageView;

use rayon::prelude::*;

pub static LOOPS: i32 = 6000; //6000
pub static DRAW_OPACITY: u8 = 130; //130
pub static REMOVE: i16 = 130; //50

pub const PATH: &str = "peppe_sad.png";

fn draw_line_2(img: &mut RgbaImage, x0: i64, y0: i64, x1: i64, y1: i64, arr: &mut Vec<Vec<i16>>, weight: &mut Vec<Vec<f32>>)// -> Vec<Vec<u8>>//Draws white
{
    // Create local variables for moving start point
    let mut x0 = x0;
    let mut y0 = y0;

    // Get absolute x/y offset
    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

    // Get slopes
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    // Initialize error
    let mut err = if dx > dy { dx } else {-dy} / 2;
    let mut err2;

    loop 
    {
        // Set pixel
        // let color = [(((255 - DRAW_OPACITY as i32 as i32) * 255 + DRAW_OPACITY as i32 * 0) / 255) as u8, (((255 - DRAW_OPACITY as i32) * 255 + DRAW_OPACITY as i32 * 0) / 255) as u8, (((255 - DRAW_OPACITY as i32) * 255 + DRAW_OPACITY as i32 * 0) / 255) as u8, 255 as u8];
        img.get_pixel_mut(x0 as u32, y0 as u32).0 = [0, 0, 0, DRAW_OPACITY]; //DRAW_OPACITY
        arr[y0 as usize][x0 as usize] += REMOVE;
        // weight[y0 as usize][x0 as usize] -= REMOVE as f32/255.0;


        // Check end condition
        if x0 == x1 && y0 == y1 { break };

        // Store old error
        err2 = 2 * err;

        // Adjust error and start position
        if err2 > -dx { err -= dy; x0 += sx; }
        if err2 < dy { err += dx; y0 += sy; }
    }

}

fn penalty(x0: i64, y0: i64, x1: i64, y1: i64, arr: &Vec<Vec<i16>>, weight: &Vec<Vec<f32>>) -> i32
{
    let mut sum_brightness = 0;
    let mut num_pixels = 0;
    
    let mut x0 = x0;
    let mut y0 = y0;

    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = if dx > dy { dx } else {-dy} / 2;
    let mut err2;

    loop 
    {
        let pixel_value = arr[y0 as usize][x0 as usize];
        sum_brightness += (pixel_value as i32).abs();
        num_pixels += 1;

        if x0 == x1 && y0 == y1 { break };

        err2 = 2 * err;

        if err2 > -dx { err -= dy; x0 += sx; }
        if err2 < dy { err += dx; y0 += sy; }
    }

    (sum_brightness - (REMOVE as i32 * num_pixels)) / num_pixels
}

pub fn run() 
{
    let beginning = Instant::now();
    println!("Preparing Image...");
    let mut input_img = image::open("tests/".to_string() + PATH).unwrap();

    let width = input_img.width() as f32;
    let height = input_img.height() as f32;

    let radius = (width/2.0).min(height/2.0);
    let img_size = radius as u32 * 2;

    input_img = input_img.grayscale();
    // for y in 0..input_img.height()
    // {
    //     for x in 0..input_img.width()
    //     {
    //         let color = input_img.get_pixel(x, y);
    //         input_img.put_pixel(x, y, Rgba([0, color.0[1], 0, 255]));
    //     }
    // }
    input_img.save("res/test.png").unwrap();

    input_img = input_img.crop_imm(0, 0, img_size, img_size);

    let mut pixel_value: Vec<Vec<i16>> = vec![vec![0; input_img.width() as usize]; input_img.height() as usize];
    // for (x, y, pixel) in input_img.pixels()
    // {
    //     pixel_value[y as usize][x as usize] = pixel.0[0] as i16;
    // }
    pixel_value.par_iter_mut().enumerate().for_each(|(y, row)| //Faster with threading
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
    
    let amount = pins.len();

    let mut output_img = RgbaImage::new(img_size, img_size);

    for y in 0..output_img.height()
    {
        for x in 0..output_img.width()
        {
            output_img.put_pixel(x, y, Rgba([200, 200, 200, 255]));
        }
    }
    // output_img.enumerate_pixels_mut().par_bridge().for_each(|(_, _, pixel)| 
    // {
    //     *pixel = Rgba([200, 200, 200, 255]);
    // });

    let mut pin_num = rand::thread_rng().gen_range(0..pins.len());
    let mut pin = pins[0];



    let start_time = Instant::now();
    let mut last_update = Instant::now();

    for i in 0..LOOPS
    {
        let pin_nums: Vec<usize> = (0..amount).filter(|&pin| pin != pin_num).collect();

        let mut darkest = i32::MAX;
        let mut num = 0;
        for i in 0..pin_nums.len()
        {
            let current_pin = pins[pin_nums[i]];
            let brightness = penalty(pin.0 as i64, pin.1 as i64, current_pin.0 as i64, current_pin.1 as i64, &pixel_value, &weight);
            if brightness < darkest
            {
                darkest = brightness;
                num = i;
            }
        }

        let current_pin = pins[pin_nums[num]];
        draw_line_2(&mut output_img, pin.0 as i64, pin.1 as i64, current_pin.0 as i64, current_pin.1 as i64, &mut pixel_value, &mut weight);
        pin_num = num;
        pin = current_pin;


        if last_update.elapsed().as_secs() >= 1
        {
            let progress = i as f32 / LOOPS as f32;
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
            io::stdout().flush().unwrap();

            last_update = Instant::now();
        }
    }
    print!("\nOne Moment...");
    output_img.save("res/result.png").unwrap();
    io::stdout().flush().unwrap();
    println!("\rDone         ");
    println!("Total Time: {:02}:{:02}", beginning.elapsed().as_secs() / 60, beginning.elapsed().as_secs() % 60);
}