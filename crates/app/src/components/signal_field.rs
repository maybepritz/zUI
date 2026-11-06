//! Фон экрана запуска: ленты из символов приходят с краёв окна, сходятся в логотипе
//! и расходятся дальше. Весь трафик идёт через zUI. Курсор притягивает ленты.
//! Без ресурсов и своего цикла анимации: время двигает таймер экрана загрузки.
use gpui_kit::*;
use std::time::Instant;

/// Символы от тихого к плотному; пробел — пустая клетка.
const CHARS: [&str; 9] = [" ", ".", "·", ":", "-", "=", "+", "x", "#"];
const CELL_W: f32 = 7.;
const CELL_H: f32 = 12.;
const LINES: usize = 5;
/// Логотип стоит на этой доле высоты: туда сходятся ленты.
pub(crate) const ANCHOR: f32 = 0.4;

pub(crate) struct SignalField {
    time: f32,
    updated: Instant,
    pointer: Option<Point<Pixels>>,
    /// Сглаженная позиция курсора и сила притяжения: ленты не прыгают, а догоняют.
    follow: Point<Pixels>,
    pull: f32,
    speed: (f32, f32),
    dim: (f32, f32),
    flash: f32,
}

/// Плавно приближает `current` к `goal` с постоянной времени `tau` секунд.
fn ease(current: f32, goal: f32, delta: f32, tau: f32) -> f32 {
    current + (goal - current) * (1. - (-delta / tau).exp())
}

impl SignalField {
    pub(crate) fn new() -> Self {
        Self {
            time: 0.,
            updated: Instant::now(),
            pointer: None,
            follow: point(px(0.), px(0.)),
            pull: 0.,
            speed: (1., 1.),
            dim: (1., 1.),
            flash: 0.,
        }
    }

    /// `animate` — окно активно и в системе не отключена анимация.
    pub(crate) fn advance(&mut self, animate: bool) {
        let now = Instant::now();
        let delta = now.duration_since(self.updated).as_secs_f32().min(0.2);
        self.updated = now;
        if !animate {
            self.pointer = None;
            self.pull = 0.;
            return;
        }
        self.pull = ease(
            self.pull,
            if self.pointer.is_some() { 1. } else { 0. },
            delta,
            0.4,
        );
        if let Some(pointer) = self.pointer {
            self.follow.x = px(ease(
                f32::from(self.follow.x),
                f32::from(pointer.x),
                delta,
                0.15,
            ));
            self.follow.y = px(ease(
                f32::from(self.follow.y),
                f32::from(pointer.y),
                delta,
                0.15,
            ));
        }
        self.speed.0 = ease(self.speed.0, self.speed.1, delta, 0.6);
        self.dim.0 = ease(self.dim.0, self.dim.1, delta, 0.6);
        self.flash = (self.flash - delta).max(0.);
        self.time += delta * self.speed.0;
    }

    pub(crate) fn pointer_moved(&mut self, position: Point<Pixels>) {
        // Курсор вошёл заново: начинаем притяжение с его места, а не издалека.
        if self.pointer.is_none() && self.pull < 0.05 {
            self.follow = position;
        }
        self.pointer = Some(position);
    }

    pub(crate) fn pointer_left(&mut self) {
        self.pointer = None;
    }

    /// Экран ждёт ответа пользователя: ленты замедляются и тускнеют.
    pub(crate) fn set_waiting(&mut self, waiting: bool) {
        self.speed.1 = if waiting { 0.3 } else { 1. };
        self.dim.1 = if waiting { 0.7 } else { 1. };
    }

    /// Короткая вспышка акцентом, когда пришёл результат.
    pub(crate) fn flash(&mut self) {
        self.flash = 0.7;
    }

    /// `ink` — цвет текста темы, `accent` — цвет вспышки.
    pub(crate) fn canvas(&self, ink: Hsla, accent: Hsla) -> impl IntoElement + Styled + use<> {
        let (time, pull, follow, dim) = (self.time, self.pull, self.follow, self.dim.0);
        let flashing = self.flash > 0.;
        canvas(
            move |_, window, _| {
                // Каждый уровень плотности — один символ своей яркости: формируем их
                // один раз на кадр и дальше только расставляем.
                let font_size = px(11.);
                let run = |text: &str, color: Hsla| TextRun {
                    len: text.len(),
                    font: font("Consolas"),
                    color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let shape = |level: usize, color: Hsla| {
                    let text = CHARS[level];
                    let alpha = 0.18 + 0.62 * level as f32 / (CHARS.len() - 1) as f32;
                    window.text_system().shape_line(
                        text.into(),
                        font_size,
                        &[run(text, color.opacity(alpha))],
                        None,
                    )
                };
                let plain: Vec<ShapedLine> = (0..CHARS.len()).map(|l| shape(l, ink)).collect();
                let bright: Vec<ShapedLine> = (0..CHARS.len()).map(|l| shape(l, accent)).collect();
                (plain, bright)
            },
            move |bounds, (plain, bright), window, cx| {
                let width = f32::from(bounds.size.width);
                let height = f32::from(bounds.size.height);
                let (cx_, cy) = (width / 2., height * ANCHOR + 40.);
                let half = width / 2.;
                let pointer = point(
                    f32::from(follow.x - bounds.origin.x),
                    f32::from(follow.y - bounds.origin.y),
                );
                let columns = (width / CELL_W).ceil() as usize;
                let rows = (height / CELL_H).ceil() as usize;
                for column in 0..columns {
                    let x = column as f32 * CELL_W + CELL_W / 2.;
                    let spread = ((x - cx_).abs() / half).min(1.).powf(1.5);
                    let side = if x < cx_ { -1. } else { 1. };
                    // Центры лент на этой вертикали не зависят от строки: считаем один раз.
                    let lanes: [(f32, f32, f32); LINES] = std::array::from_fn(|k| {
                        let lane = (k as f32 - (LINES - 1) as f32 / 2.) * 58.;
                        let mut center = cy
                            + spread
                                * (lane + 44. * (x * 0.008 + time * 1.1 + k as f32 * 1.3).sin());
                        if pull > 0.001 {
                            let near = (-((x - pointer.x) / 110.).powi(2)).exp();
                            center += (pointer.y - center) * 0.35 * pull * near;
                        }
                        let band = 7. + spread * 16.;
                        // Уплотнения бегут вдоль ленты к логотипу: это пакеты.
                        let pulse =
                            0.8 + 0.2 * ((x - cx_) * 0.025 * -side + time * 4. + k as f32).sin();
                        (center, band, pulse)
                    });
                    for row in 0..rows {
                        let y = row as f32 * CELL_H + CELL_H / 2.;
                        let mut value = 0.;
                        for (center, band, pulse) in lanes {
                            let d = (y - center) / band;
                            value += (-d * d).exp() * (0.3 + 0.7 * spread) * pulse * 0.75;
                        }
                        // Под подсказкой внизу тихо, с плавным краем.
                        let foot = ((height - 24. - y) / 40.).clamp(0., 1.);
                        value *= 0.18 + 0.82 * foot * foot * (3. - 2. * foot);
                        value *= dim;
                        let level = ((value * CHARS.len() as f32) as usize).min(CHARS.len() - 1);
                        if level == 0 {
                            continue;
                        }
                        let glyphs = if flashing && level >= CHARS.len() - 3 {
                            &bright
                        } else {
                            &plain
                        };
                        let origin =
                            bounds.origin + point(px(x - CELL_W / 2.), px(y - CELL_H / 2.));
                        let _ = glyphs[level].paint(
                            origin,
                            px(CELL_H),
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    }
                }
            },
        )
    }
}
