use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

#[derive(Clone)]
pub struct ProgressTracker {
    pb: ProgressBar,
    no_progress: bool,
}

impl ProgressTracker {
    pub fn new(total_steps: u64, no_progress: bool) -> anyhow::Result<Self> {
        let pb = if no_progress {
            ProgressBar::hidden()
        } else {
            let pb = ProgressBar::new(total_steps);
            pb.set_style(
                ProgressStyle::with_template(
                    "\n{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {pos}/{len} ({percent}%) {msg}",
                )
                .context("Failed to set progress bar template")?
                .progress_chars("█▌ "),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            pb
        };

        Ok(Self { pb, no_progress })
    }

    pub fn inc(&self) {
        self.pb.inc(1);
    }

    pub fn println(&self, msg: &str) {
        if self.no_progress {
            println!("{}", msg);
        } else {
            self.pb.println(msg);
        }
    }

    pub fn set_message(&self, msg: impl Into<String>) {
        self.pb.set_message(msg.into());
    }

    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if self.no_progress {
            println!("{}", msg.into());
        } else {
            self.pb.finish_with_message(msg.into());
        }
    }

    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if self.no_progress {
            f()
        } else {
            self.pb.suspend(f)
        }
    }
}
