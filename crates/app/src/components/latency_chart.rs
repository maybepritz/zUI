//! График задержки сервисов: за сутки по 15-минутным корзинам истории и за последний
//! час по отдельным проверкам. Это единственное место интерфейса с цветом и свечением.
use gpui_kit::*;
use z_core::health::{BUCKET_SECONDS, Bucket, DAY_BUCKETS};

const DAY: u64 = BUCKET_SECONDS * DAY_BUCKETS;
/// Наибольшее окно режима «Час».
pub const HOUR: u64 = 60 * 60;
/// Проверки идут раз в минуту; более долгий перерыв между точками — пропуск, а не линия.
const LIVE_GAP: u64 = 150;
/// Нижняя граница шкалы, чтобы ровная линия 90 мс не растягивалась на всю высоту.
const MIN_SCALE_MS: f32 = 250.;
const HEIGHT: f32 = 148.;
/// Линии сетки, мс.
const GRID_MS: [u64; 2] = [100, 200];

/// Линия одного сервиса за сутки.
pub struct Series {
    pub color: Hsla,
    pub buckets: Vec<Bucket>,
}

/// Линия одного сервиса по отдельным проверкам: время и задержка, `None` — не открылся.
pub struct LiveSeries {
    pub color: Hsla,
    pub points: Vec<(u64, Option<u64>)>,
}

fn scale_for(values: impl Iterator<Item = u64>) -> f32 {
    values
        .max()
        .map_or(MIN_SCALE_MS, |ms| (ms as f32 * 1.2).max(MIN_SCALE_MS))
}

/// Подписи сетки над линиями у левого края, текстом, а не на canvas.
fn grid_labels(scale: f32, label: Hsla) -> impl Iterator<Item = Div> {
    GRID_MS
        .into_iter()
        .filter(move |ms| (*ms as f32) < scale)
        .map(move |ms| {
            div()
                .absolute()
                .left_0()
                .top(px(HEIGHT - ms as f32 / scale * HEIGHT - 16.))
                .text_xs()
                .font_family(crate::components::tone::MONO)
                .text_color(label)
                .child(crate::i18n::tr_args(
                    "home-latency-ms",
                    &[("ms", ms.into())],
                ))
        })
}

fn stroke(window: &mut Window, points: &[Point<Pixels>], width: f32, color: Hsla) {
    let [first, rest @ ..] = points else {
        return;
    };
    if rest.is_empty() {
        return;
    }
    let mut path = PathBuilder::stroke(px(width));
    path.move_to(*first);
    for point in rest {
        path.line_to(*point);
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

/// Линия с мягким ореолом: широкие полупрозрачные слои того же цвета под тонкой яркой.
/// `glow` от 0 до 1; на светлом фоне ореол не виден, поэтому там передаётся 0.
fn glowing_line(window: &mut Window, points: &[Point<Pixels>], color: Hsla, glow: f32) {
    if glow > 0. {
        stroke(window, points, 11., color.opacity(0.05 * glow));
        stroke(window, points, 6., color.opacity(0.12 * glow));
        stroke(window, points, 3.5, color.opacity(0.22 * glow));
    }
    stroke(window, points, 1.75, color);
}

fn dot(window: &mut Window, center: Point<Pixels>, radius: f32, color: Hsla) {
    let bounds = Bounds::new(
        center - point(px(radius), px(radius)),
        size(px(radius * 2.), px(radius * 2.)),
    );
    let mut quad = fill(bounds, color);
    quad.corner_radii = px(radius).into();
    window.paint_quad(quad);
}

/// Точка «сейчас» на конце линии; ореол вокруг неё — живой элемент графика.
fn live_dot(window: &mut Window, center: Point<Pixels>, color: Hsla, glow: f32) {
    if glow > 0. {
        dot(window, center, 9., color.opacity(0.10 * glow));
        dot(window, center, 6., color.opacity(0.20 * glow));
    }
    dot(window, center, 3.5, color);
}

/// Ведущая линия светится в полную силу, остальные тише: так читается, какая главная.
fn glow_of(index: usize, enabled: bool) -> f32 {
    match (enabled, index) {
        (false, _) => 0.,
        (true, 0) => 1.,
        (true, _) => 0.55,
    }
}

fn frame(plot: impl IntoElement, labels: impl Iterator<Item = Div>) -> Div {
    div()
        .relative()
        .w_full()
        .h(px(HEIGHT))
        .child(plot)
        .children(labels)
}

/// График за сутки. `live` — рисовать ли точку «сейчас»: только пока обход работает.
/// `label` — цвет подписей сетки, `glow` — рисовать ли ореол (тёмная тема).
pub fn latency_chart(
    series: Vec<Series>,
    now: u64,
    live: bool,
    outage: Hsla,
    grid: Hsla,
    label: Hsla,
    glow: bool,
) -> impl IntoElement {
    let scale = scale_for(
        series
            .iter()
            .flat_map(|s| s.buckets.iter().filter_map(Bucket::average_ms)),
    );
    let plot = canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            let start = now.saturating_sub(DAY);
            let x = |t: u64| t.saturating_sub(start) as f32 / DAY as f32 * width;
            let y = |ms: u64| height - ms as f32 / scale * height;
            let at = |px_x: f32, px_y: f32| bounds.origin + point(px(px_x), px(px_y));

            for ms in GRID_MS {
                if (ms as f32) < scale {
                    let line = Bounds::new(at(0., y(ms)), size(px(width), px(1.)));
                    window.paint_quad(fill(line, grid));
                }
            }

            // Полоса там, где сервис не открылся ни разу за 15 минут.
            for bucket in series.iter().flat_map(|s| &s.buckets) {
                if bucket.down() && bucket.start + BUCKET_SECONDS > start {
                    let left = x(bucket.start);
                    let band = Bounds::new(
                        at(left, 0.),
                        size(px(x(bucket.start + BUCKET_SECONDS) - left), px(height)),
                    );
                    window.paint_quad(fill(band, outage.opacity(0.14)));
                }
            }

            for (index, line) in series.iter().enumerate() {
                let mut segments: Vec<Vec<Point<Pixels>>> = Vec::new();
                // Пропуск, когда приложение было закрыто или обход выключен: данных нет,
                // поэтому соседние точки соединяет пунктир, а не сплошная линия.
                let mut bridges = PathBuilder::stroke(px(1.)).dash_array(&[px(3.), px(4.)]);
                let mut previous: Option<(u64, Point<Pixels>)> = None;
                let mut last = None;
                for bucket in &line.buckets {
                    let Some(ms) = bucket.average_ms() else {
                        // Сбой: линию рвём, полоса сбоя уже показывает, что случилось.
                        previous = None;
                        continue;
                    };
                    let point = at(x(bucket.start + BUCKET_SECONDS / 2), y(ms));
                    match previous {
                        Some((start, _))
                            if bucket.start.saturating_sub(start) == BUCKET_SECONDS =>
                        {
                            if let Some(segment) = segments.last_mut() {
                                segment.push(point);
                            }
                        }
                        Some((start, from)) => {
                            let gap = Bounds::new(
                                at(x(start + BUCKET_SECONDS), 0.),
                                size(px(x(bucket.start) - x(start + BUCKET_SECONDS)), px(height)),
                            );
                            window.paint_quad(fill(gap, grid.opacity(0.35)));
                            bridges.move_to(from);
                            bridges.line_to(point);
                            segments.push(vec![point]);
                        }
                        None => segments.push(vec![point]),
                    }
                    previous = Some((bucket.start, point));
                    last = Some((bucket.start, point));
                }
                if let Ok(bridges) = bridges.build() {
                    window.paint_path(bridges, line.color.opacity(0.5));
                }
                let glow = glow_of(index, glow);
                for segment in &segments {
                    glowing_line(window, segment, line.color, glow);
                }
                if let Some((bucket_start, center)) = last
                    && live
                    && now.saturating_sub(bucket_start) < 2 * BUCKET_SECONDS
                {
                    live_dot(window, center, line.color, glow);
                }
            }
        },
    )
    .size_full();
    frame(plot, grid_labels(scale, label))
}

