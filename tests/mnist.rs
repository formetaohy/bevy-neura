mod common;

#[path = "../examples/mnist/canvas.rs"]
mod canvas;
#[path = "../examples/mnist/dataset.rs"]
mod dataset;
#[path = "../examples/mnist/digit.rs"]
mod digit;
#[path = "../examples/mnist/ink.rs"]
mod ink;
#[path = "../examples/mnist/learner.rs"]
mod learner;
#[path = "../examples/mnist/lenet.rs"]
mod lenet;
#[path = "../examples/mnist/puzzle.rs"]
mod puzzle;
#[path = "../examples/mnist/random.rs"]
mod random;
#[path = "../examples/mnist/recognizer.rs"]
mod recognizer;

use bevy::prelude::*;
use bevy_neura::NeuraRuntime;
use dataset::Archives;
use neura::MemoryRequest;
use puzzle::Operation;

const SIDE: usize = digit::SIDE;
const READBACK: u64 = 1 << 22;
const RATE: f32 = 1e-3;

fn app(heap_bytes: u64) -> App {
    common::app(MemoryRequest {
        heap_bytes,
        readback_bytes: READBACK,
        readback_slots: 4,
        ..MemoryRequest::default()
    })
}

fn drawn(points: &[Vec2]) -> canvas::Canvas {
    let mut canvas = canvas::Canvas::new();
    canvas.press(points[0]);
    for point in &points[1..] {
        canvas.drag(*point);
    }
    canvas.lift();
    canvas
}

fn vertical(column: f32) -> Vec<Vec2> {
    (40..260)
        .step_by(4)
        .map(|row| Vec2::new(column, row as f32))
        .collect()
}

fn ring(centre: Vec2, radius: f32) -> Vec<Vec2> {
    (0..=48)
        .map(|step| {
            let angle = std::f32::consts::TAU * step as f32 / 48.0;
            centre + Vec2::new(angle.sin(), angle.cos()) * radius
        })
        .collect()
}

fn centre_of_mass(values: &[f32]) -> Vec2 {
    let mass = values.iter().sum::<f32>();
    assert!(mass > 0.0, "a digit of ink weighs something");
    values
        .iter()
        .enumerate()
        .fold(Vec2::ZERO, |sum, (at, value)| {
            sum + Vec2::new((at % SIDE) as f32, (at / SIDE) as f32) * *value
        })
        / mass
}

fn bars(rows: u32) -> (Vec<f32>, Vec<f32>) {
    let mut digits = vec![0.0f32; rows as usize * SIDE * SIDE];
    let mut answers = vec![0.0f32; rows as usize];
    for row in 0..rows as usize {
        let answer = (row % 2) as u32;
        let band = if answer == 0 { 4..8 } else { 20..24 };
        for at in 4..24 {
            for column in band.clone() {
                digits[row * SIDE * SIDE + at * SIDE + column] = 1.0;
            }
        }
        answers[row] = answer as f32;
    }
    (digits, answers)
}

fn images(count: u32) -> Vec<u8> {
    let mut bytes = 0x0000_0803u32.to_be_bytes().to_vec();
    bytes.extend(count.to_be_bytes());
    bytes.extend((SIDE as u32).to_be_bytes());
    bytes.extend((SIDE as u32).to_be_bytes());
    for image in 0..count {
        bytes.extend(std::iter::repeat_n(image as u8, SIDE * SIDE));
    }
    bytes
}

fn answers(count: u32) -> Vec<u8> {
    let mut bytes = 0x0000_0801u32.to_be_bytes().to_vec();
    bytes.extend(count.to_be_bytes());
    bytes.extend((0..count).map(|answer| answer as u8));
    bytes
}

#[test]
fn every_puzzle_asks_for_one_digit() {
    let mut random = random::Random::seeded(0x5eed_0001);
    let mut held = [0u32; 3];
    for _ in 0..500 {
        let puzzle = puzzle::Puzzle::draw(&mut random);
        let (left, right, answer) = (puzzle.left, puzzle.right, puzzle.answer());
        assert!(
            left < puzzle::DIGITS && right < puzzle::DIGITS && answer < puzzle::DIGITS,
            "{left} {} {right} answers {answer}",
            puzzle.operation.symbol(),
        );
        assert_eq!(
            match puzzle.operation {
                Operation::Add => left + right,
                Operation::Subtract => left - right,
                Operation::Multiply => left * right,
            },
            answer,
            "the puzzle holds the answer it shows",
        );
        assert_eq!(
            puzzle.text(),
            format!("{left} {} {right} =", puzzle.operation.symbol()),
        );
        held[match puzzle.operation {
            Operation::Add => 0,
            Operation::Subtract => 1,
            Operation::Multiply => 2,
        }] += 1;
    }
    assert!(
        held.iter().all(|count| *count > 0),
        "the draws hold every operation: {held:?}",
    );
}

