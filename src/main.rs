use std::thread;

use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use simple_logger::SimpleLogger;
use log::{info};
use audrey::read::BufFileReader;

use std::ffi::OsString;
use std::fs::read_dir;
use std::sync::Arc;


#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let args = Cli::parse();
    let switch_main  = Arc::new(AtomicBool::new(false));
    let dir = args.path.clone();

    let (tx, rx) = std::sync::mpsc::channel();

    let switch2 = Arc::clone(&switch_main);
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line_buf = String::new();
        while let Ok(_) = stdin.read_line(&mut line_buf) {
            let line = line_buf.trim_end().to_string();
            line_buf.clear();
            tx.send(line).unwrap();
            switch2.store(false, Ordering::Release);
        }
    });

    let switch1 = Arc::clone(&switch_main);
    thread::spawn(move || {

        info!("Playback Thread Initiated");
        let pcm = PCM::new("hw:1,0", Direction::Playback, false).unwrap();
        let hwp = HwParams::any(&pcm).unwrap();

        // Set hardware parameters: 44100 Hz / Mono / 16 bit
        hwp.set_channels(2).unwrap();
        hwp.set_rate(44100, ValueOr::Nearest).unwrap();
        hwp.set_format(Format::s16()).unwrap();
        hwp.set_access(Access::RWInterleaved).unwrap();

        let reader = read_dir(dir.clone()).unwrap();
        'stim: for result in reader {
            let file = result.unwrap();
            let path = file.path();
            let fname = OsString::from(path.clone().strip_prefix(&*dir).unwrap());

            if path.extension().unwrap() == "wav" {
                let wav = audrey::open(path).unwrap();
                let wav_channels = wav.description().channel_count();

                let data = process_audio(wav, wav_channels);

                pcm.hw_params(&hwp).unwrap();
                let io = pcm.io_i16().unwrap();
                io.writei(&data).expect("Couldn't do io.writei on data");
                info!("Begin playing {:?}", fname);

                switch1.store(true, Ordering::Release);
                while pcm.state() == State::Running {
                    let pb = switch1.load(Ordering::Acquire);
                    match pb {
                        true => { continue },
                        false => {
                            pcm.drop().unwrap();
                            info!("Interrupted {:?}, moving on ", fname);
                            switch1.store(true, Ordering::Release);
                            continue 'stim
                        }
                    }
                }
                info!("Current state is {:?}", pcm.state());
                pcm.drop().unwrap();
                //pcm.reset().unwrap();
                info!("Finished playing {:?}, continuing", fname);
            }
        }
    });

    loop {
        let text = rx.recv().unwrap();
        info!("Received {:?}",text);
    }
}


fn process_audio(mut wav: BufFileReader, wav_channels: u32) -> Vec<i16>{
    let mut result = Vec::new();
    let hw_channels = 2;

    if wav_channels == 1 {
        result = wav.frames::<[i16;1]>()
            .map(Result::unwrap)
            .map(|file| audrey::dasp_frame::Frame::scale_amp(file, 0.8))
            .map(|note| note[0])
            .collect::<Vec<i16>>();
        if hw_channels == 2 {
            result = result.iter()
                .map(|note| [note, note])
                .flatten()
                .map(|f| *f )
                .collect::<Vec<_>>()
        }
    } else if wav_channels == 2 {
        result = wav.frames::<[i16;2]>()
            .map(Result::unwrap)
            .map(|file| audrey::dasp_frame::Frame::scale_amp(file, 0.8))
            .flatten()
            .collect::<Vec<i16>>();
        if hw_channels == 1 {
            result = result.iter()
                .enumerate()
                .filter(|f| f.0 % 2 == 0)
                .map(|f| *f.1)
                .collect::<Vec<_>>()
        }
    };
    result
}