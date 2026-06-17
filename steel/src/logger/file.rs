use std::{
    fs::{File, OpenOptions},
    io::{Result, Write},
};

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::config::RotationTimeFormat;

pub struct LogFile {
    file: Option<File>,
    date: DateTime<Utc>,
    base_path: String,
    rotation_time: RotationTimeFormat,
}
impl LogFile {
    pub fn new(base_path: String, rotation_time: RotationTimeFormat, enabled: bool) -> Self {
        let date = Utc::now();

        let file = if enabled {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::get_filename(&base_path, rotation_time, date))
                .ok()
        } else {
            None
        };
        Self {
            file,
            date,
            base_path,
            rotation_time,
        }
    }
    fn get_filename(
        base_path: &str,
        rotation_time: RotationTimeFormat,
        date: DateTime<Utc>,
    ) -> String {
        match rotation_time {
            RotationTimeFormat::None => format!("{base_path}/steel.log"),
            RotationTimeFormat::Hourly => {
                format!("{base_path}/steel-{}.log", date.format("%Y_%m_%d_%H"))
            }
            RotationTimeFormat::Daily | RotationTimeFormat::Weekly => {
                format!("{base_path}/steel-{}.log", date.format("%Y_%m_%d"))
            }
            RotationTimeFormat::Monthly => {
                format!("{base_path}/steel-{}.log", date.format("%Y_%m"))
            }
        }
    }
    fn check_time(&self, now: DateTime<Utc>) -> bool {
        match self.rotation_time {
            RotationTimeFormat::None => false,
            RotationTimeFormat::Hourly => {
                self.date.hour() != now.hour()
                    || self.date.day() != now.day()
                    || self.date.month() != now.month()
            }
            RotationTimeFormat::Daily => {
                self.date.day() != now.day() || self.date.month() != now.month()
            }
            RotationTimeFormat::Weekly => {
                self.date.day() - now.day() >= 7 || self.date.month() != now.month()
            }
            RotationTimeFormat::Monthly => self.date.month() != now.month(),
        }
    }
}

impl Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.file.is_none() {
            return Ok(0);
        }
        let now = Utc::now();
        if self.check_time(now) {
            self.date = now;
            self.file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::get_filename(
                    &self.base_path,
                    self.rotation_time,
                    self.date,
                ))
                .ok();
        }
        self.file.as_mut().expect("File writer dropped").write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        let Some(ref mut file) = self.file else {
            return Ok(());
        };
        file.flush()
    }
}
