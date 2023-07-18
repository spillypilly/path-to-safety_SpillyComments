use clap::{Parser, Subcommand};
use pluta_lesnura::{play, random_player, Game, Player};

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs simulations
    Sim {
        /// How many players?
        #[arg(short, long)]
        num_players: usize,
    },
}

fn main() -> Result<(), &'static str> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Sim { num_players }) => {
            let players: Vec<_> = std::iter::from_fn(|| Some(Player::new(random_player)))
                .take(*num_players)
                .collect();
            let mut game = Game::default();
            for _ in 0..*num_players {
                game.add_player();
            }
            let result = play(game, players)?;
            println!("Result: {result:?}");
            Ok(())
        }
        None => unreachable!(),
    }
}
