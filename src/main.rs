use std::f32::consts::PI;
use std::fs;
use std::io::Write;
use rand::Rng;

use image::{RgbaImage, Rgba, GenericImage, ImageBuffer};
use image::GenericImageView;

pub static LOOPS: i32 = 750;
pub static DRAW_OPACITY: u8 = 255;
pub static REMOVE: i16 = 50;

pub const PATH: &str = "number";


fn draw_generic_line<T: GenericImage>(img: &mut T, x0: i64, y0: i64, x1: i64, y1: i64, pixel: T::Pixel)//Pixel just color
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

    loop {
        // Set pixel
        img.put_pixel(x0 as u32, y0 as u32, pixel);

        // Check end condition
        if x0 == x1 && y0 == y1 { break };

        // Store old error
        err2 = 2 * err;

        // Adjust error and start position
        if err2 > -dx { err -= dy; x0 += sx; }
        if err2 < dy { err += dx; y0 += sy; }
    }
}

fn calulate_brightness_line(x0: i64, y0: i64, x1: i64, y1: i64, arr: &Vec<Vec<i16>>) -> i32
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

    loop {
        // Set pixel
        // img.put_pixel(x0 as u32, y0 as u32, pixel);
        let pixel_value = arr[y0 as usize][x0 as usize];
        sum_brightness += pixel_value as i32;
        num_pixels += 1;

        if x0 == x1 && y0 == y1 { break };

        err2 = 2 * err;

        if err2 > -dx { err -= dy; x0 += sx; }
        if err2 < dy { err += dx; y0 += sy; }
    }

    sum_brightness / num_pixels
}

fn draw_line(img: &mut RgbaImage, x0: i64, y0: i64, x1: i64, y1: i64, color: [u8; 4]) //Draws white
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
        img.get_pixel_mut(x0 as u32, y0 as u32).0 = color;

        // Check end condition
        if x0 == x1 && y0 == y1 { break };

        // Store old error
        err2 = 2 * err;

        // Adjust error and start position
        if err2 > -dx { err -= dy; x0 += sx; }
        if err2 < dy { err += dx; y0 += sy; }
    }

}

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
        img.get_pixel_mut(x0 as u32, y0 as u32).0 = [0, 0, 0, DRAW_OPACITY];
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


