// Adapted from https://github.com/tatounee/ratatui-explorer/tree/master

use std::{io::stdout, path::PathBuf};

use inquire::ui::RenderConfig;
use ratatui::{
    crossterm::{
        self, ExecutableCommand,
        event::{self, Event, KeyCode},
        style::{ResetColor, StyledContent},
        terminal::{
            EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
            enable_raw_mode,
        },
    },
    prelude::*,
    style::Styled,
    widgets::{Block, BorderType, Borders},
};
use ratatui_explorer::{FileExplorer, Theme};
use snafu::ResultExt;

use crate::{Result, inquire_stylesheet_shim};

/// Don't end `label` with a colon, it'll be added for you.
pub fn prompt_for_path(
    label: &str,
    default_path: Option<&str>,
    render_config: &RenderConfig<'static>,
) -> Result<PathBuf> {
    enable_raw_mode().whatever_context("Failed to enable raw mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .whatever_context("Failed to switch screens")?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .whatever_context("Failed to make terminal")?;

    let label_copy = label.to_string();
    let theme = Theme::default()
        .with_title_top(move |file_explorer| {
            Line::from(format!(
                " {label_copy} | {} | (VIM, ENTER=select) ",
                file_explorer.cwd().display()
            ))
        })
        .with_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    let mut file_explorer = FileExplorer::with_theme(theme)
        .whatever_context("Failed to make file explorer")?;

    if let Some(default_path) = default_path {
        let path = PathBuf::from(default_path);
        if let Some(parent) = path.parent() {
            file_explorer.set_cwd(parent).whatever_context(
                "Failed to set initial directory for file explorer",
            )?;
            if let Some(local_index) = file_explorer
                .files()
                .iter()
                .position(|file| file.path() == &path)
            {
                file_explorer.set_selected_idx(local_index);
            }
        }
    }

    loop {
        terminal
            .draw(|frame| {
                frame.render_widget(&file_explorer.widget(), frame.area());
            })
            .whatever_context("Failed to draw widget")?;

        let event =
            event::read().whatever_context("Failed to read terminal event")?;
        if let Event::Key(key) = event {
            if key.code == KeyCode::Enter {
                break;
            }
        }
        file_explorer
            .handle(&event)
            .whatever_context("Failed to handle event in file explorer")?;
    }

    disable_raw_mode().whatever_context("Failed to disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .whatever_context("Failed to restore original screen")?;

    let prefix_content_style = inquire_stylesheet_shim::stylesheet_shim(
        render_config.answered_prompt_prefix.style,
    );
    let answered_content_style =
        inquire_stylesheet_shim::stylesheet_shim(render_config.answer);

    let path = file_explorer.current().path();
    println!(
        "{} {}: {}",
        StyledContent::new(
            prefix_content_style,
            render_config.answered_prompt_prefix.content,
        ),
        label,
        StyledContent::new(answered_content_style, path.display().to_string(),)
    );

    Ok(path.clone())
}
