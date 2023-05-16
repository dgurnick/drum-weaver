use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let (_click_stream, click_stream_handle) = OutputStream::try_default().unwrap();
    let click_sink = Sink::try_new(&click_stream_handle).unwrap();

    let (_stream, _) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("examples/test.wav").unwrap());

    let (_stream_click, _) = OutputStream::try_default().unwrap();
    let click_file = BufReader::new(File::open("examples/test_click.wav").unwrap());

    let source = Decoder::new(file).unwrap();
    let click_source = Decoder::new(click_file).unwrap();

    sink.append(source);
    click_sink.append(click_source);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    click_sink.play();
    sink.sleep_until_end();
}
