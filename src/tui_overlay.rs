use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::{fmt, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState,
    },
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum ActionState {
    Normal,
    Add(AddState, Transaction),
    Update,
}

#[derive(Debug, Clone, Copy)]
enum AddState {
    Date,
    Amount,
    Description,
}

impl fmt::Display for AddState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

struct App {
    months: Vec<String>,
    month_state: ListState,
    transaction_state: TableState,
    transactions: Vec<Transaction>,
    input: String,
    state: ActionState,
}

impl App {
    fn refresh_transactions(&mut self) {
        self.transactions =
            get_transactions_for_selected_month(&self.month_state).expect("can get transactions");
    }

    fn refresh_months(&mut self) {
        self.months = transaction::get_months().unwrap_or_default();
    }
}

impl Default for App {
    fn default() -> App {
        let mut app = App {
            months: transaction::get_months().unwrap_or_default(),
            transactions: Vec::new(),
            input: String::new(),
            month_state: ListState::default(),
            transaction_state: TableState::default(),
            state: ActionState::Normal,
        };
        app.month_state.select(Some(app.months.len() - 1));
        app.transaction_state.select(Some(0));
        app.transactions =
            get_transactions_for_selected_month(&app.month_state).expect("can get transactions");
        app
    }
}

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
                            app.transactions =
                                get_transactions_for_selected_month(&app.month_state)
                                    .expect("can get transactions");
                            app.transaction_state.select(Some(0));
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
                            app.transactions =
                                get_transactions_for_selected_month(&app.month_state)
                                    .expect("can get transactions");
                            app.transaction_state.select(Some(0));
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
                        let poss_month =
                            Some(get_selected_month(&app.month_state).expect("can get month"));
                        let selected = app
                            .transaction_state
                            .selected()
                            .expect("something is always selected");
                        let amount_transactions = app.transactions.len();
                        transaction::del_entry(&poss_month, selected).expect("can delete entry");
                        if amount_transactions > 1 {
                            if selected == amount_transactions - 1 {
                                app.transaction_state.select(Some(selected - 1))
                            }
                        }
                        app.refresh_transactions()
                    }
                    KeyCode::Char('a') => {
                        app.state = ActionState::Add(AddState::Date, Transaction::default());
                        app.input = "".to_string();
                    }
                    KeyCode::Char('u') => {
                        app.state = ActionState::Update;
                        app.input = "".to_string();
                    }
                    _ => {}
                },
                ActionState::Add(_, _) | ActionState::Update => match key.code {
                    KeyCode::Esc => {
                        app.state = ActionState::Normal;
                    },
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    },
                    KeyCode::Enter => {
                        match app.state {
                            ActionState::Add(_, _) => { add_enter(&mut app) }
                            ActionState::Update => {},
                            _ => {},
                        }
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
        ActionState::Add(_, _) | ActionState::Update => {
            f.set_cursor(
                month_chunks[1].x + width + 1,
                month_chunks[1].y + 1,
            );
        }
    };
}

fn add_enter(app: &mut App) {
    match app.state {
        ActionState::Add(ref mut state, ref mut transaction) => { 
            match state {
                AddState::Date => {
                    *state = AddState::Amount; 
                    let poss_date = match app.input.is_empty() {
                        true => None,
                        false => Some(app.input.clone()),
                    };
                    transaction.date = transaction::get_date_or_today(&poss_date).expect("can get date");
                    app.input = String::new();
                },
                AddState::Amount => {
                    *state = AddState::Description; 
                    let amount: f64 = app.input.parse().expect("can get amount"); 
                    transaction.amount = amount;
                    app.input = String::new();
                },
                AddState::Description => {
                    *state = AddState::Date; 
                    transaction.description = app.input.clone();
                    transaction::add_transaction(transaction.clone()).expect("can write transaction");
                    *transaction = Transaction::default();
                    app.state = ActionState::Normal;
                    app.input = "Added entry successfully".to_string();
                    app.refresh_months();
                    app.refresh_transactions();
                },
            }
        },
        _ => {},
    }
    
}

fn render_info(app: &mut App) -> (Paragraph, u16) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(match app.state {
            ActionState::Normal => "Info",
            ActionState::Add(_, _) => "Add",
            ActionState::Update => "Update",
        })
        .border_type(BorderType::Plain);
    let (paragraph, width) = match app.state {
        ActionState::Normal => render_normal(app),
        ActionState::Add(a, _) => render_add(app, a),
        ActionState::Update => (Paragraph::new(""), 0),
    };
    (paragraph.block(block), width)
}

fn render_normal(app: &mut App) -> (Paragraph, u16) {
    let paragraph = Paragraph::new(app.input.clone())
        .style(Style::default());
    (paragraph, 0)
}

fn render_add(app: &mut App, add_state: AddState) -> (Paragraph, u16) {
    let text = format!("{}: {}", add_state, app.input);
    (Paragraph::new(text.clone()).style(Style::default()), text.width() as u16)
}

fn get_selected_month(month_list_state: &ListState) -> Result<String, Box<dyn Error>> {
    let month_list = transaction::get_months()?;
    let selected_month = month_list
        .get(
            month_list_state
                .selected()
                .expect("there is always a selected month"),
        )
        .expect("exists")
        .clone();
    Ok(format!("{}-01", selected_month))
}

fn get_transactions_for_selected_month(
    month_list_state: &ListState,
) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let month = get_selected_month(month_list_state)?;
    let poss_month = Some(month);
    transaction::get_transactions_for_month(&poss_month)
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
