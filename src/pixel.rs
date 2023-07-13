pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub fn sum_channels(&self) -> usize {
        self.r as usize + self.g as usize + self.b as usize
    }

    pub fn brightness(&self) -> f64 {
        let bits_24_default_sum = 255.0 * 3.0;
        let result = self.sum_channels() as f64 / bits_24_default_sum;
        result
    }
}
