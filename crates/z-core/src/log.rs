//! Журнал событий: последние строки в памяти для интерфейса и, если задан файл,
//! копия на диске. Без файла после падения приложения приложить к отчёту нечего.
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

const MAX_LINES: usize = 1000;
/// Размер, после которого файл уходит в `.1`, а запись начинается заново.
const MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Default)]
struct Inner {
    lines: Vec<String>,
    sink: Option<Sink>,
}

struct Sink {
    path: PathBuf,
    file: File,
    written: u64,
}

#[derive(Clone, Default)]
pub struct Log(Arc<Mutex<Inner>>);

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    /// Журнал, который дополнительно пишется в файл. Если файл открыть не удалось,
    /// журнал остаётся в памяти, а причина становится его первой строкой.
    pub fn to_file(path: &Path) -> Self {
        let log = Self::new();
        match Sink::open(path) {
            Ok(sink) => log.lock().sink = Some(sink),
            Err(error) => log.add(format!("Log file {}: {error}", path.display())),
        }
        log
    }

    /// Паника другого потока посреди записи не должна навсегда отключить журнал.
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn add(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();
        let mut inner = self.lock();
        inner.lines.push(msg.to_string());
        if inner.lines.len() > MAX_LINES {
            let extra = inner.lines.len() - MAX_LINES;
            inner.lines.drain(..extra);
        }
        // Диск мог заполниться или файл мог быть удалён: интерфейсу это мешать не должно.
        if let Some(sink) = &mut inner.sink
            && sink.write(msg).is_err()
        {
            inner.sink = None;
        }
    }

    pub fn get(&self) -> Vec<String> {
        self.lock().lines.clone()
    }

    /// Очищает список в интерфейсе; файл на диске остаётся для диагностики.
    pub fn clear(&self) {
        self.lock().lines.clear();
    }
}

impl Sink {
    fn open(path: &Path) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let written = file.metadata()?.len();
        Ok(Self {
            path: path.to_path_buf(),
            file,
            written,
        })
    }

    fn write(&mut self, msg: &str) -> std::io::Result<()> {
        if self.written >= MAX_FILE_BYTES {
            // Одна прошлая копия: журнал не растёт без предела, а недавняя история цела.
            fs::rename(&self.path, self.path.with_extension("log.1"))?;
            *self = Self::open(&self.path)?;
        }
        let line = format!("{} {msg}\n", timestamp(crate::clock::now()));
        self.file.write_all(line.as_bytes())?;
        self.written += line.len() as u64;
        Ok(())
    }
}

/// Дописывает строку в файл журнала в обход `Log`. Для обработчика паники:
/// паника могла случиться, пока журнал заблокирован.
pub fn append_line(path: &Path, msg: &str) {
    if let Ok(mut sink) = Sink::open(path) {
        let _ = sink.write(msg);
    }
}

/// Время UTC вида `2026-11-11 08:15:02Z`.
fn timestamp(unix: u64) -> String {
    let (days, seconds) = (unix / 86_400, unix % 86_400);
    // Перевод числа дней в дату григорианского календаря (алгоритм Говарда Хиннанта).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}Z",
        seconds / 3_600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zui-log-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("logs").join("zui.log")
    }

    #[test]
    fn timestamps_are_calendar_dates() {
        assert_eq!(timestamp(0), "1970-01-01 00:00:00Z");
        assert_eq!(timestamp(1_700_000_000), "2023-11-14 22:13:20Z");
        // 29 февраля високосного года.
        assert_eq!(timestamp(1_709_164_800), "2024-02-29 00:00:00Z");
    }

    #[test]
    fn lines_reach_the_file_and_survive_clearing() {
        let path = temp_file("write");
        let log = Log::to_file(&path);
        log.add("first");
        log.add("second");
        log.clear();
        assert!(log.get().is_empty());
        let saved = fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = saved.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("Z first"));
        assert!(lines[1].ends_with("Z second"));
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn full_file_is_rotated_once() {
        let path = temp_file("rotate");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![b'x'; MAX_FILE_BYTES as usize]).unwrap();
        let log = Log::to_file(&path);
        log.add("after rotation");
        assert_eq!(
            fs::metadata(path.with_extension("log.1")).unwrap().len(),
            MAX_FILE_BYTES
        );
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .ends_with("after rotation\n")
        );
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn unwritable_file_keeps_the_log_in_memory() {
        // Каталог вместо файла открыть на запись нельзя.
        let path = std::env::temp_dir();
        let log = Log::to_file(&path);
        log.add("still works");
        assert_eq!(log.get().last().unwrap(), "still works");
    }
}
