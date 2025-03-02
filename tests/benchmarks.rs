#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use crate::core::EventBus;
    use crate::shared::SharedEventBus;

    #[derive(Debug, Clone)]
    struct BenchEvent { data: Vec<u8> }

    fn bench_event_publishing(c: &mut Criterion) {
        let event_bus = EventBus::new();
        let event = BenchEvent { data: vec![0; 1024] };
        
        c.bench_function("publish small event", |b| {
            b.iter(|| {
                event_bus.publish(black_box(&event)).unwrap();
            })
        });
    }

    criterion_group!(benches, bench_event_publishing);
    criterion_main!(benches);
}