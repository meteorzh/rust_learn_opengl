use std::{io::{BufReader, Cursor}, fs};

use rodio::{Decoder, Source};

fn main() {
    // let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("Sound I/O failed");
    // let file = BufReader::new(Cursor::new(include_bytes!("../audio/breakout.mp3")));
    // let source = Decoder::new(file).expect("Sound decoding failed");
    // let converted_samples = source.convert_samples();
    // stream_handle
    //     .play_raw(converted_samples)
    //     .expect("Sound playback failed");

    // std::thread::sleep(std::time::Duration::from_secs(5));

    let (_stream, handle) =
        rodio::OutputStream::try_default().expect("Output stream failed to open");
    let sink = rodio::Sink::try_new(&handle).expect("Sink open failed");

    let file = Cursor::new(fs::read("src/audio/breakout.mp3").unwrap());
    sink.append(rodio::Decoder::new(BufReader::new(file)).expect("Decode failed"));
    sink.sleep_until_end();
}