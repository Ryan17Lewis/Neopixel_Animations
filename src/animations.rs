use core::f32::consts::PI;
use micromath::F32Ext;

use smart_leds::RGB8;

pub const WIDTH : usize = 8;
pub const HEIGHT : usize = 8;
pub const NUM_PX : usize = WIDTH*HEIGHT;

// pulse implementation
pub struct nmPulse {
    strip: [RGB8; NUM_PX],
    color: RGB8,
    px_counter: u8,
    descending: bool,
    step_size: u8,
    brightness: u8,
}

impl nmPulse {
    // constructor fn
    pub fn new(brightness: u8, step_size: u8) -> nmPulse {
        Self {
            strip: [RGB8::new(0,0,0); NUM_PX],
            color: RGB8::new(brightness,brightness,brightness),
            px_counter: 0,
            descending: false,
            step_size: step_size,
            brightness: brightness,
        }
    }

    // set an LED color
    pub fn set(&mut self, color: RGB8) {
        for px in self.strip.iter_mut() {
            *px = color;
        }
    }

    pub fn to_list(&self) -> [RGB8; NUM_PX] {
        self.strip
    }

    pub fn next(&mut self) {
        if self.px_counter <= 1 {
            self.descending = false;
        } else if self.px_counter >= self.brightness {
            self.descending = true;
        }
        if self.descending == true {
            self.px_counter -= self.step_size;
        } else {
            self.px_counter += self.step_size;
        }

        self.set(RGB8::new(self.px_counter, self.px_counter, self.px_counter));
    }
}


// sprial implementation
pub struct nmSnake
{
    strip: [RGB8; NUM_PX],
    color: RGB8,
    delta: bool,
    row: usize,
    col: usize,
}

impl nmSnake {
    // constructor
    pub fn new(color: RGB8) -> nmSnake {
        Self {
            strip: [RGB8::new(0,0,0); NUM_PX],
            color: color,
            delta: true,
            row: 0,
            col: 0,
        }
    }

    // set pixels at (row,col)
    pub fn set(&mut self){
        for (idx, px) in self.strip.iter_mut().enumerate() {
            if idx == self.col*WIDTH + self.row {
                *px = self.color;
            } else {
                *px = RGB8::new(0,0,0);
            }
        }
    }

    pub fn to_list(&self) -> [RGB8; NUM_PX] {
        self.strip
    }

    pub fn next(&mut self) {
        // bounce the row value
        if self.row == WIDTH-1 {
            self.delta = false;
            self.col = (self.col + 1) % 8;
        } else if self.row == 0 {
            self.delta = true;
            self.col = (self.col + 1) % 8;
        }
        if self.delta { self.row += 1 } else { self.row -= 1 };
        // update
        self.set();
    }

}


// wave implementation
const NUM_SHADOWS: usize = 7;
pub struct nmWave
{
    strip: [RGB8; NUM_PX],
    color: RGB8,
    row: usize,
    shadows: [usize; NUM_SHADOWS],
}

impl nmWave {
    // constructor
    pub fn new(color: RGB8) -> nmWave {
        let mut shadows: [usize; NUM_SHADOWS] = [0; NUM_SHADOWS];
        for i in 0..NUM_SHADOWS {
            shadows[i] = NUM_SHADOWS - 1 - i;
        }

        Self {
            strip: [RGB8::new(0,0,0); NUM_PX],
            color: color,
            row: NUM_SHADOWS,
            shadows: shadows,
        }
    }

    // set row of pixels
    pub fn set(&mut self, row: usize, color: RGB8) {
        let mut col: usize = 0;
        for (idx, px) in self.strip.iter_mut().enumerate() {
            if idx == col*WIDTH + row {
                *px = color;
                col += 1;
            } 
        }
    }

    // clear all pixels
    pub fn clear(&mut self) {
        for px in self.strip.iter_mut() {
            *px = RGB8::new(0,0,0);
        }
    }

    pub fn to_list(&self) -> [RGB8; NUM_PX] {
        self.strip
    }

    pub fn next(&mut self) {
        // update row value
        self.row = (self.row +1) % WIDTH;

        let intensity_step: u8 = NUM_SHADOWS as u8;

        // clear rows
        self.clear();

        // draw original row
        self.set(self.row, self.color);

        // capture rgb
        let r: u8 = self.color.r;
        let g: u8 = self.color.g;
        let b: u8 = self.color.b;

        // draw shadow rows
        for i in 0..=(self.shadows.len()-1) {
            // update shadow row value
            self.shadows[i] = (self.shadows[i] + 1) % WIDTH;

            let dimmed_color = RGB8::new(
                r - r/intensity_step*((i+1) as u8) + 1,
                g - g/intensity_step*((i+1) as u8) + 1,
                b - b/intensity_step*((i+1) as u8) + 1,
            );

            self.set(self.shadows[i], dimmed_color);
        }
    }

}


// sin implementation
const SIN_SIZE: usize = 14*WIDTH/8;
pub struct nmSin
{
    strip: [RGB8; NUM_PX],
    color: RGB8,
    window: [usize; WIDTH],
    sin: [usize; SIN_SIZE],
}

impl nmSin {
    // constructor
    pub fn new(color: RGB8) -> nmSin {
        // init sin pattern
        let amplitude: f32 = (HEIGHT) as f32/2.0;
        let offset: f32 = (HEIGHT) as f32/2.0;

        let mut sin: [usize; SIN_SIZE]  = [0; SIN_SIZE];
        for i in 0..SIN_SIZE {
            let value: usize = (amplitude * (f32::sin(i as f32 * (2.0 * PI / (SIN_SIZE) as f32))) + offset).round() as usize;
            sin[i] = value;
        }

        // start window as front of sin wave
        let mut window: [usize; WIDTH]  = [0; WIDTH];
        for i in 0..WIDTH {
            window[i] = sin[i];
        }

        Self {
            strip: [RGB8::new(0,0,0); NUM_PX],
            color: color,
            window: window,
            sin: sin,
        }
    }

    // set row of pixels up to height
    pub fn set_row_height(&mut self, row: usize, height: usize) {
        let mut col: usize = 0;
        for (idx, px) in self.strip.iter_mut().enumerate() {
            if idx == col*WIDTH + row {
                *px = self.color;
                if col < height {
                    col += 1;
                }
            } 
        }
    }

    // clear all pixels
    pub fn clear(&mut self) {
        for px in self.strip.iter_mut() {
            *px = RGB8::new(0,0,0);
        }
    }

    pub fn to_list(&self) -> [RGB8; NUM_PX] {
        self.strip
    }

    pub fn next(&mut self) {
        // draw frame/row
        self.clear();
        for i in 0..WIDTH {
            // ignore 0
            if self.window[i] != 0 {
                self.set_row_height(i, self.window[i]-1);
            }
        }

        // update sin
        for i in 0..SIN_SIZE {
            self.sin[i] = self.sin[(i+1) % (SIN_SIZE)];
        }

        // update window
        for i in 0..WIDTH {
            self.window[i] = self.sin[i];
        }
    }

}