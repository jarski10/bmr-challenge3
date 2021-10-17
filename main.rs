#![no_std]
#![no_main]

use embedded_graphics::mono_font::ascii::{FONT_10X20};
use longan_nano::hal::timer::{Event, Timer};
use longan_nano::hal::eclic::{EclicExt, Level, LevelPriorityBits, Priority, TriggerType};
use panic_halt as _;

use embedded_graphics::mono_font::{
    MonoTextStyleBuilder,
};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use embedded_graphics::text::Text;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};
use riscv_rt::entry;

static mut STATE : u8 = 1;
static mut TIMER_STAT : Option<Timer<longan_nano::hal::pac::TIMER1>> = None;

#[no_mangle]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
fn TIMER1() {
    unsafe{
        riscv::interrupt::disable();
        TIMER_STAT.as_mut().unwrap().clear_update_interrupt_flag();
        if STATE == 0 {
            STATE = 1;
        } else {
            STATE = 0
        }
        riscv::interrupt::enable();
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    longan_nano::hal::pac::ECLIC::reset();
    longan_nano::hal::pac::ECLIC::set_threshold_level(Level::L0);
    longan_nano::hal::pac::ECLIC::set_level_priority_bits(LevelPriorityBits::L3P1);
    longan_nano::hal::pac::ECLIC::setup(pac::Interrupt::TIMER1, TriggerType::Level, Level::L1, Priority::P1);

    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLUE)
        .build();

    let mut timer = Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);
    timer.listen(Event::Update);

    unsafe {
        TIMER_STAT = Some(timer);
        longan_nano::hal::pac::ECLIC::unmask(pac::Interrupt::TIMER1);
        riscv::interrupt::enable();
    }

    loop {
        // Clear screen
        Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut lcd)
        .unwrap();
        unsafe {
        if STATE == 1 {
            Text::new(" State = 1 ", Point::new(45, 40), style)
            .draw(&mut lcd)
            .unwrap();
        } else {
            Text::new(" State = 0 ", Point::new(10, 40), style)
            .draw(&mut lcd)
            .unwrap();
        }
        riscv::asm::wfi();
        }
    }
}
