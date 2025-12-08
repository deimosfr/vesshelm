use crate::util::error::InteractionError;
use console::{Style, style};
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};

pub trait UserInteraction {
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool, InteractionError>;
    fn input(&self, prompt: &str, default: Option<&str>) -> Result<String, InteractionError>;
    fn select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InteractionError>;
    fn fuzzy_select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InteractionError>;
    fn multi_select(
        &self,
        prompt: &str,
        items: &[String],
        defaults: &[bool],
    ) -> Result<Vec<usize>, InteractionError>;
}

pub struct TerminalInteraction;

impl UserInteraction for TerminalInteraction {
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool, InteractionError> {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(InteractionError::Dialoguer)
    }

    fn input(&self, prompt: &str, default: Option<&str>) -> Result<String, InteractionError> {
        if let Some(d) = default {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .default(d.to_string())
                .interact_text()
                .map_err(InteractionError::Dialoguer)
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .interact_text()
                .map_err(InteractionError::Dialoguer)
        }
    }

    fn select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InteractionError> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(InteractionError::Dialoguer)
    }

    fn fuzzy_select(
        &self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InteractionError> {
        use dialoguer::FuzzySelect;
        let theme = ColorfulTheme {
            active_item_style: Style::new().blue().bold().bright(),
            active_item_prefix: style("❯".to_string()).for_stderr().green().bright(),
            ..ColorfulTheme::default()
        };

        FuzzySelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(InteractionError::Dialoguer)
    }

    fn multi_select(
        &self,
        prompt: &str,
        items: &[String],
        defaults: &[bool],
    ) -> Result<Vec<usize>, InteractionError> {
        let theme = ColorfulTheme {
            checked_item_prefix: style("[X]".to_string()).for_stderr(),
            unchecked_item_prefix: style("[ ]".to_string()).for_stderr(),
            ..ColorfulTheme::default()
        };
        MultiSelect::with_theme(&theme)
            .with_prompt(prompt)
            .items(items)
            .defaults(defaults)
            .interact()
            .map_err(InteractionError::Dialoguer)
    }
}
