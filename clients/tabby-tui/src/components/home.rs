use std::{collections::HashMap, time::Duration};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{
  action::Action,
  config::key_event_to_string,
  core::api::{
    fetch_chat_view_data, fetch_health_view_data, stream_tabby, ChatRole, TabbyChatViewData, TabbyClientViewData,
  },
};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
}

#[derive(Default)]
pub struct Home {
  pub main_title: String,
  pub show_help: bool,
  pub counter: usize,
  pub app_ticker: usize,
  pub render_ticker: usize,
  pub mode: Mode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub text: Vec<String>,
  pub last_events: Vec<KeyEvent>,

  pub vertical_scroll_state: ScrollbarState,
  pub vertical_scroll: usize,
  pub vertical_scroll_max: usize,
}

impl Home {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  pub fn tick(&mut self) {
    log::info!("Tick");
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    log::debug!("Render Tick");
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn add(&mut self, s: String) {
    self.text.push(s)
  }

  pub fn decrement(&mut self, i: usize) {
    self.counter = self.counter.saturating_sub(i);
  }

  pub fn schedule_health_check(&mut self) {
    let tx = self.action_tx.clone().unwrap();
    tokio::spawn(async move {
      let health_view_data = fetch_health_view_data().await;
      tx.send(Action::UpdateHealthCheckView(health_view_data)).unwrap();
    });
  }

  pub fn update_health_check_view(&mut self, health_view_data: TabbyClientViewData) {
    match health_view_data.health_state {
      Some(health_state) => {
        let mut main_title =
          format!("Tabby {} | {} | {}", health_state.version.git_describe, health_state.model, health_state.device);

        if health_state.cuda_devices.len() > 0 {
          main_title += " (";
          main_title += &health_state.cuda_devices.join(", ");
          main_title += ") ";
        }

        self.main_title = main_title;
      },
      None => self.main_title = "⚠️ Tabby not respond".to_owned(),
    }
  }

  pub fn schedule_infer(&mut self, prompt: &str) {
    let tx = self.action_tx.clone().unwrap();
    tokio::spawn(async move {
      let callback = |chunk: String| {
        let msg = TabbyChatViewData { role: ChatRole::Tabby, text: Some(chunk.to_string()) };
        tx.send(Action::UpdateChatView(msg)).unwrap();
      };

      fetch_chat_view_data(callback).await;
    });
  }

  pub fn update_chat_view(&mut self, chat_view_data: TabbyChatViewData) {
    let text = match chat_view_data.text {
      Some(text) => format!("{:?}: {text}", chat_view_data.role),
      None => format!("{:?}: An error occurred.", chat_view_data.role),
    };

    self.add(text);
  }
}

impl Component for Home {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Normal | Mode::Processing => return Ok(None),
      Mode::Insert => match key.code {
        KeyCode::Esc => Action::EnterNormal,
        KeyCode::Enter => {
          if let Some(sender) = &self.action_tx {
            if let Err(e) = sender.send(Action::CompleteInput("Me".to_owned(), self.input.value().to_string())) {
              error!("Failed to send action: {:?}", e);
            }
          }
          Action::EnterNormal
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          Action::Update
        },
      },
    };
    Ok(Some(action))
  }

  fn init(&mut self) -> Result<()> {
    self.schedule_health_check();
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => self.tick(),
      Action::Render => self.render_tick(),
      Action::ToggleShowHelp => self.show_help = !self.show_help,
      Action::CompleteInput(talker, word) => {
        self.add(format!("{talker}: {word}"));
        self.schedule_infer(&word);
      },
      Action::CompleteInfer(talker, word) => self.add(format!("{talker}: {word}")),
      Action::EnterNormal => {
        self.mode = Mode::Normal;
      },
      Action::EnterInsert => {
        self.mode = Mode::Insert;
      },
      Action::EnterProcessing => {
        self.mode = Mode::Processing;
      },
      Action::ExitProcessing => {
        // TODO: Make this go to previous mode instead
        self.mode = Mode::Normal;
      },
      Action::ScheduleHealthCheck => {
        self.schedule_health_check();
      },
      Action::UpdateHealthCheckView(health_check_data) => {
        self.update_health_check_view(health_check_data);
      },
      Action::Up => {
        self.vertical_scroll_state.scroll(ScrollDirection::Backward);
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
      },
      Action::Down => {
        self.vertical_scroll_state.scroll(ScrollDirection::Forward);
        if self.vertical_scroll < self.vertical_scroll_max - 1 {
          self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        }
      },
      Action::UpdateChatView(chat_view_data) => {
        self.update_chat_view(chat_view_data);
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref()).split(rect);

    // Text area --------------------------------------------

    let line = self.text.clone().iter().map(|e| Line::from(e.clone())).collect::<Vec<_>>();

    let size = f.size();

    self.vertical_scroll_max = line.len();
    self.vertical_scroll_state = self.vertical_scroll_state.content_length(self.vertical_scroll_max as u16);

    // Chat input --------------------------------------------

    let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = self.input.visual_scroll(width as usize);
    let input = Paragraph::new(self.input.value())
      .style(match self.mode {
        Mode::Insert => Style::default().fg(Color::Yellow),
        _ => Style::default(),
      })
      .scroll((0, scroll as u16))
      .block(Block::default().borders(Borders::ALL).title(Line::from(vec![
        Span::raw("Chat "),
        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("/", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
      ])));
    f.render_widget(input, rects[1]);

    if self.mode == Mode::Insert {
      f.set_cursor((rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2), rects[1].y + 1)
    }

    if self.show_help {
      let rect = rect.inner(&Margin { horizontal: 4, vertical: 2 });
      f.render_widget(Clear, rect);
      let block = Block::default()
        .title(Line::from(vec![Span::styled("Key Bindings", Style::default().add_modifier(Modifier::BOLD))]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
      f.render_widget(block, rect);
      let rows = vec![
        Row::new(vec!["/", "Enter Input"]),
        Row::new(vec!["ESC", "Exit Input"]),
        Row::new(vec!["Enter", "Submit Input"]),
        Row::new(vec!["q", "Quit"]),
        Row::new(vec!["?", "Open Help"]),
      ];
      let table = Table::new(rows)
        .header(Row::new(vec!["Key", "Action"]).bottom_margin(1).style(Style::default().add_modifier(Modifier::BOLD)))
        .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
        .column_spacing(1);
      f.render_widget(table, rect.inner(&Margin { vertical: 4, horizontal: 2 }));
    };

    f.render_widget(
      Block::default()
        .title(
          ratatui::widgets::block::Title::from(format!(
            "{:?}",
            &self.last_events.iter().map(|k| key_event_to_string(k)).collect::<Vec<_>>()
          ))
          .alignment(Alignment::Right),
        )
        .title_style(Style::default().add_modifier(Modifier::BOLD)),
      Rect { x: rect.x + 1, y: rect.height.saturating_sub(1), width: rect.width.saturating_sub(2), height: 1 },
    );

    let paragraph = Paragraph::new(line.clone())
      .gray()
      .block(
        Block::default()
          .title(self.main_title.as_str())
          .borders(Borders::ALL)
          .border_style(match self.mode {
            Mode::Processing => Style::default().fg(Color::Yellow),
            _ => Style::default(),
          })
          .border_type(BorderType::Rounded),
      )
      .scroll((self.vertical_scroll as u16, 0));
    f.render_widget(paragraph, rects[0]);
    f.render_stateful_widget(
      Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓")),
      rects[0],
      &mut self.vertical_scroll_state,
    );

    Ok(())
  }
}
