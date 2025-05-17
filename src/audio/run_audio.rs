use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use crossbeam_channel::Receiver;

use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapCons, HeapRb};

struct AudioConsumer {
    consumer: HeapCons<f32>,
    last_sample: f32,
}

impl AudioConsumer {
    fn new(consumer: HeapCons<f32>) -> Self {
        Self {
            consumer,
            last_sample: 0.0,
        }
    }

    fn pop(&mut self) -> f32 {
        if let Some(s) = self.consumer.try_pop() {
            self.last_sample = s;
            s
        } else {
            self.last_sample
        }
    }
}

pub fn run_audio(sample_rx: Receiver<f32>) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(44100),
            buffer_size: BufferSize::Default,
        };

        let rb = HeapRb::<f32>::new(1024);
        let (mut prod, cons) = rb.split();

        let stream = build_stream(&device, &config, AudioConsumer::new(cons)).unwrap();

        stream.play().unwrap();

        loop {
            if let Ok(s) = sample_rx.try_recv() {
                _ = prod.try_push(s);
            }
        }
    });
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: AudioConsumer,
) -> Result<cpal::Stream, ()> {
    let channels = config.channels as usize;

    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [f32], _info| {
                for frame in output.chunks_mut(channels) {
                    let s = consumer.pop();
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
