use std::time::{Duration, Instant};
use std::io::{self, Write};
use ffmpeg_next::format;
use image::{DynamicImage, GenericImageView, RgbaImage};
use rand::prelude::*;

macro_rules! parse
{
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

pub fn run() 
{
    let mut input = String::new();
    println!("1: Image with overlapping Circles\n2: Image without overlapping\n");

    loop 
    {
        io::stdin().read_line(&mut input).unwrap();
        if "abcdefghijklmnopqrstuvwxyzäöü34567890 |;:,<>\"{}[]()!#@$%^&*?-_+=ß/".to_string().contains(&input.trim().to_lowercase())
        {
            println!("Only 1 or 2... Try Again!");
        }
        else
        {
            let test: Vec<char> = input.trim().chars().collect();
            if test.len() > 1
            {
                println!("Only one Number... Try Again!");
            }
            else 
            {
                let s = further(test[0]);
                if s { return; }
            }
        }
        input.clear();
    }

    // image_complex(&input.trim(), "/res", 30);
}


fn further(c: char) -> bool
{
    println!();
    let mut input = String::new();

    println!("path of input image");
    io::stdin().read_line(&mut input).unwrap();
    let path = input.trim().to_string();
    input.clear();
    println!("path of output image (folder)");
    io::stdin().read_line(&mut input).unwrap();
    let output_path = input.trim().to_string();
    input.clear();

    if c == '1'
    {
        image_one(&path, &output_path);
    }
    else 
    {
        println!("Radius of circles (default: 30)");
        io::stdin().read_line(&mut input).unwrap();
        let temp_r = input.trim().to_string();
        let radius;
        if temp_r.is_empty()
        {
            radius = 30;
        }
        else 
        {
            if "abcdefghijklmnopqrstuvwxyzäöü |;:,<>\"{}[]()!#@$%^&*?-_+=ß/".to_string().contains(&temp_r.to_lowercase())
            {
                println!("Only Numbers");
                return true;
            }
            radius = parse!(temp_r, i32);
        }
        image_complex(&path, &output_path, radius);
    }

    true
}


pub fn image_complex(name: &str, output_path: &str, mut max_radius: i32)
{
    let name = name;
    if max_radius < 10 { max_radius = 10;}

    let input = image::open(name);
    if input.is_err()
    {
        println!("Error\n(Could be wrong path/does not exist");
        return;
    }
    let input = input.unwrap();
    let size = (input.width() as i32, input.height() as i32);

    let grid_size = size.0.min(size.1)/max_radius/2;

    let circles: Vec<Vec<Vec<((i32, i32), i32)>>> = vec![vec![vec![]; 10]; 10];
    /*
        (Actually 3D-Vector, but to understand I wil say something else)
        2D-Vector, with each storing position (x, y) and radius f multiple circles
                   0                  1                 2 
        [
            [c1, c2, c3, ...] [c1, c2, c3, ...] [c1, c2, c3, ...]    0
            [c1, c2, c3, ...] [c1, c2, c3, ...] [c1, c2, c3, ...]    1
            [c1, c2, c3, ...] [c1, c2, c3, ...] [c1, c2, c3, ...]    2
        ]
    */

    generate_complex(100000, circles, grid_size, size, input, max_radius).save(format!("{}/result.png", output_path)).unwrap();
}

pub fn generate_complex(iterations: i32, mut circles: Vec<Vec<Vec<((i32, i32), i32)>>>, grid_size: i32, size: (i32, i32), input: DynamicImage, max_radius: i32) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
{
    let mut rng = thread_rng();
    let mut output = RgbaImage::new(size.0 as u32, size.1 as u32);


    let x = rng.gen_range(0..size.0);
    let y = rng.gen_range(0..size.1);
    let r = 1;
    circles[(x/size.0*grid_size) as usize][(y/size.1*grid_size) as usize].push(((x, y), r));

    let start_time = Instant::now();
    let mut last_update = Instant::now();

    for count in 0..iterations
    {
        let x = rng.gen_range(0..size.0);
        let y = rng.gen_range(0..size.1);
        let mut radius = max_radius;

        let i = 1.max(x/size.0*grid_size).min(grid_size-2);
        let j = 1.max(y/size.1*grid_size).min(grid_size-2);

        for dy in [-1, 0, 1]
        {
            for dx in [-1, 0, 1]
            {
                for c in &circles[(j+dy) as usize][(i+dx) as usize]
                {
                    radius = radius.min((((c.0.0 as f32 - x as f32).powf(2.0)+(c.0.1 as f32 - y as f32).powf(2.0)).sqrt() - c.1 as f32) as i32);
                }
            }
        }

        if radius < 0
        {
            continue;
        }
        else 
        {
            circles[(x/size.0*grid_size) as usize][(y/size.1*grid_size) as usize].push(((x, y), radius));

            for yy in 0..size.1
            {
                if yy > y + radius
                {
                    break;
                }
    
                if y - radius < 0 || y + radius > size.1
                {
                    continue;
                }
    
                for xx in 0..size.0
                {
                    if xx > x + radius
                    {
                        break;
                    }
                    
                    if x - radius < 0 || x + radius > size.0
                    {
                        continue;
                    }
    
                    let distance = (xx-x).pow(2) + (yy-y).pow(2);
    
                    if distance < radius.pow(2)
                    {
                        let pixel_color = input.get_pixel(x as u32, y as u32);
                        output.put_pixel(xx as u32, yy as u32, pixel_color);
                    }
                }
            }
        }

        if last_update.elapsed().as_secs() >= 1
        {
            let progress = count as f32 / iterations as f32;// i
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

    output
}




pub fn image_one(name: &str, output_path: &str)
{
    let name = name;

    let func = |x: f32| x.powf(0.4);

    let input = image::open(name);
    if input.is_err()
    {
        println!("Error\n(Kann sein, dass du einen falschen Pfad angegeben hast (existiert nicht/falsche Endung)");
        return;
    }
    let input = input.unwrap();
    let size = (input.width() as i32, input.height() as i32);

    generate_simple(97500, func, size, input, output_path);
}

fn generate_simple(iterations: i32, func: impl Fn(f32) -> f32, size: (i32, i32), input: DynamicImage, output_path: &str)
{
    let mut rng = thread_rng();
    let mut output = RgbaImage::new(size.0 as u32, size.1 as u32);

    let start_time = Instant::now();
    
    for i in 0..iterations
    {  
        let bigger = (1.0_f32.max(30.0 - func(i as f32))).max(1.0_f32.max(100.0 - func(i as f32)));
        let smaller = (1.0_f32.max(30.0 - func(i as f32))).min(1.0_f32.max(100.0 - func(i as f32)));
        let radius = rng.gen_range(smaller..bigger) as i32;
        let r = radius*radius;
        let x = rng.gen_range(0..size.0);
        let y = rng.gen_range(0..size.1);



        for yy in 0..size.1
        {
            if yy > y + radius
            {
                break;
            }

            if y - radius < 0 || y + radius > size.1
            {
                continue;
            }

            for xx in 0..size.0
            {
                if xx > x + radius
                {
                    break;
                }
                
                if x - radius < 0 || x + radius > size.0
                {
                    continue;
                }

                let distance = (xx-x).pow(2) + (yy-y).pow(2);

                if distance < r
                {
                    let pixel_color = input.get_pixel(x as u32, y as u32);
                    output.put_pixel(xx as u32, yy as u32, pixel_color);
                }
            }
        }

        let progress = i as f32 / iterations as f32;
        let elapsed = start_time.elapsed();
        let remaining = if progress > 0.0
        {
            Duration::from_secs_f32(elapsed.as_secs_f32() * (1.0 - progress) / progress)//elapsed.as_secs_f32() / progress - elapsed.as_secs_f32();
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
    }
    _ = output.save(format!("{}\\result.png", output_path));
}