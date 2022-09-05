use crate::transaction::{self, Transaction};
use std::fmt;
use std::error::Error;
use chrono::NaiveDate;
use tui::widgets::{TableState, ListState};

pub enum ActionState {
    Normal,
    Add(AddState, Transaction),
    Update(UpdateState, Transaction),
}

#[derive(Debug, Clone, Copy)]
pub enum AddState {
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

#[derive(Debug, Clone, Copy)]
pub enum UpdateState {
    Date,
    Amount,
    Description,
}

impl fmt::Display for UpdateState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct App {
    pub months: Vec<String>,
    pub current_month: NaiveDate,
    pub month_state: ListState,
    pub transaction_state: TableState,
    pub transactions: Vec<Transaction>,
    pub input: String,
    pub state: ActionState,
}

impl App {
    pub fn refresh_transactions(&mut self) {
        self.refresh_current_month();
        self.transactions =
            get_transactions_for_selected_month(&self.month_state).expect("can get transactions");
    }

    pub fn refresh_months(&mut self) {
        self.months = transaction::get_months().unwrap_or_default();
    }

    pub fn set_input_to_sum(&mut self) {
        if let Ok(sum) = transaction::get_formatted_sum_for_month(&self.current_month) {
            self.input = format!("Sum for current month: {}", sum);
        } else {
            self.input = String::new();
        }
    }

    fn refresh_current_month(&mut self) {
        let month_without_day =
            &self.months[self.month_state.selected().expect("something is selected")];
        let month_with_day = format!("{}-01", month_without_day);
        self.current_month = transaction::get_date(&month_with_day).expect("months are correct");
    }
}

impl Default for App {
    fn default() -> App {
        let mut app = App {
            months: transaction::get_months().unwrap_or_default(),
            current_month: NaiveDate::default(),
            transactions: Vec::new(),
            input: String::new(),
            month_state: ListState::default(),
            transaction_state: TableState::default(),
            state: ActionState::Normal,
        };
        app.month_state.select(Some(app.months.len() - 1));
        app.refresh_current_month();
        app.transaction_state.select(Some(0));
        app.transactions =
            get_transactions_for_selected_month(&app.month_state).unwrap_or_default();
        app
    }
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
