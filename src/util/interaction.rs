use anyhow::Result;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

pub trait UserInteraction {
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool>;
    fn input(&self, prompt: &str, default: Option<&str>) -> Result<String>;
    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize>;
    fn fuzzy_select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize>;
}

pub struct TerminalInteraction;

impl UserInteraction for TerminalInteraction {
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool> {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn input(&self, prompt: &str, default: Option<&str>) -> Result<String> {
        if let Some(d) = default {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .default(d.to_string())
                .interact_text()
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .interact_text()
                .map_err(|e| anyhow::anyhow!(e))
        }
    }

    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn fuzzy_select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize> {
        use dialoguer::FuzzySelect;
        FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(|e| anyhow::anyhow!(e))
    }
}