#[test]
fn the_canvas_holds_the_strokes_it_draws() {
    let mut canvas = canvas::Canvas::new();
    assert!(!canvas.drawn() && !canvas.drawing());
    let before = canvas.revision();
    canvas.press(Vec2::new(30.0, 30.0));
    assert!(canvas.drawn() && canvas.drawing());
    canvas.drag(Vec2::new(120.0, 200.0));
    assert!(canvas.revision() > before, "a stroke moves the canvas on");
    assert!(
        canvas.ink().values().iter().any(|value| *value > 0.0),
        "a stroke paints the ink it draws",
    );
    canvas.lift();
    assert!(!canvas.drawing());
    canvas.clear();
    assert!(!canvas.drawn());
    assert!(canvas.ink().values().iter().all(|value| *value == 0.0));
}

#[test]
fn the_box_places_the_pointer_it_reports() {
    assert_eq!(
        canvas::placed(Vec2::new(-0.5, -0.5)),
        Vec2::ZERO,
        "the corner the box reports at the top left is the corner of its ink",
    );
    assert_eq!(
        canvas::placed(Vec2::new(0.5, 0.5)),
        Vec2::splat(canvas::SIDE),
        "the corner the box reports at the bottom right is the corner of its ink",
    );
    assert_eq!(
        canvas::placed(Vec2::ZERO),
        Vec2::splat(canvas::SIDE / 2.0),
        "the centre the box reports is the centre of its ink",
    );
}

#[test]
fn a_blank_canvas_holds_no_digit() {
    let mut canvas = canvas::Canvas::new();
    assert!(digit::Digit::drawn(&canvas).is_none());
    assert!(canvas.bounds().is_none());
    canvas.press(Vec2::splat(150.0));
    canvas.lift();
    assert!(
        digit::Digit::drawn(&canvas).is_some(),
        "a tap of the pen draws a dot",
    );
}

#[test]
fn a_drawn_digit_lands_centred_in_the_frame() {
    for (points, thin) in [
        (vertical(150.0), true),
        (ring(Vec2::splat(150.0), 90.0), false),
    ] {
        let canvas = drawn(&points);
        let digit = digit::Digit::drawn(&canvas).expect("a stroke draws a digit");
        let values = digit.values();
        assert!(
            values.iter().all(|value| (0.0..=1.0).contains(value)),
            "the digit holds ink between nothing and a whole pixel",
        );
        let mass = centre_of_mass(values);
        assert!(
            (mass - Vec2::splat(SIDE as f32 / 2.0)).length() <= 0.75,
            "the ink of a stroke weighs {mass} about the centre of the frame",
        );
        let rows = (0..SIDE)
            .filter(|row| {
                values[row * SIDE..(row + 1) * SIDE]
                    .iter()
                    .any(|v| *v > 0.0)
            })
            .count();
        let columns = (0..SIDE)
            .filter(|column| (0..SIDE).any(|row| values[row * SIDE + column] > 0.0))
            .count();
        assert!(
            rows.max(columns) >= 18,
            "a stroke of {rows} rows and {columns} columns fills a box of 20",
        );
        let narrow = rows.min(columns);
        assert!(
            if thin { narrow <= 6 } else { narrow >= 14 },
            "a stroke of {rows} rows and {columns} columns spans the shape it draws",
        );
    }
}

#[test]
fn the_archives_hold_the_digits_they_carry() {
    let dataset = dataset::Dataset::of(Archives {
        train_digits: images(3),
        train_answers: answers(3),
        test_digits: images(2),
        test_answers: answers(2),
    });
    assert_eq!(dataset.train_size(), 3);
    assert_eq!(dataset.test_size(), 2);
    for index in 0..dataset.train_size() {
        let (digits, answer) = dataset.train(index);
        assert_eq!(digits.len(), SIDE * SIDE);
        assert!(digits.iter().all(|value| *value == index as u8));
        assert_eq!(answer, index as u8);
        assert_eq!(dataset.test(0).1, 0);
    }
}

#[test]
fn the_network_reads_the_digits_it_learned() {
    let mut app = app(1 << 28);
    app.update();
    let runtime = app.world().resource::<NeuraRuntime>();
    let learner = learner::Learner::attach(runtime);
    let recognizer = recognizer::Recognizer::share(runtime, learner.model());
    let (digits, answers) = bars(learner::BATCH);
    let mut loss = f32::MAX;
    for step in 0..24 {
        loss = learner.step(runtime, &digits, &answers, RATE);
        if step % 8 == 0 {
            println!("step {step} holds a loss of {loss}");
        }
    }
    assert!(loss < 0.2, "a loss of {loss} over the bars it trains on");
    for (row, guess) in recognizer.read(runtime, &digits).into_iter().enumerate() {
        assert_eq!(
            u32::from(guess.digit),
            answers[row] as u32,
            "the digit {row} reads {} where it holds {}",
            guess.digit,
            answers[row],
        );
    }
}
