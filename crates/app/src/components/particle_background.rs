//! Small monochrome canvas effect, driven only by its host's loading timer.
//! No assets, random-number dependency, global listeners or independent animation loop.
use gpui_kit::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const PARTICLE_COUNT: usize = 48;
const TRAIL_LIMIT: usize = 36;
const TRAIL_LIFETIME: Duration = Duration::from_millis(550);

#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
}

#[derive(Clone)]
struct TrailPoint {
    position: Point<Pixels>,
    created: Instant,
}

pub(crate) struct ParticleField {
    particles: Vec<Particle>,
    trail: VecDeque<TrailPoint>,
    pointer: Option<Point<Pixels>>,
    elapsed: f32,
    updated: Instant,
}

fn seed(index: usize, salt: u32) -> f32 {
    let value = (index as u32).wrapping_add(salt).wrapping_mul(747_796_405);
    let value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    ((value ^ (value >> 22)) & 0xffff) as f32 / 65536.
}

impl ParticleField {
    pub(crate) fn new() -> Self {
        Self {
            particles: (0..PARTICLE_COUNT)
                .map(|index| Particle {
                    x: seed(index, 17),
                    y: seed(index, 43),
                    vx: (seed(index, 89) - 0.5) * 0.009,
                    vy: (seed(index, 127) - 0.5) * 0.012,
                    radius: 0.8 + seed(index, 193) * 0.9,
                })
                .collect(),
            trail: VecDeque::with_capacity(TRAIL_LIMIT),
            pointer: None,
            elapsed: 0.,
            updated: Instant::now(),
        }
    }

    pub(crate) fn advance(&mut self, animate: bool) {
        let now = Instant::now();
        let delta = now.duration_since(self.updated).as_secs_f32().min(0.1);
        self.updated = now;
        if animate {
            self.elapsed += delta;
            for particle in &mut self.particles {
                particle.x = (particle.x + particle.vx * delta).rem_euclid(1.);
                particle.y = (particle.y + particle.vy * delta).rem_euclid(1.);
            }
            self.trail
                .retain(|point| now.duration_since(point.created) < TRAIL_LIFETIME);
        } else {
            self.pointer = None;
            self.trail.clear();
        }
    }

    pub(crate) fn pointer_moved(&mut self, position: Point<Pixels>) {
        self.pointer = Some(position);
        let now = Instant::now();
        let previous = self
            .trail
            .back()
            .map(|point| point.position)
            .unwrap_or(position);
        let dx = f32::from(position.x - previous.x);
        let dy = f32::from(position.y - previous.y);
        let distance = dx.hypot(dy);
        if distance < 2. && !self.trail.is_empty() {
            return;
        }
        // Interpolate fast movements without allocating an unbounded event trail.
        let steps = ((distance / 6.).ceil() as usize).clamp(1, 8);
        for step in 1..=steps {
            let ratio = step as f32 / steps as f32;
            self.trail.push_back(TrailPoint {
                position: point(previous.x + px(dx * ratio), previous.y + px(dy * ratio)),
                created: now,
            });
        }
        while self.trail.len() > TRAIL_LIMIT {
            self.trail.pop_front();
        }
    }

    pub(crate) fn pointer_left(&mut self) {
        self.pointer = None;
    }

    pub(crate) fn indicator_alpha(&self, index: usize) -> f32 {
        0.18 + 0.42 * ((self.elapsed * 4. - index as f32 * 0.8).sin() * 0.5 + 0.5)
    }

    pub(crate) fn clear(&mut self) {
        self.particles.clear();
        self.trail.clear();
        self.pointer = None;
    }

    /// `ink` is the theme's text color; dots use it at low opacity in both themes.
    pub(crate) fn canvas(&self, ink: Hsla) -> impl IntoElement + Styled + use<> {
        let particles = self.particles.clone();
        let trail = self.trail.clone();
        let pointer = self.pointer;
        canvas(
            |_, _, _| (),
            move |bounds, (), window, _| {
                let width = f32::from(bounds.size.width);
                let height = f32::from(bounds.size.height);
                let now = Instant::now();
                for particle in particles {
                    let mut x = particle.x * width;
                    let mut y = particle.y * height;
                    let mut proximity = 0.;
                    if let Some(pointer) = pointer {
                        let dx = x - f32::from(pointer.x - bounds.origin.x);
                        let dy = y - f32::from(pointer.y - bounds.origin.y);
                        let distance = dx.hypot(dy);
                        proximity = (1. - distance / 120.).clamp(0., 1.);
                        if distance > 0.1 {
                            x += dx / distance * proximity * 14.;
                            y += dy / distance * proximity * 14.;
                        }
                    }
                    // Keep the logo and status area quieter than the edges.
                    let quiet = ((x - width * 0.5).abs() / 190.)
                        .max((y - height * 0.5).abs() / 155.)
                        .clamp(0.2, 1.);
                    let alpha = (0.10 + proximity * 0.24) * quiet;
                    paint_dot(
                        window,
                        ink,
                        bounds.origin + point(px(x), px(y)),
                        particle.radius,
                        alpha,
                    );
                }
                for point in trail {
                    let age = now.duration_since(point.created).as_secs_f32()
                        / TRAIL_LIFETIME.as_secs_f32();
                    let fade = (1. - age).clamp(0., 1.);
                    paint_dot(
                        window,
                        ink,
                        point.position,
                        1. + fade * 1.4,
                        fade * fade * 0.22,
                    );
                }
            },
        )
    }
}

fn paint_dot(window: &mut Window, ink: Hsla, center: Point<Pixels>, radius: f32, alpha: f32) {
    let bounds = Bounds::new(
        center - point(px(radius), px(radius)),
        size(px(radius * 2.), px(radius * 2.)),
    );
    let mut dot = fill(bounds, ink.opacity(alpha));
    dot.corner_radii = px(radius).into();
    window.paint_quad(dot);
}
