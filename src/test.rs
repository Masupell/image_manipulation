#[cfg(test)]
mod tests
{
    // use super::super::read_video::read_video;
    // use super::read_video::read;
    // use super::super::sobel_operator::sobel_gpu;

    use super::super::read_video::test;

    #[test]
    pub fn run()
    {
        // read_video();
        // sobel_gpu();
        test();
    }
}