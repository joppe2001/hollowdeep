//! Animation system for sprite-based entities
//!
//! Handles animation states, frame cycling, and sprite selection.

use std::collections::HashMap;
use std::path::Path;
use image::DynamicImage;

/// Animation states for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    Attack,
    Hurt,
    Death,
}

impl AnimationState {
    pub fn name(&self) -> &'static str {
        match self {
            AnimationState::Idle => "Idle",
            AnimationState::Walk => "Walk",
            AnimationState::Attack => "Attack",
            AnimationState::Hurt => "Hurt",
            AnimationState::Death => "Death",
        }
    }
}

/// Direction the entity is facing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Facing {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

impl Facing {
    /// Get facing direction from movement delta
    pub fn from_delta(dx: i32, dy: i32) -> Self {
        // Prioritize vertical for diagonal movement
        if dy < 0 {
            Facing::Up
        } else if dy > 0 {
            Facing::Down
        } else if dx < 0 {
            Facing::Left
        } else if dx > 0 {
            Facing::Right
        } else {
            Facing::Down // Default
        }
    }
}

/// Frames for a single animation
#[derive(Clone)]
pub struct AnimationFrames {
    pub frames: Vec<DynamicImage>,
    /// Duration per frame in seconds
    pub frame_duration: f32,
    /// Whether to loop the animation
    pub looping: bool,
}

impl AnimationFrames {
    pub fn new(frames: Vec<DynamicImage>, frame_duration: f32, looping: bool) -> Self {
        Self {
            frames,
            frame_duration,
            looping,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn get_frame(&self, index: usize) -> Option<&DynamicImage> {
        self.frames.get(index)
    }
}

/// Complete sprite set for an entity (all animations, all directions)
/// For now, we only support single-direction sprites (facing camera)
pub struct EntitySprites {
    /// Animation name -> frames
    pub animations: HashMap<AnimationState, AnimationFrames>,
    /// Sprite dimensions
    pub width: u32,
    pub height: u32,
    /// Kitty image IDs for uploaded frames (animation_state, frame_index) -> kitty_id
    pub kitty_ids: HashMap<(AnimationState, usize), u32>,
}

impl EntitySprites {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            animations: HashMap::new(),
            width,
            height,
            kitty_ids: HashMap::new(),
        }
    }

    pub fn add_animation(&mut self, state: AnimationState, frames: AnimationFrames) {
        self.animations.insert(state, frames);
    }

    pub fn get_animation(&self, state: AnimationState) -> Option<&AnimationFrames> {
        self.animations.get(&state)
    }

    pub fn get_frame(&self, state: AnimationState, frame_index: usize) -> Option<&DynamicImage> {
        self.animations.get(&state)?.get_frame(frame_index)
    }

    /// Store a Kitty image ID for a frame
    pub fn set_kitty_id(&mut self, state: AnimationState, frame_index: usize, kitty_id: u32) {
        self.kitty_ids.insert((state, frame_index), kitty_id);
    }

    /// Get the Kitty image ID for a frame
    pub fn get_kitty_id(&self, state: AnimationState, frame_index: usize) -> Option<u32> {
        self.kitty_ids.get(&(state, frame_index)).copied()
    }
}

/// Animation controller - tracks current state and frame timing
#[derive(Debug, Clone)]
pub struct AnimationController {
    pub current_state: AnimationState,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub facing: Facing,
    /// Set to true when animation completes (for non-looping)
    pub finished: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_state: AnimationState::Idle,
            current_frame: 0,
            frame_timer: 0.0,
            facing: Facing::Down,
            finished: false,
        }
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update animation timing
    pub fn update(&mut self, delta_seconds: f32, animation: &AnimationFrames) {
        if self.finished && !animation.looping {
            return;
        }

        self.frame_timer += delta_seconds;

        if self.frame_timer >= animation.frame_duration {
            self.frame_timer -= animation.frame_duration;
            self.current_frame += 1;

            if self.current_frame >= animation.frame_count() {
                if animation.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = animation.frame_count().saturating_sub(1);
                    self.finished = true;
                }
            }
        }
    }

    /// Switch to a new animation state
    pub fn set_state(&mut self, state: AnimationState) {
        if self.current_state != state {
            self.current_state = state;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.finished = false;
        }
    }

    /// Set facing direction
    pub fn set_facing(&mut self, facing: Facing) {
        self.facing = facing;
    }

    /// Set facing from movement delta
    pub fn set_facing_from_delta(&mut self, dx: i32, dy: i32) {
        if dx != 0 || dy != 0 {
            self.facing = Facing::from_delta(dx, dy);
        }
    }

    /// Advance one frame (for turn-based movement)
    pub fn advance_frame(&mut self, frame_count: usize, looping: bool) {
        self.current_frame += 1;
        if self.current_frame >= frame_count {
            if looping {
                self.current_frame = 0;
            } else {
                self.current_frame = frame_count.saturating_sub(1);
                self.finished = true;
            }
        }
    }
}

/// Load player sprites from the assets directory
pub fn load_player_sprites<P: AsRef<Path>>(base_path: P) -> Result<EntitySprites, image::ImageError> {
    let base = base_path.as_ref();
    let mut sprites = EntitySprites::new(128, 128);

    // Load idle sprite (single frame)
    let idle_path = base.join("rogue.png");
    if idle_path.exists() {
        let idle_img = image::open(&idle_path)?;
        sprites.add_animation(
            AnimationState::Idle,
            AnimationFrames::new(vec![idle_img], 1.0, true),
        );
        log::info!("Loaded idle sprite from {:?}", idle_path);
    }

    // Load walk animation
    let walk_frames = load_animation_frames(base, "Walk", "walk", 2)?;
    if !walk_frames.is_empty() {
        sprites.add_animation(
            AnimationState::Walk,
            AnimationFrames::new(walk_frames, 0.15, true),
        );
        log::info!("Loaded walk animation");
    }

    // Load attack animation
    let attack_frames = load_animation_frames(base, "Attack", "Attack", 3)?;
    if !attack_frames.is_empty() {
        sprites.add_animation(
            AnimationState::Attack,
            AnimationFrames::new(attack_frames, 0.1, false),
        );
        log::info!("Loaded attack animation");
    }

    // Load hurt animation
    let hurt_frames = load_animation_frames(base, "Hurt", "hurt", 4)?;
    if !hurt_frames.is_empty() {
        sprites.add_animation(
            AnimationState::Hurt,
            AnimationFrames::new(hurt_frames, 0.1, false),
        );
        log::info!("Loaded hurt animation");
    }

    Ok(sprites)
}

/// Load numbered animation frames from a subdirectory
fn load_animation_frames<P: AsRef<Path>>(
    base: P,
    subdir: &str,
    prefix: &str,
    max_frames: usize,
) -> Result<Vec<DynamicImage>, image::ImageError> {
    let dir = base.as_ref().join(subdir);
    let mut frames = Vec::new();

    for i in 1..=max_frames {
        let filename = format!("{}{}.png", prefix, i);
        let path = dir.join(&filename);

        if path.exists() {
            let img = image::open(&path)?;
            frames.push(img);
            log::debug!("Loaded animation frame: {:?}", path);
        }
    }

    Ok(frames)
}
