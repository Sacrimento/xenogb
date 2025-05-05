use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Receiver;

pub fn run_audio(sample_rx: Receiver<f32>) {
    std::thread::spawn(|| {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = device
            .default_output_config()
            .expect("No device counfig")
            .config();

        let stream = build_stream(&device, &config, sample_rx).unwrap();

        stream.play().unwrap();

        loop {
            // Keep the thread alive
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_rx: Receiver<f32>,
) -> Result<cpal::Stream, ()> {
    let channels = config.channels as usize;

    let mut start = Instant::now();
    let mut s_count = 0;

    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [f32], _| {
                for frame in output.chunks_mut(channels) {
                    let s = if let Ok(s) = sample_rx.try_recv() {
                        s_count += 1;
                        s
                    } else {
                        // If no sample is available, fill with silence
                        0.0
                    };

                    for out in frame.iter_mut() {
                        *out = s;
                    }
                }

                if start.elapsed() > Duration::from_secs(1) {
                    println!(
                        "Samples per second: {} (output len = {})",
                        s_count,
                        output.len()
                    );
                    s_count = 0;
                    start = Instant::now();
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .unwrap();

    Ok(stream)
}
