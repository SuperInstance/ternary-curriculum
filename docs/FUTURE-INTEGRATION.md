# Future Integration: ternary-curriculum

## Current State
Provides curriculum learning for ternary agents: progressive training through increasingly difficult lessons with configurable `DifficultySchedule` (Linear, Exponential, Adaptive), lesson management with mastery tracking, and curriculum evaluation. The adaptive schedule adjusts difficulty based on mastery feedback — speeding up when mastery is high, slowing when it's low.

## Integration Opportunities

### With ternary-cell (Progressive Room Education)
ternary-cell grids can be trained via curriculum learning: start with simple environments (few cells, low noise, predictable patterns) and progressively increase complexity (more cells, higher noise, adversarial inputs). The `Adaptive` schedule is key — when a cell grid masters its current complexity level (low surprise, high prediction accuracy), difficulty increases. When surprise spikes (grid is struggling), difficulty plateaus or decreases.

### With ternary-memory (Curriculum Memory)
ternary-memory stores learning history. ternary-curriculum uses mastery tracking. Together: mastery is stored in `LongTermMemory` (stable learned patterns), lesson history in `EpisodicMemory` (significant learning milestones). When an agent re-enters a room, it resumes the curriculum from its last mastery checkpoint rather than starting over. `MemoryIndex` tags memories with curriculum level for efficient retrieval.

### With ternary-transfer (Curriculum Transfer)
A well-designed curriculum for Room A may help Room B if the rooms are similar. ternary-transfer moves curriculum schedules between rooms: `DifficultySchedule::Linear` parameters transfer directly, while `Adaptive` parameters (mastery_rate, acceleration) need adjustment via `WeightedBlend`. The curriculum becomes another form of transferable knowledge — not just what was learned, but how it was learned.

## Potential in Mature Systems
In room-as-codespace, PLATO acts as the curriculum designer for rooms. New rooms start with simple tasks (basic monitoring, low-resolution sensing) and progress to complex tasks (multi-modal fusion, adversarial defense). The `Adaptive` schedule responds to real-time room performance: fast-learning rooms (Jetson with GPU) accelerate; slow-learning rooms (ESP32 with limited compute) get more practice at each level. PLATO tracks curriculum progress across all rooms and identifies which rooms need additional lessons.

## Cross-Pollination Ideas
- **ternary-noise**: Noise-aware curriculum — increase noise injection as difficulty increases, training rooms to handle progressively noisier environments.
- **ternary-ensemble**: Ensemble curriculum — train multiple agents through the same curriculum, then combine them into an ensemble for superior final performance.
- **ternary-thermodynamics**: Thermodynamic curriculum — temperature schedule IS the difficulty schedule. Start hot (high exploration, low difficulty), cool down (exploitation, high difficulty).

## Dependencies for Next Steps
- Define `RoomCurriculum` with room-specific difficulty schedules and mastery tracking
- Add curriculum checkpoints to ternary-memory's long-term storage
- Implement curriculum transfer between rooms using ternary-transfer
- Build adaptive difficulty adjustment based on cell grid surprise levels
