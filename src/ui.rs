use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Percentage(100),
        ])
        .split(size);

    let block = Block::default()
        .title(&*app.title)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(block, chunks[0]);

    let constraints =
        vec![Constraint::Percentage(100), Constraint::Percentage(90)];

    let chunks = Layout::default()
        .constraints(constraints)
        .horizontal_margin(1)
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    if !app.loading {
        draw_tasks(f, app, chunks[0]);
        if let Some(task) = app.opened_task.as_ref() {
            let block = Block::default()
                .title(task.name.to_owned())
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            let area = centered_rect(60, 20, size);
            f.render_widget(Clear, area);
            f.render_widget(block, area);

            // add gauge in the middle of the popup
            let chunks = Layout::default()
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .margin(2)
                .split(area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Editing task: {} ({})", task.name, task.id)).style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(block, area);

            let label = format!("{}%", app.gauge_value);
            let gauge = Gauge::default()
                .block(Block::default().title("Percentage Done:"))
                .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                .gauge_style(
                    Style::default()
                        .fg(Color::LightCyan)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC | Modifier::BOLD),
                )
                .label(label)
                .ratio(app.gauge_value as f64 / 100.0);
            f.render_widget(gauge, chunks[0]);

            // add press Enter to save label
            let block =
                Block::default().borders(Borders::ALL).title(Span::styled(
                    "",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
            let paragraph =
                Paragraph::new("Press Enter to save, Esc to cancel").block(block).wrap(Wrap { trim: true }).alignment(Alignment::Center).style(
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                );
            f.render_widget(paragraph, chunks[1]);
        }
    } else {
        draw_loading(f, chunks[0]);
    }
}

fn draw_tasks<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let longest_task_name = app
        .tasks
        .items
        .iter()
        .map(|i| i.name.len())
        .max()
        .unwrap_or(0);
    let longest_username = app
        .tasks
        .items
        .iter()
        .map(|i| i.username.len())
        .max()
        .unwrap_or(0);
    let id_max_length = app
        .tasks
        .items
        .iter()
        .map(|i| i.id.to_string().len())
        .max()
        .unwrap_or(0);

    let labels_width = 43 + id_max_length;
    let list_width = area.width as usize;
    let remaining_width = list_width - labels_width;

    let usernam_width = longest_username;
    let task_name_width = remaining_width - usernam_width;

    let tasks: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|i| {
            let id_label_span =
                Span::styled(format!("Id: "), Style::default().fg(Color::Blue));
            let id_span = Span::styled(
                format!("{:0padding$}", i.id, padding=id_max_length),
                Style::default().fg(Color::Cyan),
            );

            let name_label_span =
                Span::styled("Name: ", Style::default().fg(Color::Blue));
            let name_span = Span::styled(
                format!("{:<width$.width$}", i.name, width=task_name_width),
                Style::default().fg(Color::Cyan),
            );

            let percentage_done_label_span = Span::styled(
                "Progress: ",
                Style::default().fg(Color::Blue),
            );
            let percentage_done_span = Span::styled(
                format!("{:>2}%", i.percentage_done),
                Style::default().fg(Color::Cyan),
            );

            let username_label_span = Span::styled(
                "Owner: ",
                Style::default().fg(Color::Blue),
            );
            let username_span = Span::styled(
                format!("{:<width$.width$}", i.username, width=usernam_width),
                Style::default().fg(Color::Cyan),
            );

            let separator =
                Span::styled(" | ", Style::default().fg(Color::DarkGray));

            let spans = Spans::from(vec![
                id_label_span,
                id_span,
                separator.clone(),
                percentage_done_label_span,
                percentage_done_span,
                separator.clone(),
                name_label_span,
                name_span,
                separator,
                username_label_span,
                username_span,
            ]);

            ListItem::new(spans)
        })
        .collect();

    let task_selected = app.tasks.state.selected() != None;

    let chunks = Layout::default()
        .constraints(
            [Constraint::Percentage(100), Constraint::Percentage(100)].as_ref(),
        )
        .direction(Direction::Horizontal)
        .vertical_margin(2)
        .horizontal_margin(if task_selected { 1 } else { 3 })
        .split(area);

    let tasks = List::new(tasks)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Magenta),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);
}

fn draw_loading<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [Constraint::Percentage(100), Constraint::Percentage(100)].as_ref(),
        )
        .direction(Direction::Horizontal)
        .vertical_margin(2)
        .horizontal_margin(3)
        .split(area);

    let simple = throbber_widgets_tui::Throbber::default()
        .label("Loading tasks...")
        .style(tui::style::Style::default().fg(tui::style::Color::Cyan));
    f.render_widget(simple, chunks[0]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
