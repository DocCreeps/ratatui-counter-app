// src/tests/mod.rs

use super::*;
use ratatui::{layout::Position, style::Style}; // Import Position

/// Teste le rendu visuel de l'application.
#[test]
fn render() {
    let mut app = App::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 15));

    (&mut app).render(buf.area, &mut buf);

    let expected_lines = vec![
        "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Compteur Avancé ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
        "┃ Décrémenter <Gauche> Incrémenter <Droite> Quitter <Q>                      ┃", // Ligne d'instruction modifiée
        "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
        " ┌──────────────────────────────────────────────────────────────────────────┐ ",
        " │ Progression Compteur                                                     │ ",
        " │                            Valeur: 0                                     │ ",
        " │ 0%                                                                       │ ",
        " └──────────────────────────────────────────────────────────────────────────┘ ",
        " ┌──────────────────────────────────────────────────────────────────────────┐ ",
        " │ Objectif Tours                                                           │ ",
        " │                            Tours: 0                                      │ ",
        " │ 0/50                                                                     │ ",
        " └──────────────────────────────────────────────────────────────────────────┘ ",
        "                                                                                ",
        "                                                                                ",
    ];
    let mut expected = Buffer::with_lines(expected_lines);

    let title_style = Style::new().bold();
    let key_style = Style::new().blue().bold();

    expected.set_style(Rect::new(26, 0, 17, 1), title_style);
    expected.set_style(Rect::new(22, 1, 7, 1), key_style); // <Gauche>

    assert_eq!(buf, expected);
}

/// Teste la gestion des événements clavier.
#[test]
fn handle_key_event() -> io::Result<()> {
    // Crée une fonction utilitaire pour récupérer le texte du label du gauge
    // afin d'éviter de dupliquer la logique de rendu pour chaque assertion.
    fn get_gauge_label_text(app_instance: &mut App, area: Rect) -> String {
        let mut buf = Buffer::empty(area);

        // Rendu de l'application
        app_instance.render(buf.area, &mut buf);

        // The line where the main counter Gauge label is displayed is at Y=6 in the global buffer (0-indexed)
        // assuming a Rect of 80x15 and the current layouts.
        let gauge_label_y_pos = area.y + 6;

        let mut extracted_string = String::new();
        // Recalculate the area where the gauge is drawn
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        let gauge_area_within_main_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Height for the Gauge
                Constraint::Length(4), // Height for the Round Gauge
                Constraint::Min(0),
            ])
            .margin(1)
            .split(main_chunks[1])[0]; // We want the first content chunk (the main counter gauge)


        // Iterate over cells in the region where the gauge label is expected to be
        // The label is centered on the third line of the gauge.
        let label_start_x = gauge_area_within_main_chunk.x;
        let label_end_x = gauge_area_within_main_chunk.x + gauge_area_within_main_chunk.width;

        for x in label_start_x .. label_end_x {
            // FIX: Pass a tuple (x, y) as Position and unwrap the Option
            let cell = buf.cell_mut((x, gauge_label_y_pos)).unwrap();
            extracted_string.push(cell.symbol().chars().next().unwrap_or(' '));
        }

        extracted_string.trim().to_string()
    }


    let mut app = App::default();
    let area = Rect::new(0,0, 80, 15); // Use a fixed screen size for rendering tests

    // Test the new progress bar logic:
    app.counter = -10; // Set the value for this test
    app.max_counter = 100; // Ensure max_counter is set
    assert_eq!(get_gauge_label_text(&mut app, area), "0%");

    app.counter = 0; // Set the value for this test
    assert_eq!(get_gauge_label_text(&mut app, area), "0%");

    app.counter = 2; // Set the value for this test
    app.max_counter = 5; // Adjust max_counter for a simple calculation (2/5 = 40%)
    assert_eq!(get_gauge_label_text(&mut app, area), "40%");


    // Continue with existing tests for the rest of the logic...
    let mut app = App::default(); // Reset the app for subsequent tests
    app.max_counter = 5;
    app.min_counter = -5;

    for _ in 0..app.max_counter {
        app.handle_key_event(KeyCode::Right.into());
    }
    assert_eq!(app.counter, app.max_counter);
    assert_eq!(app.round_counter, 0);

    app.handle_key_event(KeyCode::Right.into());
    assert_eq!(app.counter, 0);
    assert_eq!(app.round_counter, 1);
    assert_eq!(app.message, "Nouveau tour ! Tour actuel: 1");

    for _ in 0..app.min_counter.abs() {
        app.handle_key_event(KeyCode::Left.into());
    }
    assert_eq!(app.counter, app.min_counter);
    assert_eq!(app.round_counter, 1);

    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, 0);
    assert_eq!(app.round_counter, 0);
    assert_eq!(app.message, "Retour au tour précédent ! Tour actuel: 0");

    app.counter = -5;
    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, app.min_counter);
    assert_eq!(app.round_counter, 0);
    assert_eq!(app.message, format!("Limite inférieure des tours atteinte et compteur à {}. Impossible de décrémenter davantage.", app.min_counter));

    let mut app = App::default();
    app.handle_key_event(KeyCode::Char('q').into());
    assert!(app.exit);

    Ok(())
}
