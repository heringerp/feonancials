use clap::{Parser, Subcommand};

mod date_serializer;
mod transaction;
mod tui;

#[derive(Parser)]
struct Arguments {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(value_parser)]
        amount: f64,

        #[clap(value_parser)]
        description: String,

        #[clap(value_parser)]
        repeat: Option<String>,
    },

    List {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(long, short, action)]
        full: bool,
    },

    Del {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(value_parser)]
        index: usize,
    },

    Menu,
}

fn main() {
    let arg = Arguments::parse();
    let command = &arg.command;
    if command.is_none() {
    } else {
        let res = match &arg.command.unwrap() {
            Commands::Add {
                date,
                amount,
                description,
                repeat,
            } => transaction::add_date_entry(date, *amount, description, repeat),
            Commands::List { date, full } => transaction::print_date_list(date, *full),
            Commands::Del { date, index } => transaction::del_entry(date, *index),
            Commands::Menu => tui::show_tui(),
        };
        if let Err(r) = res {
            eprintln!("{}", r);
            std::process::exit(1)
        }
    }
}
