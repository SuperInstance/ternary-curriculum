# ternary-curriculum

Curriculum learning for ternary agents — progressively harder environments that train better strategies.

## Overview

This crate implements a **curriculum learning** framework for agents that operate in ternary decision spaces (three-valued logic: true, false, unknown / -1, 0, +1 / low, medium, high). Instead of training on the hardest problems from the start, the curriculum gradually increases difficulty, allowing the agent to build foundational skills before tackling complex scenarios.

## Curriculum Learning Theory

### What is Curriculum Learning?

Curriculum learning is a training strategy inspired by how humans learn — starting with easy examples and progressively introducing harder ones. The term was formalized by Bengio et al. (2009), but the idea has deep roots in educational psychology (Piaget's stages, Vygotsky's zone of proximal development).

### Why Does It Work?

1. **Warm-start effect**: Easy examples provide a good initialization for model parameters
2. **Curse of dimensionality**: Hard problems often have many local optima; easy problems have fewer, guiding the learner toward better basins
3. **Knowledge scaffolding**: Skills learned on easy tasks transfer to harder tasks
4. **Confidence building**: Early successes maintain exploration pressure rather than premature exploitation

### Ternary Agents

Ternary agents operate in a three-valued decision space. This makes curriculum learning particularly effective because:

- The decision space has natural ordinal structure (low → medium → high)
- Difficulty can be controlled along multiple axes: noise level, horizon length, opponent strength, state complexity
- Mastery of binary distinctions (two of three values) naturally scaffolds toward full ternary reasoning

### Difficulty Schedules

The crate supports three scheduling strategies:

| Schedule | Formula | Best For |
|---|---|---|
| **Linear** | `d(t) = start + (end - start) * (t / total)` | Predictable, steady progress |
| **Exponential** | `d(t) = start * (end / start)^(t / total)` | Slow start, rapid late acceleration |
| **Adaptive** | Adjusts based on agent mastery rate | Variable learner speeds |

### Mastery-Based Progression

Rather than fixed-time transitions, the curriculum supports **mastery-based progression**: the agent advances only when it demonstrates sufficient performance on the current lesson. This prevents the common failure mode of advancing too quickly.

## Core Concepts

- **Lesson**: A single training stage with a difficulty level, environment configuration, and success criteria
- **Curriculum**: An ordered sequence of lessons with progression rules
- **CurriculumTrainer**: Orchestrates training an agent through a curriculum, tracking progress
- **DifficultySchedule**: Configurable difficulty ramp (linear, exponential, adaptive)
- **MasteryTracker**: Tracks which lessons are mastered and which need review
- **CurriculumResult**: Structured result with lessons completed, mastery rate, and time spent

## Usage

```rust
use ternary_curriculum::*;

// Create a curriculum with a linear schedule
let schedule = DifficultySchedule::linear(0.1, 1.0, 10);
let mut curriculum = Curriculum::new("Basic Training");

for i in 0..10 {
    let difficulty = schedule.difficulty_at(i);
    let lesson = Lesson::new(i, difficulty)
        .with_success_threshold(0.7)
        .with_max_attempts(5);
    curriculum.add_lesson(lesson);
}

// Train through the curriculum
let mut trainer = CurriculumTrainer::new(curriculum);
let result = trainer.train(|lesson, state| {
    // Your training logic here
    // Return true if the agent passed this attempt
    state.attempt < lesson.max_attempts()
});

println!("Mastery rate: {:.1}%", result.mastery_rate() * 100.0);
println!("Lessons completed: {}/{}", result.lessons_completed(), result.total_lessons());
```

## License

MIT
