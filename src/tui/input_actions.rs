use crate::transaction::{self, Transaction};
use crate::tui::app::{App, ActionState, AddState, UpdateState};

pub fn add_enter(app: &mut App) {
    match app.state {
        ActionState::Add(ref mut state, ref mut transaction) => match state {
            AddState::Date => {
                let poss_date = match app.input.is_empty() {
                    true => None,
                    false => Some(app.input.clone()),
                };
                if let Ok(date) = transaction::get_date_or_today(&poss_date) {
                    transaction.date = date;
                    *state = AddState::Amount;
                }
                app.input = String::new();
            }
            AddState::Amount => {
                *state = AddState::Description;
                let amount: f64 = app.input.parse().expect("can get amount");
                transaction.amount = amount;
                app.input = String::new();
            }
            AddState::Description => {
                *state = AddState::Date;
                transaction.description = app.input.clone();
                transaction::add_transaction(transaction.clone()).expect("can write transaction");
                *transaction = Transaction::default();
                app.state = ActionState::Normal;
                app.input = "Added entry successfully".to_string();
                app.refresh_months();
                app.refresh_transactions();
            }
        },
        _ => {}
    }
}

pub fn update_enter(app: &mut App) {
    match app.state {
        ActionState::Update(ref mut state, ref mut transaction) => match state {
            UpdateState::Date => {
                *state = UpdateState::Amount;
                let poss_date = match app.input.is_empty() {
                    true => None,
                    false => Some(app.input.clone()),
                };
                transaction.date =
                    transaction::get_date_or_today(&poss_date).expect("can get date");
                app.input = transaction.amount.to_string();
            }
            UpdateState::Amount => {
                *state = UpdateState::Description;
                let amount: f64 = app.input.parse().expect("can get amount");
                transaction.amount = amount;
                app.input = transaction.description.to_string();
            }
            UpdateState::Description => {
                *state = UpdateState::Date;
                transaction.description = app.input.clone();
                app.transactions[app.transaction_state.selected().expect("can get selected")] =
                    transaction.clone();
                transaction::write_transactions(&mut app.transactions).expect("can write");
                app.state = ActionState::Normal;
                app.input = "Updated entry successfully".to_string();
                app.refresh_months();
                app.refresh_transactions();
            }
        },
        _ => {}
    }
}

