use crate::config::{LogConfig, LogTimeFormat};
use chrono::Utc;
use crossterm::{
    style::{Color::DarkGrey, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, disable_raw_mode},
};
#[cfg(feature = "spawn_chunk_display")]
use std::io::Result;
use std::{
    io::Write,
    sync::Arc,
    time::{self, Instant},
};
use steel_utils::locks::AsyncRwLock;
use steel_utils::logger::{Level, LogData, STEEL_LOGGER, SteelLogger};
use tokio::{sync::mpsc, task, time::timeout};
use tokio_util::sync::CancellationToken;
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

mod file;
mod history;
mod input;
mod output;
mod selection;
#[cfg(feature = "spawn_chunk_display")]
mod spawn_progress;
mod state;
mod suggestions;

/// Returns the terminal width, falling back to 80 columns if unavailable or it's <= 0.
fn terminal_width() -> usize {
    terminal::size().map_or(80, |(w, _)| if w == 0 { 80 } else { w as usize })
}

pub(crate) use state::LogState;

#[cfg(feature = "spawn_chunk_display")]
pub(crate) use spawn_progress::Grid;

pub(crate) enum Move {
    None,
    Up,
    Down,
}

/// A logger implementation with commands suggestions
pub struct CommandLogger {
    input: Arc<AsyncRwLock<LogState>>,
    sender: mpsc::UnboundedSender<(Level, LogData)>,
    cancel_token: CancellationToken,
    stopped: CancellationToken,
    start_time: Instant,
    log_config: Option<LogConfig>,
}

impl CommandLogger {
    /// Initializes the `CommandLogger`
    pub async fn init(
        cancel_token: CancellationToken,
        log_config: Option<LogConfig>,
    ) -> Option<Arc<Self>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let log_cancel_token = CancellationToken::new();

        let log = Arc::new(Self {
            input: Arc::new(AsyncRwLock::const_new(
                LogState::new(log_config.as_ref(), cancel_token).await,
            )),
            sender,
            cancel_token: log_cancel_token.clone(),
            stopped: CancellationToken::new(),
            start_time: Instant::now(),
            log_config,
        });
        task::spawn(log.clone().log_loop(receiver));
        task::spawn(log.clone().input_main());
        STEEL_LOGGER.set(log.clone()).ok()?;
        Some(log)
    }

    /// Stops the logger and waits for cleanup to complete
    pub async fn stop(&self) {
        self.cancel_token.cancel();
        if timeout(time::Duration::from_secs(1), self.stopped.cancelled())
            .await
            .is_err()
        {
            let _ = disable_raw_mode();
            self.stopped.cancel();
        }
    }

    async fn log_loop(self: Arc<Self>, mut receiver: mpsc::UnboundedReceiver<(Level, LogData)>) {
        loop {
            #[cfg(feature = "spawn_chunk_display")]
            if self.input.read().await.spawn_display.rendered {
                continue;
            }
            tokio::select! {
                biased;
                Some((lvl, data)) = receiver.recv() => {
                    let (lvl, data) = self.write_log_entry(lvl, data).await;
                    if self.log_config.as_ref().is_some_and(|l| l.file) {
                        self.write_file_entry(lvl, data).await;
                    }
                }
                () = self.cancel_token.cancelled() => break,
            }
        }
    }

    async fn write_log_entry(&self, lvl: Level, data: LogData) -> (Level, LogData) {
        let mut input = self.input.write().await;
        let pos = input.out.get_current_pos();

        if let Err(err) = input.out.cursor_to(pos, (0, 0)) {
            log::error!("{err}");
            return (lvl, data);
        }

        let time_str = self.format_time();
        let module_path_str = self.format_module_path(&data, true);
        let extra_str = self.format_extra(&data, true);

        if let Err(err) = writeln!(
            input.out,
            "{}{time_str}{lvl} {module_path_str}{}{extra_str}\r",
            Clear(ClearType::FromCursorDown),
            data.message,
        ) {
            log::error!("{err}");
            return (lvl, data);
        }

        if let Err(err) = input.out.cursor_to((0, 0), pos) {
            log::error!("{err}");
        }
        if let Err(err) = input.rewrite_current_input() {
            log::error!("{err}");
        }
        (lvl, data)
    }

    async fn write_file_entry(&self, lvl: Level, data: LogData) {
        let mut input = self.input.write().await;

        let time_str = self.format_time();
        let module_path_str = self.format_module_path(&data, false);
        let extra_str = self.format_extra(&data, false);

        if let Err(err) = writeln!(
            input.file,
            "{time_str}{lvl:?} {module_path_str}{}{extra_str}",
            strip_ansi_escapes::strip_str(&data.message),
        ) {
            log::error!("{err}");
        }
    }

    fn format_time(&self) -> String {
        match self.log_config.as_ref().map(|l| &l.time) {
            Some(LogTimeFormat::Date) => {
                let time: chrono::DateTime<Utc> = time::SystemTime::now().into();
                format!("{} ", time.format("%T:%3f"))
            }
            Some(LogTimeFormat::Uptime) => {
                let elapsed = self.start_time.elapsed();
                format!("{:>6.2}s ", elapsed.as_secs_f64())
            }
            _ => String::new(),
        }
    }

    fn format_module_path(&self, data: &LogData, color: bool) -> String {
        if self.log_config.as_ref().is_some_and(|l| l.module_path) {
            if color {
                format!(
                    " {}{}{} ",
                    SetForegroundColor(DarkGrey),
                    data.module_path,
                    ResetColor
                )
            } else {
                format!(" {} ", data.module_path)
            }
        } else {
            String::new()
        }
    }

    fn format_extra(&self, data: &LogData, color: bool) -> String {
        if self.log_config.as_ref().is_some_and(|l| l.extra) {
            if color {
                format!(
                    "{}{}{}",
                    SetForegroundColor(DarkGrey),
                    data.extra,
                    ResetColor
                )
            } else {
                data.extra.clone()
            }
        } else {
            String::new()
        }
    }
}

