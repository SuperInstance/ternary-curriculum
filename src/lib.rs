//! Curriculum learning for ternary agents.
//!
//! This crate provides a framework for progressive training through
//! increasingly difficult lessons, with configurable difficulty schedules
//! and mastery tracking.

use std::time::{Duration, Instant};

// ── DifficultySchedule ──────────────────────────────────────────────────────

/// Configurable difficulty ramp strategy.
#[derive(Debug, Clone)]
pub enum DifficultySchedule {
    /// Linear interpolation from start to end over `steps` increments.
    Linear { start: f64, end: f64, steps: usize },
    /// Exponential interpolation from start to end over `steps` increments.
    Exponential { start: f64, end: f64, steps: usize },
    /// Adaptive schedule that adjusts based on mastery feedback.
    Adaptive {
        start: f64,
        end: f64,
        steps: usize,
        current: f64,
        mastery_rate: f64,
        acceleration: f64,
    },
}

impl DifficultySchedule {
    /// Create a linear schedule from `start` to `end` over `steps` increments.
    pub fn linear(start: f64, end: f64, steps: usize) -> Self {
        Self::Linear { start, end, steps }
    }

    /// Create an exponential schedule from `start` to `end` over `steps` increments.
    pub fn exponential(start: f64, end: f64, steps: usize) -> Self {
        Self::Exponential { start, end, steps }
    }

    /// Create an adaptive schedule that speeds up when mastery is high and slows down when low.
    pub fn adaptive(start: f64, end: f64, steps: usize) -> Self {
        Self::Adaptive {
            start,
            end,
            steps,
            current: start,
            mastery_rate: 0.5,
            acceleration: 1.0,
        }
    }

    /// Get the difficulty at step `i`.
    pub fn difficulty_at(&self, i: usize) -> f64 {
        match self {
            Self::Linear { start, end, steps } => {
                if *steps == 0 {
                    return *end;
                }
                let t = (i as f64) / (*steps as f64);
                start + (end - start) * t.min(1.0)
            }
            Self::Exponential { start, end, steps } => {
                if *steps == 0 {
                    return *end;
                }
                let t = ((i as f64) / (*steps as f64)).min(1.0);
                if *start == 0.0 {
                    *end * t
                } else {
                    start * (end / start).powf(t)
                }
            }
            Self::Adaptive { current, .. } => *current,
        }
    }

    /// Update adaptive schedule with the latest mastery rate.
    pub fn update_mastery(&mut self, mastery_rate: f64) {
        if let Self::Adaptive {
            start,
            end,
            current,
            mastery_rate: mr,
            acceleration,
            ..
        } = self
        {
            *mr = mastery_rate;
            // If mastery is high, accelerate; if low, decelerate
            let target_accel = if mastery_rate > 0.8 {
                1.5
            } else if mastery_rate < 0.4 {
                0.5
            } else {
                1.0
            };
            *acceleration = *acceleration * 0.7 + target_accel * 0.3;
            let step_size = (*end - *start) / 20.0; // granularity
            *current = (*current + step_size * *acceleration).clamp(*start, *end);
        }
    }

    /// Total number of steps in this schedule.
    pub fn steps(&self) -> usize {
        match self {
            Self::Linear { steps, .. }
            | Self::Exponential { steps, .. }
            | Self::Adaptive { steps, .. } => *steps,
        }
    }
}

impl Default for DifficultySchedule {
    fn default() -> Self {
        Self::linear(0.1, 1.0, 10)
    }
}

// ── Lesson ──────────────────────────────────────────────────────────────────

/// A single lesson in the curriculum with difficulty, config, and success criteria.
#[derive(Debug, Clone)]
pub struct Lesson {
    /// Lesson index in the curriculum.
    pub index: usize,
    /// Difficulty level (0.0 to 1.0 typically).
    pub difficulty: f64,
    /// Score threshold to consider the lesson passed.
    pub success_threshold: f64,
    /// Maximum attempts before giving up or moving on.
    pub max_attempts: usize,
    /// Arbitrary environment configuration as key-value pairs.
    pub env_config: Vec<(String, String)>,
    /// Human-readable description.
    pub description: String,
}

