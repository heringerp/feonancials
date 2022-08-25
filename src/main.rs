use clap::{Parser, Subcommand};

mod date_serializer;
mod transaction;

#[derive(Parser)]
struct Arguments {
    #[clap(subcommand)]
    command: Option<Commands>

}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(value_parser)]
        amount: f64,

        #[clap(value_parser)]
        description: String
    },

    List {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(long, short, action)]
        full: bool,
    }
}


fn main() {
    let arg = Arguments::parse();
    let command = &arg.command;
    if command.is_none() {
    } else {
        let res = match &arg.command.unwrap() {
            Commands::Add { date, amount, description } => transaction::add_date_entry(date, *amount, description),
            Commands::List { date, full } => transaction::print_date_list(date, *full),
        };
        if let Err(r) = res {
            eprintln!("{}", r);
            std::process::exit(1)
        }
    }
}