#[cfg(feature = "spawn_chunk_display")]
impl CommandLogger {
    /// Initializes the display of the spawn chunks
    pub async fn activate_spawn_display(&self) -> Result<()> {
        use crate::spawn_progress::DISPLAY_RADIUS;
        use crossterm::terminal::Clear;
        use std::time::Duration;
        use tokio::time::sleep;

        // Extra time to let the logs appear correctly
        sleep(Duration::from_millis(1)).await;
        let mut input = self.input.write().await;
        input.spawn_display.rendered = true;
        let pos = input.out.get_current_pos();
        input.out.cursor_to(pos, (0, 0))?;
        write!(input.out, "\r{}", Clear(ClearType::FromCursorDown))?;
        for _ in 0..=DISPLAY_RADIUS {
            writeln!(input.out)?;
        }
        input.out.cursor_to((0, 0), pos)?;
        input.out.flush()?;
        input.rewrite_current_input()?;
        Ok(())
    }

    /// Ends the spawn display cleaning the screen
    pub async fn deactivate_spawn_display(&self) {
        use crate::spawn_progress::DISPLAY_RADIUS;
        use crossterm::cursor::MoveUp;

        let mut input = self.input.write().await;
        write!(
            input.out,
            "{}\n{}",
            MoveUp(DISPLAY_RADIUS as u16 + 2),
            Clear(ClearType::FromCursorDown)
        )
        .ok();
        input.rewrite_current_input().ok();
        input.spawn_display.rendered = false;
    }

    /// Updates the spawn grid, and displays it if required
    pub async fn update_spawn_grid(&self, grid: &Grid, should_render: bool) -> Result<()> {
        let mut state = self.input.write().await;
        state.spawn_display.set_grid(grid);
        if !should_render {
            return Ok(());
        }
        {
            let state = &mut state as &mut LogState;
            state.spawn_display.rewrite(&mut state.out)?;
        }
        state.rewrite_current_input()
    }
}

impl SteelLogger for CommandLogger {
    fn log(&self, lvl: Level, data: LogData) {
        self.sender.send((lvl, data)).ok();
    }
}

/// A logger layer for tracing
pub struct LoggerLayer(pub Arc<CommandLogger>);

impl LoggerLayer {
    /// Creates a new logger
    pub async fn new(
        cancel_token: CancellationToken,
        log_config: Option<LogConfig>,
    ) -> Option<Self> {
        Some(Self(CommandLogger::init(cancel_token, log_config).await?))
    }
}

impl<S: Subscriber> Layer<S> for LoggerLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut data = LogData::new();
        event.record(&mut data);
        self.0.log(Level::Tracing(*event.metadata().level()), data);
    }
}