fn main() 
{
    // let mut input_img = image::open("res/cat_face.png").unwrap();

    // println!("dimensions {:?}", input_img.dimensions());
    // println!("{:?}", input_img.color());

    // input_img = input_img.grayscale();


    // let mut arr: Vec<Vec<(u8, u8, u8, u8)>> = vec![vec![(0, 0, 0, 0); input_img.width() as usize]; input_img.height() as usize];
    // for (x, y, pixel) in input_img.pixels()
    // {
    //     arr[y as usize][x as usize] = (pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
    // }

    // for t in input_img.pixels()
    // {

    // }

    // input_img.save("res/cat_face-grayscale.png").unwrap();


    // let mut output_img = RgbaImage::new(input_img.width(), input_img.height());

    // for (x, y, pixel) in input_img.pixels()
    // {
    //     output_img.put_pixel(x, y, image::Rgba([arr[y as usize][x as usize].0, arr[y as usize][x as usize].1, arr[y as usize][x as usize].2, arr[y as usize][x as usize].3]));
    // }

    // draw_generic_line(&mut output_img, 0, 0, 359, 449, image::Rgba([0, 0, 255, 255]));

    // output_img.save("res/result2.png").unwrap();

    // let comb_img = image::imageops::overlay(&mut input_img, &output_img, 0, 0);

    // input_img.save("res/result.png").unwrap();



    let mut input_img = image::open("res/".to_string() + PATH + ".png").unwrap();

    let width = input_img.width() as f32;
    let height = input_img.height() as f32;

    let radius = (width/2.0).min(height/2.0);
    let img_size = radius as u32 * 2;

    input_img = input_img.grayscale();

    input_img = input_img.crop_imm(0, 0, img_size, img_size);

    let mut pixel_value: Vec<Vec<i16>> = vec![vec![0; input_img.width() as usize]; input_img.height() as usize];
    for (x, y, pixel) in input_img.pixels()
    {
        pixel_value[y as usize][x as usize] = pixel.0[0] as i16;
    }

    let mut weight: Vec<Vec<f32>> = vec![vec![0.0; input_img.width() as usize]; input_img.height() as usize];
    for (x, y, pixel) in input_img.pixels()
    {
        weight[y as usize][x as usize] = (1.0 - ((pixel.0[0] as f32)/255.0)).abs();
    }

    let mut test_img = RgbaImage::new(img_size, img_size);
    let mut test_file = fs::File::create("res/test_file.txt").unwrap();

    for y in 0..test_img.height()
    {
        for x in 0..test_img.width()
        {
            let pixel = (weight[y as usize][x as usize] * 255.0) as u8;
            test_img.put_pixel(x, y, Rgba([pixel, pixel, pixel, 255]));
            let b = pixel.to_string() + " ";
            if x % 10 == 0
            {
                test_file.write(b.as_bytes()).unwrap();
            }
        }
        // if y % 10 == 0
        {
            test_file.write(b"\n").unwrap();
        }
    }

    test_img.save("res/test.png").unwrap();

    // for y in 0..arr.len()/10
    // {
    //     for x in 0..arr[0].len()/10
    //     {
    //         let t = arr[y*10][x*10];
    //         if t < 10
    //         {
    //             print!(" {}  ", t);
    //         }
    //         else if t < 100
    //         {
    //             print!("{}  ", t);
    //         }
    //         else 
    //         {
    //             print!("{} ", t);
    //         }
    //     }
    //     println!();
    // }

    {
        let path = "res/".to_string() + PATH + "-grayscale.png";
        input_img.save(&path).unwrap();
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

    // let mut test_image = RgbaImage::new(img_size, img_size);

    // for y in 0..test_image.height()
    // {
    //     for x in 0..test_image.width()
    //     {
    //         test_image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
    //     }
    // }

    // for y in 0..test_image.height()
    // {
    //     for x in 0..test_image.width()
    //     {
    //         if pins.contains(&(x, y))
    //         {
    //             for yy in (y as isize - 3)..(y as isize + 3) 
    //             {
    //                 for xx in (x as isize - 3)..(x as isize + 3)
    //                 {
    //                     if xx >= 0 && xx < test_image.width() as isize && yy >= 0 && yy < test_image.height() as isize
    //                     {
    //                         test_image.put_pixel(xx as u32, yy as u32, Rgba([255, 255, 255, 255]));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    // test_image.save("res/circle_result.png").unwrap();

    let mut pin_nums = Vec::new();
    let pin_num = rand::thread_rng().gen_range(0..pins.len());
    for i in 0..100
    {
        if i == pins.len()-1 { continue; }
        let mut temp_pin_num = rand::thread_rng().gen_range(0..pins.len());
        while temp_pin_num == pin_num || pin_nums.contains(&temp_pin_num)
        {
            temp_pin_num = rand::thread_rng().gen_range(0..pins.len());
        }
        pin_nums.push(temp_pin_num);
        // println!("First Loop: {}", i);
    }

    // for i in 0..pin_nums.len()
    // {
    //     draw_line(&mut test_image, pins[0].0 as i64, pins[0].1 as i64, pins[pin_nums[i]].0 as i64, pins[pin_nums[i]].1 as i64, [255, 0, 0, 255]);
    //     let path = String::from("res/circleLines/") + &i.to_string() + "circleLine.png";
    //     test_image.save(path).unwrap();
    // }
    // test_image.save("res/circleLinesResult.png").unwrap();

    let amount = pins.len();

    let mut output_img = RgbaImage::new(img_size, img_size);

    for y in 0..output_img.height()
    {
        for x in 0..output_img.width()
        {
            output_img.put_pixel(x, y, Rgba([200, 200, 200, 255]));
        }
    }

    
    
    let mut pin_num = rand::thread_rng().gen_range(0..pins.len());
    let mut pin = pins[0];

    for i in 0..LOOPS
    {
        let mut pin_nums = Vec::new();
        for i in 0..amount
        {
            if i == pins.len()-1 { continue; }
            let mut temp_pin_num = rand::thread_rng().gen_range(0..pins.len());
            while temp_pin_num == pin_num || pin_nums.contains(&temp_pin_num)
            {
                temp_pin_num = rand::thread_rng().gen_range(0..pins.len());
            }
            pin_nums.push(temp_pin_num);
            // println!("First Loop: {}", i);
        }

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
        // println!("{}", darkest);

        let current_pin = pins[pin_nums[num]];
        draw_line_2(&mut output_img, pin.0 as i64, pin.1 as i64, current_pin.0 as i64, current_pin.1 as i64, &mut pixel_value, &mut weight);
        pin_num = num;
        pin = current_pin;
        // println!("{}", i);
    }

    // for y in 0..arr.len()/10
    // {
    //     for x in 0..arr[0].len()/10
    //     {
    //         let t = arr[y*10][x*10];
    //         if t < 10
    //         {
    //             print!(" {}  ", t);
    //         }
    //         else if t < 100
    //         {
    //             print!("{}  ", t);
    //         }
    //         else 
    //         {
    //             print!("{} ", t);
    //         }
    //     }
    //     println!();
    // }

    output_img.save("res/result.png").unwrap();
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


    // let mut sum_brightness = 0.0;
    // let mut num_pixels = 0;
    // let mut sum_weight = 0.0;
    // let mut total_weight = 0.0;
    
    // let mut x0 = x0;
    // let mut y0 = y0;

    // let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    // let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

    // let sx = if x0 < x1 { 1 } else { -1 };
    // let sy = if y0 < y1 { 1 } else { -1 };

    // let mut err = if dx > dy { dx } else {-dy} / 2;
    // let mut err2;

    // loop 
    // {
    //     let pixel_value = arr[y0 as usize][x0 as usize];
    //     let pixel_weight = weight[y0 as usize][x0 as usize];
    //     sum_brightness += pixel_value as f32 * pixel_weight;
    //     total_weight += pixel_weight;

    //     if x0 == x1 && y0 == y1 { break };

    //     err2 = 2 * err;

    //     if err2 > -dx { err -= dy; x0 += sx; }
    //     if err2 < dy { err += dx; y0 += sy; }
    // }

    // // (((sum_brightness - (REMOVE as i32 * num_pixels)) as f32 * sum_weight) / num_pixels as f32)

    // // (sum_brightness - (REMOVE as i32 * num_pixels)) / num_pixels
    // if total_weight > 0.0
    // {
    //     sum_brightness / total_weight
    // }
    // else
    // {
    //     0.0
    // }
}