impl Lesson {
    /// Create a new lesson with the given index and difficulty.
    pub fn new(index: usize, difficulty: f64) -> Self {
        Self {
            index,
            difficulty,
            success_threshold: 0.7,
            max_attempts: 10,
            env_config: Vec::new(),
            description: format!("Lesson {} (difficulty {:.2})", index, difficulty),
        }
    }

    /// Set the success threshold (0.0–1.0).
    pub fn with_success_threshold(mut self, threshold: f64) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set maximum attempts.
    pub fn with_max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    /// Add an environment config entry.
    pub fn with_env_config(mut self, key: &str, value: &str) -> Self {
        self.env_config.push((key.to_string(), value.to_string()));
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Get an environment config value by key.
    pub fn env(&self, key: &str) -> Option<&str> {
        self.env_config
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Check if a score meets the success threshold.
    pub fn is_passed(&self, score: f64) -> bool {
        score >= self.success_threshold
    }
}

// ── Curriculum ──────────────────────────────────────────────────────────────

/// Progression rule for how the agent moves between lessons.
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressionRule {
    /// Advance after passing once.
    PassOnce,
    /// Advance after achieving `n` consecutive passes.
    ConsecutivePasses(usize),
    /// Advance when average score over all attempts exceeds threshold.
    AverageScore,
}

impl Default for ProgressionRule {
    fn default() -> Self {
        Self::PassOnce
    }
}

/// An ordered sequence of lessons with progression rules.
#[derive(Debug, Clone)]
pub struct Curriculum {
    /// Name of this curriculum.
    pub name: String,
    /// Ordered list of lessons.
    lessons: Vec<Lesson>,
    /// How the agent progresses between lessons.
    pub progression: ProgressionRule,
    /// Whether to allow revisiting previous lessons for review.
    pub allow_review: bool,
    /// Review threshold: if mastery drops below this, trigger review.
    pub review_threshold: f64,
}

impl Curriculum {
    /// Create a new empty curriculum.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            lessons: Vec::new(),
            progression: ProgressionRule::default(),
            allow_review: true,
            review_threshold: 0.5,
        }
    }

    /// Add a lesson to the curriculum.
    pub fn add_lesson(&mut self, lesson: Lesson) {
        self.lessons.push(lesson);
    }

    /// Get the number of lessons.
    pub fn len(&self) -> usize {
        self.lessons.len()
    }

    /// Check if the curriculum is empty.
    pub fn is_empty(&self) -> bool {
        self.lessons.is_empty()
    }

    /// Get a lesson by index.
    pub fn lesson(&self, index: usize) -> Option<&Lesson> {
        self.lessons.get(index)
    }

    /// Get all lessons.
    pub fn lessons(&self) -> &[Lesson] {
        &self.lessons
    }

    /// Set the progression rule.
    pub fn with_progression(mut self, rule: ProgressionRule) -> Self {
        self.progression = rule;
        self
    }

    /// Enable or disable review.
    pub fn with_review(mut self, allow: bool) -> Self {
        self.allow_review = allow;
        self
    }

    /// Build a curriculum from a difficulty schedule, generating one lesson per step.
    pub fn from_schedule(name: &str, schedule: &DifficultySchedule) -> Self {
        let mut curriculum = Self::new(name);
        for i in 0..schedule.steps() {
            let difficulty = schedule.difficulty_at(i);
            let lesson = Lesson::new(i, difficulty);
            curriculum.add_lesson(lesson);
        }
        curriculum
    }
}

// ── MasteryTracker ──────────────────────────────────────────────────────────

/// Status of a single lesson in the mastery tracker.
#[derive(Debug, Clone, PartialEq)]
pub enum LessonStatus {
    /// Not yet attempted.
    NotStarted,
    /// Currently being worked on.
    InProgress { attempts: usize, best_score: f64 },
    /// Mastered (passed the success criteria).
    Mastered { attempts: usize, score: f64 },
    /// Failed (exhausted max attempts without mastery).
    Failed { attempts: usize, best_score: f64 },
}

impl LessonStatus {
    /// Is this lesson mastered?
    pub fn is_mastered(&self) -> bool {
        matches!(self, Self::Mastered { .. })
    }

