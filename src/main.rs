#![no_std]
#![no_main]

/**** low-level imports *****/
use panic_halt as _;
// use cortex_m::prelude::*;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

/***** board-specific imports *****/
use adafruit_feather_rp2040::hal::{self as hal, fugit::RateExtU32, pio::PIOExt};
use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    watchdog::Watchdog,
    Sio,
    i2c::I2C,
    Timer,
};
use adafruit_feather_rp2040::{
    Pins, XOSC_CRYSTAL_FREQ,
};

/***** peripheral-specific imports *****/
use lis3dh::{Lis3dh, SlaveAddr};
use accelerometer::{Accelerometer, vector::F32x3};
use ws2812_pio::Ws2812;

/***** custom module imports *****/
mod animations;
use animations::{nmPulse, nmSnake, nmWave, nmSin};
use smart_leds::{RGB8, SmartLedsWrite};


#[entry]
fn main() -> ! {
    // Grab the singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    // Init the watchdog timer, to pass into the clock init
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // Create timers
    let mut delay_timer = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // initialize the Single Cycle IO
    let sio = Sio::new(pac.SIO);
    // initialize the pins to default state
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    
    // Initialize I2C bus
    let sda_pin = pins.sda.into_function::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.scl.into_function::<hal::gpio::FunctionI2C>();    
    let i2c = I2C::i2c1(
        pac.I2C1,
        sda_pin,
        scl_pin,
        400_000.Hz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    // Initialize LIS3DH
    let mut lis3dh = Lis3dh::new_i2c(i2c, SlaveAddr::Default).unwrap();
    lis3dh.set_range(lis3dh::Range::G2).unwrap(); // Set the accelerometer range
    lis3dh.set_datarate(lis3dh::DataRate::Hz_100).unwrap(); // Set the data rate

    // instantiate ws2812
    let (mut pio0, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut neopixels = Ws2812::new(
        pins.d5.into_function(),
        &mut pio0,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    // enable neopixel matrix
    let mut neopixel_en_pin = pins.d10.into_push_pull_output();
    neopixel_en_pin.set_high().unwrap();


    // create animations
    let mut nm_pulse = nmPulse::new(10 as u8, 1 as u8);
    let mut nm_snake = nmSnake::new(RGB8::new(255, 0 ,0));
    let mut nm_wave = nmWave::new(RGB8::new(0, 0, 50));
    let mut nm_sin = nmSin::new(RGB8::new(0, 30, 0));
    
    // loop vals
    let mut nticks : u8 = 9;
    let mut mode : u8 = 5;

    // constants
    const THRESH : f32 = 0.8;

    loop {
        // Read X, Y, Z values
        let accel_data: F32x3 = lis3dh.accel_norm().unwrap();

        // Choose animation based on oritentation
        if accel_data.x > THRESH {
            mode = 0;
        }
        else if accel_data.x < -THRESH {
            mode = 1;
        }
        else if accel_data.y > THRESH {
            mode = 2;
        }
        else if accel_data.y < -THRESH {
            mode = 3;
        }

        // write frame to neopixel every nticks
        if nticks > 8 {
            nticks = 0;
            // itr thru the applicable nodes
            nm_pulse.next();
            nm_snake.next();
            nm_wave.next();
            nm_sin.next();

            // select list based off current node
            let ds: [RGB8; animations::NUM_PX] = match mode {
                0 => nm_wave.to_list(),
                1 => nm_pulse.to_list(),
                2 => nm_sin.to_list(),
                3 => nm_snake.to_list(),
                _ => [RGB8::new(0,0,0); animations::NUM_PX],
            };

            // write to neomatrix
            neopixels.write(ds.iter().cloned()).unwrap();
        }

        nticks += 1;
        delay_timer.delay_ms(5 as u32);
    }

}
