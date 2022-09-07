use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table,
    },
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

mod app;
mod input_actions;

use app::{App, ActionState, AddState, UpdateState};

use crate::transaction::{self, Transaction};

pub fn show_tui() -> Result<(), Box<dyn Error>> {
    match show_tui_with_io_error() {
        Ok(_) => Ok(()),
        Err(r) => Err(r.into()),
    }
}

fn show_tui_with_io_error() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(r) = res {
        eprintln!("{:?}", r);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.set_input_to_sum();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.state {
                ActionState::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => {
                        if let Some(selected) = app.month_state.selected() {
                            let amount_months = app.months.len();
                            if selected >= amount_months - 1 {
                                app.month_state.select(Some(0))
                            } else {
                                app.month_state.select(Some(selected + 1))
                            }
                            app.refresh_transactions();
                            app.transaction_state.select(Some(0));
                            app.set_input_to_sum();
                        }
                    }
                    KeyCode::Char('p') => {
                        if let Some(selected) = app.month_state.selected() {
                            let amount_months = app.months.len();
                            if selected > 0 {
                                app.month_state.select(Some(selected - 1))
                            } else {
                                app.month_state.select(Some(amount_months - 1))
                            }
                            app.refresh_transactions();
                            app.transaction_state.select(Some(0));
                            app.set_input_to_sum();
                        }
                    }
                    KeyCode::Char('j') => {
                        if let Some(selected) = app.transaction_state.selected() {
                            let amount_transactions = app.transactions.len();
                            if selected >= amount_transactions - 1 {
                                app.transaction_state.select(Some(0))
                            } else {
                                app.transaction_state.select(Some(selected + 1))
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Some(selected) = app.transaction_state.selected() {
                            let amount_transactions = app.transactions.len();
                            if selected > 0 {
                                app.transaction_state.select(Some(selected - 1))
                            } else {
                                app.transaction_state.select(Some(amount_transactions - 1))
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                            // expect is okay, since error only happens when files are out of sync
                            // with application
                        if let Some(selected) = app.transaction_state.selected() {
                            let amount_transactions = app.transactions.len();
                            let result = transaction::del_entry_by_date(&app.current_month, selected);
                            match result {
                                Ok(_) => {
                                    if amount_transactions > 1 {
                                        if selected == amount_transactions - 1 {
                                            app.transaction_state.select(Some(selected - 1))
                                        }
                                    }
                                    app.refresh_transactions();
                                }
                                Err(_) => {
                                    app.input = "Cannot delete entry".to_string();
                                }
                            }
                        } else {
                            app.input = "No entry to delete is selected".to_string();
                        }
                    }
                    KeyCode::Char('a') => {
                        app.state = ActionState::Add(AddState::Date, Transaction::default());
                        app.input = "".to_string();
                    }
                    KeyCode::Char('u') => {
                        let transaction = app.transactions[app
                            .transaction_state
                            .selected()
                            .expect("there is smth. selected")]
                        .clone();
                        app.input = transaction.date.to_string();
                        app.state = ActionState::Update(UpdateState::Date, transaction);
                    }
                    _ => {}
                },
                ActionState::Add(_, _) | ActionState::Update(_, _) => match key.code {
                    KeyCode::Esc => {
                        app.state = ActionState::Normal;
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => match app.state {
                        ActionState::Add(_, _) => input_actions::add_enter(&mut app),
                        ActionState::Update(_, _) => input_actions::update_enter(&mut app),
                        _ => {}
                    },
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());
    let month_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(chunks[1]);
    let (left, right) = render_months(app);
    f.render_stateful_widget(left, chunks[0], &mut app.month_state);
    f.render_stateful_widget(right, month_chunks[0], &mut app.transaction_state);
    let (info, width) = render_info(app);
    f.render_widget(info, month_chunks[1]);

    match app.state {
        ActionState::Normal => {}
        ActionState::Add(_, _) | ActionState::Update(_, _) => {
            f.set_cursor(month_chunks[1].x + width + 1, month_chunks[1].y + 1);
        }
    };
}

fn render_info(app: &mut App) -> (Paragraph, u16) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(match app.state {
            ActionState::Normal => "Info",
            ActionState::Add(_, _) => "Add",
            ActionState::Update(_, _) => "Update",
        })
        .border_type(BorderType::Plain);
    let (paragraph, width) = match app.state {
        ActionState::Normal => render_normal(app),
        ActionState::Add(a, _) => render_add(app, a),
        ActionState::Update(a, _) => render_update(app, a),
    };
    (paragraph.block(block), width)
}

fn render_normal(app: &mut App) -> (Paragraph, u16) {
    let paragraph = Paragraph::new(app.input.clone()).style(Style::default());
    (paragraph, 0)
}

fn render_add(app: &mut App, add_state: AddState) -> (Paragraph, u16) {
    let text = format!("{}: {}", add_state, app.input);
    (
        Paragraph::new(text.clone()).style(Style::default()),
        text.width() as u16,
    )
}

fn render_update(app: &mut App, update_state: UpdateState) -> (Paragraph, u16) {
    let text = format!("{}: {}", update_state, app.input);
    (
        Paragraph::new(text.clone()).style(Style::default()),
        text.width() as u16,
    )
}

fn render_months<'a>(app: &mut App) -> (List<'a>, Table<'a>) {
    let months = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Months")
        .border_type(BorderType::Plain);

    let items: Vec<_> = app
        .months
        .iter()
        .map(|month| {
            ListItem::new(Spans::from(vec![Span::styled(
                month.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let list = List::new(items).block(months).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut rows: Vec<Row> = Vec::new();

    for transaction in &app.transactions {
        let row = Row::new(vec![
            Cell::from(Span::raw(transaction.date.to_string())),
            Cell::from(Span::raw(transaction.amount.to_string())),
            Cell::from(Span::raw(transaction.description.clone())),
        ]);
        rows.push(row)
    }

    let month_detail = Table::new(rows)
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "Date",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Amount",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Description",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Detail")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(70),
        ])
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    (list, month_detail)
}