    /// Is this lesson failed?
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Number of attempts so far.
    pub fn attempts(&self) -> usize {
        match self {
            Self::NotStarted => 0,
            Self::InProgress { attempts, .. }
            | Self::Mastered { attempts, .. }
            | Self::Failed { attempts, .. } => *attempts,
        }
    }

    /// Best score achieved.
    pub fn best_score(&self) -> f64 {
        match self {
            Self::NotStarted => 0.0,
            Self::InProgress { best_score, .. } | Self::Failed { best_score, .. } => *best_score,
            Self::Mastered { score, .. } => *score,
        }
    }
}

/// Tracks mastery status for all lessons in a curriculum.
#[derive(Debug, Clone)]
pub struct MasteryTracker {
    statuses: Vec<LessonStatus>,
}

impl MasteryTracker {
    /// Create a new tracker for `n` lessons.
    pub fn new(n: usize) -> Self {
        Self {
            statuses: vec![LessonStatus::NotStarted; n],
        }
    }

    /// Get the status of lesson `i`.
    pub fn status(&self, index: usize) -> &LessonStatus {
        &self.statuses[index]
    }

    /// Record an attempt on lesson `i` with the given score.
    /// Returns `true` if the lesson is now mastered.
    pub fn record_attempt(&mut self, index: usize, score: f64, lesson: &Lesson) -> bool {
        let current = &self.statuses[index];
        let attempts = current.attempts() + 1;
        let best_score = score.max(current.best_score());

        let mastered = lesson.is_passed(score) && attempts <= lesson.max_attempts;

        if mastered {
            self.statuses[index] = LessonStatus::Mastered { attempts, score };
        } else if attempts >= lesson.max_attempts {
            self.statuses[index] = LessonStatus::Failed { attempts, best_score };
        } else {
            self.statuses[index] = LessonStatus::InProgress { attempts, best_score };
        }

        mastered
    }

    /// Number of mastered lessons.
    pub fn mastered_count(&self) -> usize {
        self.statuses.iter().filter(|s| s.is_mastered()).count()
    }

    /// Number of failed lessons.
    pub fn failed_count(&self) -> usize {
        self.statuses.iter().filter(|s| s.is_failed()).count()
    }

    /// Mastery rate (0.0–1.0).
    pub fn mastery_rate(&self) -> f64 {
        if self.statuses.is_empty() {
            return 0.0;
        }
        self.mastered_count() as f64 / self.statuses.len() as f64
    }

    /// Lessons that need review (mastered but with low scores, or failed).
    pub fn lessons_for_review(&self, threshold: f64) -> Vec<usize> {
        self.statuses
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match s {
                LessonStatus::Mastered { score, .. } if *score < threshold + 0.1 => Some(i),
                LessonStatus::Failed { .. } => Some(i),
                _ => None,
            })
            .collect()
    }

    /// Total attempts across all lessons.
    pub fn total_attempts(&self) -> usize {
        self.statuses.iter().map(|s| s.attempts()).sum()
    }

    /// Best scores across all lessons.
    pub fn best_scores(&self) -> Vec<f64> {
        self.statuses.iter().map(|s| s.best_score()).collect()
    }

    /// Average score across all lessons.
    pub fn average_score(&self) -> f64 {
        if self.statuses.is_empty() {
            return 0.0;
        }
        self.best_scores().iter().sum::<f64>() / self.statuses.len() as f64
    }

    /// All lesson statuses.
    pub fn statuses(&self) -> &[LessonStatus] {
        &self.statuses
    }
}

// ── LessonState ─────────────────────────────────────────────────────────────

/// Mutable state for a lesson being trained.
#[derive(Debug, Clone)]
pub struct LessonState {
    /// Current attempt number (0-indexed).
    pub attempt: usize,
    /// Scores from each attempt.
    pub scores: Vec<f64>,
    /// Whether this lesson has been passed.
    pub passed: bool,
}

impl LessonState {
    /// Create a fresh state for a lesson.
    pub fn new() -> Self {
        Self {
            attempt: 0,
            scores: Vec::new(),
            passed: false,
        }
    }

