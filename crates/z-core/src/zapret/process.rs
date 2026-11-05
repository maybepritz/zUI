//! Владеет одним процессом winws и перекладывает его вывод в лог приложения.
use super::strategies::{ensure_user_lists, validate_args};
use crate::log::Log;
use anyhow::{Context as _, Result, bail};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
};

pub struct Zapret {
    child: Option<Child>,
    pub pid: Option<u32>,
    log: Log,
}

impl Zapret {
    pub fn new(log: Log) -> Self {
        Self {
            child: None,
            pid: None,
            log,
        }
    }

    /// Запускает winws с проверенными файлами стратегии и аргументами Windows как есть.
    ///
    /// Нужны права администратора; прежний свой процесс останавливается. При ошибке
    /// процесс, который не удалось остановить, остаётся во владении этого экземпляра.
    pub fn start(&mut self, bin_dir: &Path, lists_dir: &Path, raw_args: &str) -> Result<u32> {
        if !crate::dns::is_elevated() {
            bail!(
                "Для запуска WinDivert нужны права администратора. Перезапустите приложение от имени администратора."
            );
        }
        ensure_user_lists(lists_dir)?;

        let exe_path = bin_dir.join("winws.exe");
        if !exe_path.exists() {
            bail!("winws.exe не найден по пути: {}", exe_path.display());
        }
        validate_args(raw_args)?;
        for dependency in ["WinDivert.dll", "WinDivert64.sys", "cygwin1.dll"] {
            if !bin_dir.join(dependency).is_file() {
                bail!("Отсутствует {dependency} в {}", bin_dir.display());
            }
        }
        self.stop();
        if self.child.is_some() {
            bail!("Предыдущий процесс не остановлен");
        }

        self.log
            .add(format!("Запуск winws.exe из: {}", bin_dir.display()));

        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(bin_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            cmd.creation_flags(crate::CREATE_NO_WINDOW);
            cmd.raw_arg(raw_args);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Не удалось запустить {}", exe_path.display()))?;

        let pid = child.id();
        self.pid = Some(pid);
        self.log.add(format!("Процесс winws запущен (PID: {pid})"));

        // Читатели вывода из stdout и stderr
        if let Some(stdout) = child.stdout.take() {
            pipe_output(stdout, "out", self.log.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            pipe_output(stderr, "err", self.log.clone());
        }

        self.child = Some(child);
        Ok(pid)
    }

    /// Останавливает свой процесс и дожидается его; если завершить не удалось, владение сохраняется.
    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let pid = self.pid.unwrap_or(0);
            if let Err(error) = c.kill()
                && !matches!(c.try_wait(), Ok(Some(_)))
            {
                self.log.add(format!("Ошибка остановки PID {pid}: {error}"));
                self.child = Some(c);
                return;
            }
            let _ = c.wait();
            self.pid = None;
            self.log.add(format!("Процесс (PID: {pid}) остановлен"));
        }
    }

    /// Проверяет, завершился ли процесс, и после этого очищает дескриптор и PID.
    pub fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(c) => match c.try_wait() {
                Ok(None) => true,
                Ok(Some(status)) => {
                    self.log
                        .add(format!("Процесс завершился с кодом: {status}"));
                    self.child = None;
                    self.pid = None;
                    false
                }
                Err(error) => {
                    self.log.add(format!("Ошибка проверки процесса: {error}"));
                    true
                }
            },
            None => false,
        }
    }
}

impl Drop for Zapret {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Читает поток построчно в отдельном потоке и пишет строки в журнал
fn pipe_output(stream: impl Read + Send + 'static, tag: &'static str, log: Log) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for bytes in reader.split(b'\n').map_while(std::result::Result::ok) {
            log.add(format!("[{tag}] {}", String::from_utf8_lossy(&bytes)));
        }
    });
}
