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
    widgets::{Block, Borders, ListState, List, ListItem, Row, Cell, Table, BorderType },
    Frame, Terminal,
};

use crate::transaction;

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

    let res = run_app(&mut terminal);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut month_list_state = ListState::default();
    month_list_state.select(Some(transaction::get_months().expect("can get months").len() - 1));
    loop {
        terminal.draw(|f| ui(f, &mut month_list_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(())
                },
                KeyCode::Char('n') => {
                    if let Some(selected) = month_list_state.selected() {
                        let amount_months = transaction::get_months().expect("can open files").len();
                        if selected >= amount_months - 1 {
                            month_list_state.select(Some(0))
                        } else {
                            month_list_state.select(Some(selected + 1))
                        }
                    }
                },
                KeyCode::Char('p') => {
                    if let Some(selected) = month_list_state.selected() {
                        let amount_months = transaction::get_months().expect("can open files").len();
                        if selected > 0 {
                            month_list_state.select(Some(selected - 1))
                        } else {
                            month_list_state.select(Some(amount_months - 1))
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, month_list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ]
            .as_ref(),
        )
        .split(f.size());
    let (left, right) = render_months(month_list_state);
    f.render_stateful_widget(left, chunks[0], month_list_state);
    f.render_widget(right, chunks[1]);
}

fn render_months<'a>(month_list_state: &mut ListState) -> (List<'a>, Table<'a>) {
    let months = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Months")
        .border_type(BorderType::Plain);

    let month_list = transaction::get_months().unwrap();
    let items: Vec<_> = month_list
        .iter()
        .map(|month| {
            ListItem::new(Spans::from(vec![Span::styled(
                        month.clone(),
                        Style::default(),
                        )]))
        })
    .collect();
    let selected_month = month_list
        .get(
            month_list_state
            .selected()
            .expect("there is always a selected month")
            )
        .expect("exists")
        .clone();

    let list = List::new(items).block(months).highlight_style(
        Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
        );

    let mut rows: Vec<Row> = Vec::new();
    let poss_month = Some(format!("{}-01", selected_month));
    let month_entries = transaction::get_transactions_for_month(&poss_month).expect("can get transactions");

    for transaction in month_entries {
        let row = Row::new(vec![
                           Cell::from(Span::raw(transaction.date.to_string())),
                           Cell::from(Span::raw(transaction.amount.to_string())),
                           Cell::from(Span::raw(transaction.description)),
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
        ]) ;

    (list, month_detail)
}