    /// Record an attempt score.
    pub fn record(&mut self, score: f64) {
        self.scores.push(score);
        self.attempt += 1;
    }

    /// Best score so far.
    pub fn best_score(&self) -> f64 {
        self.scores.iter().copied().fold(0.0_f64, f64::max)
    }

    /// Average score so far.
    pub fn average_score(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        self.scores.iter().sum::<f64>() / self.scores.len() as f64
    }

    /// Last `n` consecutive pass results.
    pub fn consecutive_passes(&self, threshold: f64) -> usize {
        let mut count = 0;
        for &score in self.scores.iter().rev() {
            if score >= threshold {
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

// ── CurriculumResult ────────────────────────────────────────────────────────

/// Structured result of curriculum training.
#[derive(Debug, Clone)]
pub struct CurriculumResult {
    /// Name of the curriculum.
    pub curriculum_name: String,
    /// Number of lessons completed (passed or failed after max attempts).
    pub lessons_completed: usize,
    /// Total number of lessons.
    pub total_lessons: usize,
    /// Mastery rate (0.0–1.0).
    pub mastery_rate: f64,
    /// Total time spent training.
    pub duration: Duration,
    /// Per-lesson states.
    pub lesson_states: Vec<LessonState>,
    /// Mastery tracker snapshot.
    pub mastery: MasteryTracker,
}

impl CurriculumResult {
    /// Number of lessons completed.
    pub fn lessons_completed(&self) -> usize {
        self.lessons_completed
    }

    /// Total lessons.
    pub fn total_lessons(&self) -> usize {
        self.total_lessons
    }

    /// Mastery rate.
    pub fn mastery_rate(&self) -> f64 {
        self.mastery_rate
    }

    /// Duration.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Whether the entire curriculum was mastered.
    pub fn fully_mastered(&self) -> bool {
        self.lessons_completed == self.total_lessons && self.mastery_rate >= 1.0
    }

    /// Average score across all lessons.
    pub fn average_score(&self) -> f64 {
        let total: f64 = self.lesson_states.iter().map(|s| s.best_score()).sum();
        if self.lesson_states.is_empty() {
            0.0
        } else {
            total / self.lesson_states.len() as f64
        }
    }

    /// Format a summary string.
    pub fn summary(&self) -> String {
        format!(
            "Curriculum '{}': {}/{} lessons completed, {:.1}% mastery, {:?} spent",
            self.curriculum_name,
            self.lessons_completed,
            self.total_lessons,
            self.mastery_rate * 100.0,
            self.duration
        )
    }
}

// ── CurriculumTrainer ───────────────────────────────────────────────────────

/// Trains an agent through a curriculum, tracking progress.
pub struct CurriculumTrainer {
    curriculum: Curriculum,
}

impl CurriculumTrainer {
    /// Create a new trainer for the given curriculum.
    pub fn new(curriculum: Curriculum) -> Self {
        Self { curriculum }
    }

    /// Train through the curriculum using the provided training function.
    ///
    /// The training function receives the current `Lesson` and `LessonState`,
    /// and should return the score for that attempt (0.0–1.0).
    pub fn train<F>(&mut self, mut train_fn: F) -> CurriculumResult
    where
        F: FnMut(&Lesson, &LessonState) -> f64,
    {
        let start = Instant::now();
        let n = self.curriculum.len();
        let mut tracker = MasteryTracker::new(n);
        let mut lesson_states: Vec<LessonState> = (0..n).map(|_| LessonState::new()).collect();
        let mut completed = 0;

        for (i, lesson) in self.curriculum.lessons().iter().enumerate() {
            let state = &mut lesson_states[i];

            loop {
                let score = train_fn(lesson, state);
                state.record(score);

                let mastered = tracker.record_attempt(i, score, lesson);
                if mastered {
                    state.passed = true;
                    completed += 1;
                    break;
                }

                // Check if we've exhausted attempts
                if state.attempt >= lesson.max_attempts {
                    completed += 1;
                    break;
                }
            }
        }

        let duration = start.elapsed();
        let mastery_rate = tracker.mastery_rate();

        CurriculumResult {
            curriculum_name: self.curriculum.name.clone(),
            lessons_completed: completed,
            total_lessons: n,
            mastery_rate,
            duration,
            lesson_states,
            mastery: tracker,
        }
    }

    /// Train with a schedule, generating lessons dynamically.
    pub fn train_with_schedule<F>(
        &mut self,
        schedule: &mut DifficultySchedule,
        mut train_fn: F,
    ) -> CurriculumResult
    where
        F: FnMut(&Lesson, &LessonState) -> f64,
    {
        let start = Instant::now();
        let n = schedule.steps();
        let mut tracker = MasteryTracker::new(n);
        let mut lesson_states: Vec<LessonState> = (0..n).map(|_| LessonState::new()).collect();
        let mut completed = 0;

        for i in 0..n {
            let difficulty = schedule.difficulty_at(i);
            let lesson = Lesson::new(i, difficulty);
            let state = &mut lesson_states[i];

            loop {
                let score = train_fn(&lesson, state);
                state.record(score);

                let mastered = tracker.record_attempt(i, score, &lesson);

                // Update adaptive schedule
                schedule.update_mastery(tracker.mastery_rate());

                if mastered {
                    state.passed = true;
                    completed += 1;
                    break;
                }

                if state.attempt >= lesson.max_attempts {
                    completed += 1;
                    break;
                }
            }
        }

        let duration = start.elapsed();

        CurriculumResult {
            curriculum_name: self.curriculum.name.clone(),
            lessons_completed: completed,
            total_lessons: n,
            mastery_rate: tracker.mastery_rate(),
            duration,
            lesson_states,
            mastery: tracker,
        }
    }

    /// Get a reference to the curriculum.
    pub fn curriculum(&self) -> &Curriculum {
        &self.curriculum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DifficultySchedule tests ──

    #[test]
    fn test_linear_schedule_start() {
        let s = DifficultySchedule::linear(0.1, 1.0, 10);
        assert!((s.difficulty_at(0) - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_linear_schedule_end() {
        let s = DifficultySchedule::linear(0.1, 1.0, 10);
        assert!((s.difficulty_at(10) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_linear_schedule_midpoint() {
        let s = DifficultySchedule::linear(0.0, 1.0, 10);
        assert!((s.difficulty_at(5) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_exponential_schedule_start() {
        let s = DifficultySchedule::exponential(1.0, 100.0, 4);
        assert!((s.difficulty_at(0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_exponential_schedule_growth() {
        let s = DifficultySchedule::exponential(1.0, 100.0, 4);
        let d2 = s.difficulty_at(2);
        assert!(d2 > 1.0 && d2 < 100.0);
    }

    #[test]
    fn test_adaptive_schedule_updates() {
        let mut s = DifficultySchedule::adaptive(0.1, 1.0, 10);
        let initial = s.difficulty_at(0);
        s.update_mastery(0.9); // high mastery → accelerate
        // Current should have moved
        assert!(s.difficulty_at(0) >= initial);
    }

    #[test]
    fn test_schedule_steps() {
        let s = DifficultySchedule::linear(0.0, 1.0, 42);
        assert_eq!(s.steps(), 42);
    }

    #[test]
    fn test_schedule_default() {
        let s = DifficultySchedule::default();
        assert_eq!(s.steps(), 10);
    }

    // ── Lesson tests ──

    #[test]
    fn test_lesson_creation() {
        let l = Lesson::new(3, 0.5);
        assert_eq!(l.index, 3);
        assert!((l.difficulty - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_lesson_pass() {
        let l = Lesson::new(0, 0.5).with_success_threshold(0.7);
        assert!(l.is_passed(0.8));
        assert!(!l.is_passed(0.6));
    }

    #[test]
    fn test_lesson_env_config() {
        let l = Lesson::new(0, 0.5)
            .with_env_config("noise", "0.1")
            .with_env_config("horizon", "10");
        assert_eq!(l.env("noise"), Some("0.1"));
        assert_eq!(l.env("horizon"), Some("10"));
        assert_eq!(l.env("missing"), None);
    }

    // ── Curriculum tests ──

    #[test]
    fn test_curriculum_from_schedule() {
        let schedule = DifficultySchedule::linear(0.1, 1.0, 5);
        let c = Curriculum::from_schedule("test", &schedule);
        assert_eq!(c.len(), 5);
    }

    #[test]
    fn test_curriculum_empty() {
        let c = Curriculum::new("empty");
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    // ── MasteryTracker tests ──

    #[test]
    fn test_tracker_initial_state() {
        let t = MasteryTracker::new(3);
        assert_eq!(t.mastered_count(), 0);
        assert!((t.mastery_rate()).abs() < 1e-9);
    }

    #[test]
    fn test_tracker_record_mastered() {
        let mut t = MasteryTracker::new(1);
        let lesson = Lesson::new(0, 0.5).with_success_threshold(0.6);
        let mastered = t.record_attempt(0, 0.8, &lesson);
        assert!(mastered);
        assert!(t.status(0).is_mastered());
    }

    #[test]
    fn test_tracker_record_failed() {
        let mut t = MasteryTracker::new(1);
        let lesson = Lesson::new(0, 0.5)
            .with_success_threshold(0.9)
            .with_max_attempts(2);
        t.record_attempt(0, 0.3, &lesson);
        let mastered = t.record_attempt(0, 0.4, &lesson);
        assert!(!mastered);
        assert!(t.status(0).is_failed());
    }

    #[test]
    fn test_tracker_mastery_rate() {
        let mut t = MasteryTracker::new(2);
        let lesson = Lesson::new(0, 0.5).with_success_threshold(0.5);
        t.record_attempt(0, 0.8, &lesson);
        assert!((t.mastery_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_tracker_lessons_for_review() {
        let mut t = MasteryTracker::new(2);
        let lesson = Lesson::new(0, 0.5)
            .with_success_threshold(0.6)
            .with_max_attempts(2);
        // Lesson 0: mastered with score just above threshold
        t.record_attempt(0, 0.65, &lesson);
        // Lesson 1: failed
        t.record_attempt(1, 0.3, &lesson);
        t.record_attempt(1, 0.4, &lesson);

        let review = t.lessons_for_review(0.6);
        assert!(review.contains(&0)); // low mastery score
        assert!(review.contains(&1)); // failed
    }

    // ── LessonState tests ──

    #[test]
    fn test_lesson_state_best_score() {
        let mut s = LessonState::new();
        s.record(0.3);
        s.record(0.7);
        s.record(0.5);
        assert!((s.best_score() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_lesson_state_average() {
        let mut s = LessonState::new();
        s.record(0.4);
        s.record(0.6);
        assert!((s.average_score() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_consecutive_passes() {
        let mut s = LessonState::new();
        s.record(0.3);
        s.record(0.8);
        s.record(0.9);
        s.record(0.85);
        assert_eq!(s.consecutive_passes(0.7), 3);
    }

    // ── CurriculumTrainer tests ──

    #[test]
    fn test_trainer_perfect_score() {
        let schedule = DifficultySchedule::linear(0.1, 1.0, 5);
        let curriculum = Curriculum::from_schedule("perfect", &schedule);
        let mut trainer = CurriculumTrainer::new(curriculum);
        let result = trainer.train(|_lesson, _state| 1.0);
        assert!(result.fully_mastered());
        assert_eq!(result.lessons_completed(), 5);
    }

    #[test]
    fn test_trainer_all_fail() {
        let schedule = DifficultySchedule::linear(0.1, 1.0, 3);
        let curriculum = Curriculum::from_schedule("fail", &schedule);
        let mut trainer = CurriculumTrainer::new(curriculum);
        let result = trainer.train(|_lesson, _state| 0.1);
        assert!(!result.fully_mastered());
        assert_eq!(result.lessons_completed(), 3);
    }

    #[test]
    fn test_curriculum_result_summary() {
        let result = CurriculumResult {
            curriculum_name: "test".to_string(),
            lessons_completed: 4,
            total_lessons: 5,
            mastery_rate: 0.8,
            duration: Duration::from_secs(10),
            lesson_states: vec![],
            mastery: MasteryTracker::new(0),
        };
        let summary = result.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("4/5"));
        assert!(summary.contains("80.0%"));
    }
}
