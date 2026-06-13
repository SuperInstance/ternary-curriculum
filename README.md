# Ternary Curriculum

Curriculum learning for ternary agents — **progressively harder environments** that train better strategies. Generates lesson sequences with configurable difficulty schedules (linear, exponential, adaptive), tracks mastery per lesson, and produces structured training reports.

## Why It Matters

Reinforcement learning agents trained on fixed difficulty often converge to local optima. **Curriculum learning** — introduced by Bengio et al. (2009) — starts with easy tasks and progressively increases difficulty, mimicking how humans and animals learn. The result: faster convergence, better final performance, and more robust generalization.

For ternary agents (whose actions are {-1, 0, +1}), curriculum learning is especially impactful because the ternary action space is small enough that individual lessons have clear, measurable outcomes — pass/fail/inconclusive — enabling precise mastery tracking.

The mathematical foundation: if $\pi_\theta(a|s)$ is the agent's policy and $D(\theta, d)$ is the expected return at difficulty $d$, then curriculum learning optimizes:

$$\max_\theta \int_0^1 D(\theta, d) \cdot w(d) \, dd$$

where $w(d)$ is the curriculum schedule — a weighting function over difficulty levels.

## How It Works

### Difficulty Schedules

**Linear**:
$$d(i) = d_{\text{start}} + (d_{\text{end}} - d_{\text{start}}) \cdot \frac{i}{S}$$

**Exponential**:
$$d(i) = d_{\text{start}} \cdot \left(\frac{d_{\text{end}}}{d_{\text{start}}}\right)^{i/S}$$

**Adaptive**: Adjusts step size based on mastery rate $\rho$:

$$\alpha_t = \begin{cases} 1.5 & \rho_t > 0.8 \;\text{(accelerate)} \\ 0.5 & \rho_t < 0.4 \;\text{(slow down)} \\ 1.0 & \text{otherwise} \end{cases}$$

$$d_{t+1} = \text{clamp}(d_t + \Delta \cdot \alpha_t, \; d_{\text{start}}, \; d_{\text{end}})$$

The adaptive schedule uses exponential moving average: $\alpha \leftarrow 0.7\alpha + 0.3\alpha_{\text{target}}$ for smooth transitions.

### Mastery Tracking

Each lesson has a success threshold $\theta$ and max attempts $M$. The tracker records:

| Status | Condition |
|--------|-----------|
| NotStarted | 0 attempts |
| InProgress | < threshold met, attempts < M |
| Mastered | score ≥ θ |
| Failed | attempts = M without mastery |

Mastery rate across the curriculum: $\rho = M_{\text{mastered}} / N_{\text{lessons}}$.

### Progression Rules

| Rule | Advancement Criterion |
|------|----------------------|
| PassOnce | Score ≥ threshold once |
| ConsecutivePasses(n) | n consecutive passes |
| AverageScore | Mean score ≥ threshold |

### Complexity

| Operation | Time |
|-----------|------|
| `DifficultySchedule::difficulty_at(i)` | O(1) |
| `MasteryTracker::record_attempt(i, score, lesson)` | O(1) |
| `CurriculumTrainer::train(F)` | O(L · A) — L lessons, A avg attempts |
| `train_with_schedule(schedule, F)` | O(S · A) — S schedule steps |

## Quick Start

```rust
use ternary_curriculum::{Curriculum, DifficultySchedule, CurriculumTrainer, Lesson};

// Build a curriculum from a schedule
let schedule = DifficultySchedule::linear(0.1, 1.0, 5);
let curriculum = Curriculum::from_schedule("agent-training", &schedule);
let mut trainer = CurriculumTrainer::new(curriculum);

// Train with a scoring function
let result = trainer.train(|lesson, state| {
    // Your training logic here — returns score 0.0..1.0
    // Higher difficulty → harder environment
    let difficulty = lesson.difficulty;
    if difficulty < 0.3 { 1.0 }      // easy → pass
    else if difficulty < 0.7 { 0.6 }  // medium → marginal
    else { 0.2 }                       // hard → fail
});

println!("{}", result.summary());
// "Curriculum 'agent-training': 3/5 lessons completed, 60.0% mastery, ..."
```

## API

### Schedule

| Type/Method | Description |
|-------------|-------------|
| `DifficultySchedule::linear(start, end, steps)` | Linear ramp |
| `DifficultySchedule::exponential(start, end, steps)` | Exponential ramp |
| `DifficultySchedule::adaptive(start, end, steps)` | Mastery-driven ramp |
| `.difficulty_at(i) → f64` | Difficulty at step i |
| `.update_mastery(rate)` | Adjust adaptive schedule |

### Curriculum

| Type/Method | Description |
|-------------|-------------|
| `Lesson::new(index, difficulty)` | Build a lesson |
| `Curriculum::new(name)` | Empty curriculum |
| `Curriculum::from_schedule(name, schedule)` | Auto-generate lessons |
| `CurriculumTrainer::new(curriculum)` | Create trainer |
| `.train(F) → CurriculumResult` | Train through all lessons |
| `.train_with_schedule(schedule, F) → CurriculumResult` | Train with dynamic schedule |

### Results

| Type/Method | Description |
|-------------|-------------|
| `CurriculumResult` | Full training report |
| `.mastery_rate() → f64` | Fraction of lessons mastered |
| `.fully_mastered() → bool` | All lessons passed |
| `.summary() → String` | Human-readable report |

## Architecture Notes

The curriculum system implements the **γ + η = C** conservation principle through learning dynamics:

- **γ (structure)**: the lesson sequence — ordered, with fixed difficulty progression
- **η (dynamics)**: the agent's performance — the score stream that determines advancement
- **C (conservation)**: the mastery invariant — the curriculum is complete when all lessons are mastered, and the schedule ensures the agent is never overwhelmed (η never exceeds γ capacity)

The adaptive schedule directly embodies the γ-η balance: when the agent's η (performance perturbation) is strong (high mastery), the curriculum increases γ (difficulty) to match. When η is weak, γ decreases to provide scaffolding.

## References

- Bengio, Y. et al. (2009). *Curriculum Learning*. ICML. — The foundational paper.
- Graves, A. et al. (2017). *Automated Curriculum Learning for Neural Networks*. arXiv:1704.03003.
| Narvekar, S. et al. (2020). *Curriculum Learning for Reinforcement Learning Domains: A Framework and Survey*. JMLR.
| Elman, J.L. (1993). *Learning and Development in Neural Networks: The Importance of Starting Small*. Cognition.

## License: MIT
