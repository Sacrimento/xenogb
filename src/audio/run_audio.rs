use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use crossbeam_channel::Receiver;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct AudioBuf {
    pub buf: VecDeque<f32>,
    max_size: usize,

    last_sample: f32,
}

impl AudioBuf {
    pub fn new(size: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(size),
            max_size: size,
            last_sample: 0.0,
        }
    }

    pub fn push(&mut self, sample: f32) {
        if self.buf.len() == self.max_size {
            self.buf.pop_front();
        }
        self.buf.push_back(sample);
    }

    pub fn pop(&mut self) -> f32 {
        // if self.buf.len() < self.max_size / 2 {
        //     return self.last_sample;
        // }

        if let Some(s) = self.buf.pop_front() {
            self.last_sample = s;
            s
        } else {
            println!("Buffer behind!");
            self.last_sample
        }
    }
}

pub fn run_audio(sample_rx: Receiver<f32>) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        // let mut supported_configs = device.supported_output_configs().unwrap();
        // let sconfig = supported_configs.find(|c| c.channels() == 2).unwrap();
        // println!("Supported config: {:?}", sconfig);

        // let config = device.default_output_config().unwrap().config();
        // println!("Default output device: {}", device.name().unwrap());
        // println!("Default output config: {:?}", config.buffer_size);

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(44100),
            buffer_size: BufferSize::Default,
        };

        let audio_buf = Arc::new(Mutex::new(AudioBuf::new(2048)));

        let stream = build_stream(&device, &config, audio_buf.clone(), sample_rx).unwrap();

        stream.play().unwrap();

        loop {
            // if let Ok(s) = sample_rx.try_recv() {
            //     audio_buf.lock().unwrap().push(s);
            // }
            // std::thread::sleep(Duration::from_secs(1));
            // std::thread::yield_now();
        }
    });
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_buf: Arc<Mutex<AudioBuf>>,
    sample_rx: Receiver<f32>,
) -> Result<cpal::Stream, ()> {
    let channels = config.channels as usize;

    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [f32], _info| {
                // println!("Audiobuf length: {}", audio_buf.lock().unwrap().buf.len());
                // println!("cpal buf len: {:?}", info);
                for frame in output.chunks_mut(channels) {
                    // let s = audio_buf.lock().unwrap().pop();
                    let s = sample_rx.try_recv().unwrap_or(0.0);
                    // println!("Sample: {}", s);
                    for out in frame.iter_mut() {
                        *out = s;
                    }
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .unwrap();

    Ok(stream)
}