/// График по отдельным проверкам за последние `span` секунд (не больше часа):
/// одна точка на проверку, новая появляется справа.
pub fn live_chart(
    series: Vec<LiveSeries>,
    now: u64,
    span: u64,
    outage: Hsla,
    grid: Hsla,
    label: Hsla,
    glow: bool,
) -> impl IntoElement {
    let scale = scale_for(
        series
            .iter()
            .flat_map(|s| s.points.iter().filter_map(|(_, ms)| *ms)),
    );
    let plot = canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            let start = now.saturating_sub(span);
            let x = |t: u64| t.saturating_sub(start) as f32 / span as f32 * width;
            let y = |ms: u64| height - ms as f32 / scale * height;
            let at = |px_x: f32, px_y: f32| bounds.origin + point(px(px_x), px(px_y));

            for ms in GRID_MS {
                if (ms as f32) < scale {
                    let line = Bounds::new(at(0., y(ms)), size(px(width), px(1.)));
                    window.paint_quad(fill(line, grid));
                }
            }

            for (index, line) in series.iter().enumerate() {
                let mut segments: Vec<Vec<Point<Pixels>>> = Vec::new();
                let mut previous: Option<u64> = None;
                let mut last = None;
                for &(time, ms) in line.points.iter().filter(|(time, _)| *time >= start) {
                    let Some(ms) = ms else {
                        // Проверка не прошла: полоса шириной в один интервал и разрыв линии.
                        let band = Bounds::new(
                            at(x(time.saturating_sub(30)), 0.),
                            size(px(60. / span as f32 * width), px(height)),
                        );
                        window.paint_quad(fill(band, outage.opacity(0.14)));
                        previous = None;
                        continue;
                    };
                    let point = at(x(time), y(ms));
                    match (previous, segments.last_mut()) {
                        (Some(before), Some(segment))
                            if time.saturating_sub(before) <= LIVE_GAP =>
                        {
                            segment.push(point);
                        }
                        _ => segments.push(vec![point]),
                    }
                    previous = Some(time);
                    last = Some((time, point));
                }
                let glow = glow_of(index, glow);
                for segment in &segments {
                    match segment.as_slice() {
                        // Одиночная проверка: линию провести не из чего, показываем точку.
                        [single] => dot(window, *single, 2., line.color),
                        points => glowing_line(window, points, line.color, glow),
                    }
                }
                if let Some((time, center)) = last
                    && now.saturating_sub(time) < LIVE_GAP
                {
                    live_dot(window, center, line.color, glow);
                }
            }
        },
    )
    .size_full();
    frame(plot, grid_labels(scale, label))
}

/// Есть ли в истории пропуски: время, когда проверок не было.
pub fn has_gaps(buckets: &[Bucket]) -> bool {
    buckets
        .windows(2)
        .any(|pair| pair[1].start - pair[0].start > BUCKET_SECONDS)
}
