use super::*;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Extrait une chaîne de caractères d'une zone spécifique du buffer.
fn get_string_from_buffer(buf: &Buffer, x: u16, y: u16, width: u16) -> String {
    let mut extracted_string = String::new();
    for i in 0..width {
        let cell = buf.cell((x + i, y))
            .expect(&format!("Expected a cell at ({}, {}) for string extraction, but none was found.", x + i, y));
        extracted_string.push(cell.symbol().chars().next().unwrap_or(' '));
    }
    extracted_string.trim().to_string()
}

/// Teste le rendu visuel de l'application.
#[test]
fn render() {
    let mut app = App::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 15));

    (&mut app).render(buf.area, &mut buf);

    // Vérifie le titre principal de l'application.
    assert_eq!(
        get_string_from_buffer(&buf, 31, 0, 17),
        "Compteur Avancé",
        "Le titre principal ne correspond pas."
    );

    // Vérifie les instructions affichées en bas de l'écran.
    let instructions_line_full = get_string_from_buffer(&buf, 0, 3, 80);
    assert!(instructions_line_full.contains("Décrémenter"), "Instructions missing 'Décrémenter'.");
    assert!(instructions_line_full.contains("<Gauche>"), "Instructions missing '<Gauche>'.");
    assert!(instructions_line_full.contains("Incrémenter"), "Instructions missing 'Incrémenter'.");
    assert!(instructions_line_full.contains("<Droite>"), "Instructions missing '<Droite>'.");
    assert!(instructions_line_full.contains("Quitter"), "Instructions missing 'Quitter'.");
    assert!(instructions_line_full.contains("<Q>"), "Instructions missing '<Q>'.");

    // Vérifie le texte de la valeur du compteur principal.
    assert_eq!(
        get_string_from_buffer(&buf, 35, 6, 9),
        "Valeur:",
        "Le texte de la valeur du compteur ne correspond pas."
    );

    // Vérifie le label du gauge de progression.
    assert_eq!(
        get_string_from_buffer(&buf, 37, 7, 3),
        "0",
        "Le label du gauge de progression ne correspond pas."
    );

    // Vérifie le texte du compteur de tours.
    assert_eq!(
        get_string_from_buffer(&buf, 35, 10, 8),
        "Tours:",
        "Le texte du compteur de tours ne correspond pas."
    );

    // Vérifie le label du gauge de tours.
    assert_eq!(
        get_string_from_buffer(&buf, 37, 11, 4),
        "0/5",
        "Le label du gauge de tours ne correspond pas."
    );
}

/// Teste la gestion des événements clavier.
#[test]
fn handle_key_event() -> io::Result<()> {
    // Fonction utilitaire pour extraire le texte du label du gauge.
    fn get_gauge_label_text(app_instance: &mut App, area: Rect) -> String {
        let mut buf = Buffer::empty(area);
        app_instance.render(buf.area, &mut buf);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(main_chunks[1]);

        let gauge_area = content_chunks[0];
        let gauge_label_y_pos = gauge_area.y + 2;
        let label_width = 10;
        let label_start_x = gauge_area.x + (gauge_area.width.saturating_sub(label_width)) / 2;
        let label_end_x = label_start_x + label_width;

        let mut extracted_string = String::new();
        for x in label_start_x.max(gauge_area.x)..label_end_x.min(gauge_area.right()) {
            let cell = buf.cell((x, gauge_label_y_pos)).expect(&format!(
                "Cell at ({}, {}) not found in gauge label area",
                x, gauge_label_y_pos
            ));
            extracted_string.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        extracted_string.trim().to_string()
    }

    let mut app = App::default();
    let area = Rect::new(0, 0, 80, 15);

    // Teste la logique de la barre de progression pour différentes valeurs de compteur.
    app.counter = -10;
    app.max_counter = 100;
    assert_eq!(get_gauge_label_text(&mut app, area), "0%", "Le gauge devrait afficher 0% pour un compteur négatif.");

    app.counter = 0;
    assert_eq!(get_gauge_label_text(&mut app, area), "0%", "Le gauge devrait afficher 0% pour un compteur à zéro.");

    app.counter = 2;
    app.max_counter = 5;
    assert_eq!(get_gauge_label_text(&mut app, area), "40%", "Le gauge devrait afficher 40% pour un compteur à 2 et max à 5.");

    // Réinitialise l'application pour les tests de gestion des événements clavier.
    let mut app = App::default();
    app.max_counter = 5;
    app.min_counter = -5;

    // Teste l'incrémentation du compteur jusqu'à la limite maximale et le passage au tour suivant.
    for _ in 0..app.max_counter { app.handle_key_event(KeyCode::Right.into()); }
    assert_eq!(app.counter, app.max_counter, "Le compteur devrait atteindre max_counter.");
    assert_eq!(app.round_counter, 0, "Le compteur de tours devrait rester à 0.");

    app.handle_key_event(KeyCode::Right.into());
    assert_eq!(app.counter, 0, "Le compteur devrait se réinitialiser à 0 après un nouveau tour.");
    assert_eq!(app.round_counter, 1, "Le compteur de tours devrait passer à 1.");
    assert_eq!(app.message, "Nouveau tour ! Tour actuel: 1", "Le message devrait indiquer un nouveau tour.");

    // Teste la décrémentation du compteur jusqu'à la limite minimale et le retour au tour précédent.
    for _ in 0..app.min_counter.abs() { app.handle_key_event(KeyCode::Left.into()); }
    assert_eq!(app.counter, app.min_counter, "Le compteur devrait atteindre min_counter.");
    assert_eq!(app.round_counter, 1, "Le compteur de tours devrait rester à 1.");

    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, 0, "Le compteur devrait se réinitialiser à 0 après retour au tour précédent.");
    assert_eq!(app.round_counter, 0, "Le compteur de tours devrait revenir à 0.");
    assert_eq!(app.message, "Retour au tour précédent ! Tour actuel: 0", "Le message devrait indiquer le retour au tour précédent.");

    // Teste le comportement à la limite inférieure des tours.
    app.counter = -5;
    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, app.min_counter, "Le compteur devrait rester à min_counter si déjà à la limite.");
    assert_eq!(app.round_counter, 0, "Le compteur de tours devrait rester à 0 à la limite inférieure.");
    assert_eq!(app.message, format!("Limite inférieure des tours atteinte et compteur à {}. Impossible de décrémenter davantage.", app.min_counter), "Le message devrait indiquer que la limite inférieure est atteinte.");

    // Teste la sortie de l'application.
    let mut app = App::default();
    app.handle_key_event(KeyCode::Char('q').into());
    assert!(app.exit, "L'application devrait se quitter après 'q'.");

    Ok(())
}