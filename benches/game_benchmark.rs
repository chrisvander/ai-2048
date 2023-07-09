use ai_2048::{
    agent::random::RandomAgent,
    agent::Agent,
    game::{Game, Move},
};
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

    c.bench_function("merge horizontal", |b| {
        b.iter_batched(
            || {
                let mut game = black_box(Game::new_seeded(0));
                // spin the board around a bit
                for _ in 0..10 {
                    game.update(Move::Up);
                    game.update(Move::Left);
                    game.update(Move::Right);
                    game.update(Move::Down);
                }
                game
            },
            |mut game| {
                game.update(black_box(Move::Left));
                game.update(black_box(Move::Right));
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("merge vertical", |b| {
        b.iter_batched(
            || {
                let mut game = black_box(Game::new_seeded(0));
                // spin the board around a bit
                for _ in 0..10 {
                    game.update(Move::Up);
                    game.update(Move::Left);
                    game.update(Move::Right);
                    game.update(Move::Down);
                }
                game
            },
            |mut game| {
                game.update(black_box(Move::Up));
                game.update(black_box(Move::Down));
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
