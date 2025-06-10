use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    buffer::Buffer,
    // Suppression de Margin et Position car inutilisés
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Gauge, Paragraph, Widget},
};

// Fonction d'initialisation et de restauration du terminal
pub fn init() -> io::Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = init()?;
    let app_result = App::default().run(&mut terminal);
    restore()?;
    app_result
}

/// Structure qui représente l'état de notre application.
#[derive(Debug)]
pub struct App {
    counter: i8,
    round_counter: i32,
    exit: bool,
    max_counter: i8,
    min_counter: i8,
    max_rounds: i32,
    message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            counter: 0,
            round_counter: 0,
            exit: false,
            max_counter: 100,
            min_counter: -100,
            max_rounds: 50,
            message: String::new(),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area())
            })?;

            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {
                self.message.clear();
            }
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.message.clear();
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        if self.counter < self.max_counter {
            self.counter += 1;
        } else {
            self.round_counter += 1;
            self.counter = 0;
            self.message = format!("Nouveau tour ! Tour actuel: {}", self.round_counter);
        }
    }

    fn decrement_counter(&mut self) {
        if self.counter > self.min_counter {
            self.counter -= 1;
        } else {
            if self.round_counter > 0 {
                self.round_counter -= 1;
                self.counter = 0;
                self.message = format!("Retour au tour précédent ! Tour actuel: {}", self.round_counter);
            } else {
                self.message = format!("Limite inférieure des tours atteinte et compteur à {}. Impossible de décrémenter davantage.", self.min_counter);
                self.counter = self.min_counter;
            }
        }
    }
}

// Implémentation du trait Widget pour &mut App
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Compteur Avancé ".bold());
        let instructions = Line::from(vec![
            " Décrémenter ".into(),
            "<Gauche>".blue().bold(),
            " Incrémenter ".into(),
            "<Droite>".blue().bold(),
            " Quitter ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        block.render(main_chunks[0], buf);

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(main_chunks[1]);

        // --- Rendu du Compteur Principal et sa barre ---
        let counter_color = if self.counter > 0 {
            Color::LightGreen
        } else if self.counter < 0 {
            Color::LightRed
        } else {
            Color::Yellow
        };

        let counter_text_line = Line::from(vec![
            "Valeur: ".into(),
            self.counter.to_string().fg(counter_color).bold(),
        ]);

        let progress_value_counter = if self.counter >= 0 {
            (self.counter as f64 / self.max_counter as f64).min(1.0)
        } else {
            0.0
        };

        let counter_gauge_block = Block::bordered().title("Progression Compteur");

        let counter_gauge = Gauge::default()
            .block(counter_gauge_block.clone())
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .ratio(progress_value_counter)
            .label(format!("{:.0}%", progress_value_counter * 100.0));

        counter_gauge.render(content_chunks[0], buf);

        Paragraph::new(counter_text_line)
            .centered()
            .render(counter_gauge_block.inner(content_chunks[0]), buf);


        // --- Rendu du Compteur de Tours et sa barre ---
        let round_text_line = Line::from(vec![
            "Tours: ".into(),
            self.round_counter.to_string().cyan().bold(),
        ]);

        let round_progress_value = if self.max_rounds > 0 {
            (self.round_counter as f64 / self.max_rounds as f64)
                .max(0.0)
                .min(1.0)
        } else {
            0.0
        };

        let round_gauge_block = Block::bordered().title("Objectif Tours");

        let round_gauge = Gauge::default()
            .block(round_gauge_block.clone())
            .gauge_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray))
            .ratio(round_progress_value)
            .label(format!("{}/{}", self.round_counter, self.max_rounds));

        round_gauge.render(content_chunks[1], buf);

        Paragraph::new(round_text_line)
            .centered()
            .render(round_gauge_block.inner(content_chunks[1]), buf);


        // --- Rendu du message temporaire ---
        if !self.message.is_empty() {
            let message_text =
                Text::from(Line::from(self.message.clone()).fg(Color::Yellow).centered());
            Paragraph::new(message_text).render(main_chunks[2], buf);
        }
    }
}


// Déclaration du module de tests
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;