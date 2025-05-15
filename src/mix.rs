use crate::circle_image::generate_complex;
use crate::image_shader::image_shader;

pub fn run()
{
    let input = image::open("tests/koala.webp");
    if input.is_err()
    {
        println!("Error\n(Could be wrong path/does not exist");
        return;
    }
    let input = input.unwrap();
    let size = (input.width() as i32, input.height() as i32);

    let grid_size = size.0.min(size.1)/30/2;

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

    let circle_image = image::DynamicImage::ImageRgba8(generate_complex(100000, circles, grid_size, size, input, 30));

    let output = pollster::block_on(image_shader(circle_image, "src/shader/sobel_operator.wgsl"));
    output.save("tests/result.png").unwrap();
}