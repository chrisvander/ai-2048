use ai_2048::{agent::random::RandomAgent, agent::Agent, game::Game};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("random game", |b| {
        b.iter_batched(
            || {
                let game = black_box(Game::new_seeded(0));
                let agent = black_box(RandomAgent::new_seeded(0, game));
                agent
            },
            |mut agent| {
                while !agent.get_game().game_over() {
                    agent.next_move();
                }
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
