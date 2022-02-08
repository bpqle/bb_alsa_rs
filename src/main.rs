use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};

fn main() {
    // Open default playback device
    let pcm = PCM::new("hw:1", Direction::Playback, false).unwrap();

// Set hardware parameters: 44100 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(2).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();

    let io = match pcm.io_u8() {
        Ok(io) => io,
        Err(error) => match pcm.io_i8() {
            Ok(io) => io,
            Err(error) => match pcm.io_u16() {
                Ok(io) => io,
                Err(error) => match pcm.io_i16() {
                    Ok(io) => io,
                    Err(error) => match pcm.io_u32() {
                        Ok(io) => io,
                        Err(error) => match pcm.io_i32() {
                            Ok(io) => io,
                            Err(error) => match pcm.io_f32() {
                                Ok(io) => io,
                                Err(error) => match pcm.io_f64() {
                                    Ok(io) => io,
                                    Err(error) => panic!("exhausted io options")
                                }
                            }
                        }
                    }
                }
            }
        }
    };
// Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();
    let swp = pcm.sw_params_current().unwrap();
    swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
    pcm.sw_params(&swp).unwrap();
}
