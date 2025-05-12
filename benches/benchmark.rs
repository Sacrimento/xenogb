use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crossbeam_channel::unbounded;
use xenogb::core::{
    cpu::LR35902CPU,
    mem::{boot::BootRom, bus::Bus, cartridge::parse_cartridge},
};

fn setup_cpu(cartridge: PathBuf) -> LR35902CPU {
    let (vcs, _) = unbounded();
    let (acs, _) = unbounded();

    LR35902CPU::new(
        Bus::new(
            parse_cartridge(PathBuf::from("benches/roms").join(cartridge)).unwrap(),
            BootRom::NONE,
            vcs,
            acs,
        ),
        false,
        u32::MAX,
    )
}

pub fn cpu_step_bench(c: &mut Criterion) {
    c.bench_function("cpu nop step", |b| {
        let mut cpu = setup_cpu(PathBuf::from("nop.gb"));

        b.iter(|| {
            #[allow(clippy::unit_arg)]
            black_box(cpu.step());
        });
    });

    c.bench_function("cpu inc a step", |b| {
        let mut cpu = setup_cpu(PathBuf::from("inc_a.gb"));

        b.iter(|| {
            #[allow(clippy::unit_arg)]
            black_box(cpu.step());
        });
    });

    c.bench_function("cpu add a, b step", |b| {
        let mut cpu = setup_cpu(PathBuf::from("add_a_b.gb"));

        b.iter(|| {
            #[allow(clippy::unit_arg)]
            black_box(cpu.step());
        });
    });
}

criterion_group!(benches, cpu_step_bench);
criterion_main!(benches);
