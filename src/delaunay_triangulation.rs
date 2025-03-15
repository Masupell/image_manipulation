use image::DynamicImage;

pub fn run(img_path: &str)
{
    let input_img = image::open(img_path).unwrap();


}

fn gaussian_blur(img: &DynamicImage, sigma: f32)// -> DynamicImage
{
    let kernel_size = 3;
    let mut init = Vec::new();

    for y in (-kernel_size / 2)..=(kernel_size / 2)
    {
        let mut row = Vec::new();
        for x in (-kernel_size / 2)..=(kernel_size / 2)
        {
            row.push((x, y));
        }
        init.push(row);
    }

    let kernel = calculate_kernel(&mut init, sigma);
}

fn calculate_kernel(init: &Vec<Vec<(i32, i32)>>, sigma: f32) -> Vec<Vec<f32>>
{
    let pi:f32 = 3.141592653;
    let e: f32 = 2.718281828;
    let mut kernel: Vec<Vec<f32>> = Vec::new();
    for kernel_y in 0..init.len()
    {
        let mut row = Vec::new();
        for kernel_x in 0..init[kernel_y].len()
        {
            let x = init[kernel_y][kernel_x].0 as f32;
            let y = init[kernel_y][kernel_x].1 as f32;
            
            let step_one = -(x*x+y*y)/(2.0*sigma*sigma);
            let step_two = e.powf(step_one);
            let step_three = 1.0/(2.0*pi*sigma*sigma);
            row.push(step_two*step_three);
        }
        kernel.push(row);
    }

    let sum = kernel.iter().flatten().sum::<f32>();

    for value in kernel.iter_mut().flatten()
    {
        *value /= sum;
    }

    kernel
}



fn test(var: i32)
{
    let kernel_size = var;
    let mut init = Vec::new();

    for y in (-kernel_size / 2)..=(kernel_size / 2)
    {
        let mut row = Vec::new();
        for x in (-kernel_size / 2)..=(kernel_size / 2)
        {
            row.push((x, y));
        }
        init.push(row);
    }
    // println!("{:?}", init);
    let kernel = calculate_kernel(&mut init, 1.0);
    println!("{:?}", kernel);
}

#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn it_works() 
    {
        let x = 3;
        test(x);
        // println!("{:?}",test(x));
    }// cargo test -- --nocapture
}