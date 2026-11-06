//! График задержки сервисов за сутки по 15-минутным корзинам истории.
use gpui_kit::*;
use z_core::health::{BUCKET_SECONDS, Bucket, DAY_BUCKETS};

const DAY: u64 = BUCKET_SECONDS * DAY_BUCKETS;
/// Нижняя граница шкалы, чтобы ровная линия 90 мс не растягивалась на всю высоту.
const MIN_SCALE_MS: f32 = 250.;
const HEIGHT: f32 = 128.;
/// Линии сетки, мс.
const GRID_MS: [u64; 2] = [100, 200];

/// Линия одного сервиса.
pub struct Series {
    pub color: Hsla,
    pub buckets: Vec<Bucket>,
}

/// `live` — цвет точки «сейчас» на конце линии; `None`, когда обход выключен.
/// `label` — цвет подписей сетки.
pub fn latency_chart(
    series: Vec<Series>,
    now: u64,
    live: Option<Hsla>,
    outage: Hsla,
    grid: Hsla,
    label: Hsla,
) -> impl IntoElement {
    let scale = series
        .iter()
        .flat_map(|s| s.buckets.iter().filter_map(Bucket::average_ms))
        .max()
        .map_or(MIN_SCALE_MS, |ms| (ms as f32 * 1.2).max(MIN_SCALE_MS));
    // Подписи сетки над линиями у левого края, текстом, а не на canvas.
    let labels = GRID_MS
        .into_iter()
        .filter(|ms| (*ms as f32) < scale)
        .map(move |ms| {
            div()
                .absolute()
                .left_0()
                .top(px(HEIGHT - ms as f32 / scale * HEIGHT - 16.))
                .text_xs()
                .font_family("Consolas")
                .text_color(label)
                .child(crate::i18n::tr_args(
                    "home-latency-ms",
                    &[("ms", ms.into())],
                ))
        });
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

            for line in &series {
                let mut path = PathBuilder::stroke(px(1.5));
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
                            path.line_to(point);
                        }
                        Some((start, from)) => {
                            let gap = Bounds::new(
                                at(x(start + BUCKET_SECONDS), 0.),
                                size(px(x(bucket.start) - x(start + BUCKET_SECONDS)), px(height)),
                            );
                            window.paint_quad(fill(gap, grid.opacity(0.35)));
                            bridges.move_to(from);
                            bridges.line_to(point);
                            path.move_to(point);
                        }
                        None => path.move_to(point),
                    }
                    previous = Some((bucket.start, point));
                    last = Some((bucket.start, point));
                }
                if let Ok(bridges) = bridges.build() {
                    window.paint_path(bridges, line.color.opacity(0.5));
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, line.color);
                }
                if let (Some(color), Some((bucket_start, center))) = (live, last)
                    && now.saturating_sub(bucket_start) < 2 * BUCKET_SECONDS
                {
                    let dot = Bounds::new(center - point(px(3.5), px(3.5)), size(px(7.), px(7.)));
                    let mut dot = fill(dot, color);
                    dot.corner_radii = px(3.5).into();
                    window.paint_quad(dot);
                }
            }
        },
    )
    .size_full();
    div()
        .relative()
        .w_full()
        .h(px(HEIGHT))
        .child(plot)
        .children(labels)
}

/// Есть ли в истории пропуски: время, когда проверок не было.
pub fn has_gaps(buckets: &[Bucket]) -> bool {
    buckets
        .windows(2)
        .any(|pair| pair[1].start - pair[0].start > BUCKET_SECONDS)
}
