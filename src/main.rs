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
        /// For momentum play, draw how often?  0-1
        #[arg(short = 'p', long, default_value_t = 0.5)]
        draw_chance: f64,
        /// How many games?
        #[arg(short = 'g', long, default_value_t = 1)]
        num_games: usize,
        /// How many players?
        #[arg(short = 'p', long)]
        num_players: usize,
    },
}

fn main() -> Result<(), &'static str> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Sim {
            draw_chance,
            num_games,
            num_players,
        }) => {
            for _ in 0..*num_games {
                let players: Vec<_> =
                    std::iter::from_fn(|| Some(Player::new(random_player(*draw_chance))))
                        .take(*num_players)
                        .collect();
                let mut game = Game::default();
                for _ in 0..*num_players {
                    game.add_player();
                }
                let result = play(game, players)?;
                println!("Result: {result:?}");
            }
            Ok(())
        }
        None => unreachable!(),
    }
}
