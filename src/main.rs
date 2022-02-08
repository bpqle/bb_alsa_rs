use std::thread;
use std::time::Duration;
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};

fn main() {
    thread::spawn(move || {
        let pcm = PCM::new("hw:1", Direction::Playback, false).unwrap();

// Set hardware parameters: 44100 Hz / Mono / 16 bit
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_channels(2).unwrap();
        hwp.set_rate(44100, ValueOr::Nearest).unwrap();
        hwp.set_format(Format::s16()).unwrap();
        hwp.set_access(Access::RWInterleaved).unwrap();
        pcm.hw_params(&hwp).unwrap();
        println!("Begin IO test");

        if pcm.io_u8().is_err() {
            println!("u8 failed");
            if pcm.io_i8().is_err() {
                println!("i8 failed");

                if pcm.io_u16().is_err() {
                    println!("u16 failed");

                    if pcm.io_i16().is_err() {
                        println!("i16 failed");

                        if pcm.io_u32().is_err() {
                            println!("u32 failed");

                            if pcm.io_f32().is_err() {
                                println!("f32 failed");

                                if pcm.io_f64().is_err() {
                                    println!("f64 failed");

                                } else {let io = pcm.io_f64().unwrap();};
                            } else {let io = pcm.io_f32().unwrap();};
                        } else {let io = pcm.io_u32().unwrap();};
                    } else {let io = pcm.io_i16().unwrap();};
                } else {let io = pcm.io_u16().unwrap();};
            } else {let io = pcm.io_i8().unwrap();};
        } else {let io = pcm.io_u8().unwrap();};
// Make sure we don't start the stream too early
        let hwp = pcm.hw_params_current().unwrap();
        let swp = pcm.sw_params_current().unwrap();
        swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
        pcm.sw_params(&swp).unwrap();
    });
    thread::sleep(Duration::from_secs(20))// Open default playback device

}
