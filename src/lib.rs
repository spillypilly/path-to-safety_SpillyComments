use rand::seq::SliceRandom;
use rand::Rng;

pub const NUM_RANKS: usize = 13;
pub const NUM_SUITS: usize = 4;
pub const NUM_JOKERS: usize = 2;
pub const NUM_CARDS: usize = NUM_RANKS * NUM_SUITS + NUM_JOKERS;

pub const STARTING_CARDS: u8 = 3;
pub const STARTING_MAD_SCIENCE_TOKENS: i8 = 15;
pub const STARTING_PROGRESS: i8 = -10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rank(u8);
impl Rank {
    #[must_use]
    pub fn value(&self) -> u8 {
        self.0 + 1
    }
    #[must_use]
    pub fn is_face(&self) -> bool {
        self.value() > 10
    }
    #[must_use]
    pub fn random() -> Self {
        Self(
            rand::thread_rng()
                .gen_range(0..NUM_RANKS)
                .try_into()
                .expect("Too many ranks?"),
        )
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Suit(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Card(u8);
impl Card {
    #[must_use]
    pub fn is_joker(&self) -> bool {
        usize::from(self.0) >= NUM_RANKS * NUM_SUITS
    }
    #[must_use]
    pub fn rank(&self) -> Option<Rank> {
        (!self.is_joker()).then_some(Rank(self.0 >> 2))
    }
    #[must_use]
    pub fn suit(&self) -> Option<Suit> {
        (!self.is_joker()).then_some(Suit(self.0 & 3))
    }
}

#[derive(Clone, Copy)]
pub enum WithOrWithoutJokers {
    WithJokers,
    WithoutJokers,
}

#[must_use]
pub fn deck(j: WithOrWithoutJokers) -> Vec<Card> {
    let limit = u8::try_from(match j {
        WithOrWithoutJokers::WithJokers => NUM_CARDS,
        WithOrWithoutJokers::WithoutJokers => NUM_SUITS * NUM_RANKS,
    })
    .expect("Too many cards?");
    (0..limit).map(Card).collect()
}

fn shuffle(cards: &mut Vec<Card>) {
    cards.shuffle(&mut rand::thread_rng());
}
#[must_use]
fn shuffled(mut cards: Vec<Card>) -> Vec<Card> {
    shuffle(&mut cards);
    cards
}

#[derive(Clone, Copy, Debug)]
pub struct PathLength(Rank);
impl PathLength {
    #[must_use]
    pub fn random() -> Self {
        Self(Rank::random())
    }
}

#[derive(Clone, Copy, Default)]
pub struct PathLengthInfo(u16);
impl PathLengthInfo {
    #[must_use]
    pub fn is_showing(&self, i: Rank) -> bool {
        (self.0 >> i.0) & 1 == 1
    }
    fn reveal(&mut self, i: Rank) {
        self.0 |= 1 << i.0;
    }
    pub fn reveal_random(&mut self, true_length: PathLength) -> Option<Rank> {
        let showing = usize::try_from(self.0.count_ones()).expect("There aren't that many bits");
        let not_showing = NUM_RANKS - showing;
        if not_showing <= 1 {
            return None;
        }

        let mut show = rand::thread_rng().gen_range(0..not_showing - 1);
        for i in 0..NUM_RANKS {
            let r = Rank(u8::try_from(i).expect("Too many cards?"));
            if !self.is_showing(r) && r != true_length.0 {
                if show == 0 {
                    self.reveal(r);
                    return Some(r);
                }
                show -= 1;
            }
        }
        unreachable!()
    }
}

#[derive(Default)]
pub struct Discard {
    cards: Vec<Card>,
}
impl Discard {
    pub fn discard(&mut self, card: Card) {
        self.cards.push(card);
    }
    #[must_use]
    pub fn top(&self) -> Option<&Card> {
        self.cards.last()
    }
    fn len(&self) -> usize {
        self.cards.len()
    }
}

pub struct Library {
    cards: Vec<Card>,
}
impl Library {
    #[must_use]
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }
    pub fn draw(&mut self, discard: &mut Discard) -> Option<Card> {
        if self.cards.is_empty() {
            if let Some(top_discard) = discard.cards.pop() {
                std::mem::swap(&mut self.cards, &mut discard.cards);
                discard.discard(top_discard);
                shuffle(&mut self.cards);
            }
        }
        self.cards.pop()
    }
    fn len(&self) -> usize {
        self.cards.len()
    }
}

#[derive(Debug, Default)]
pub struct Hand {
    cards: Vec<Card>,
}
impl Hand {
    fn add(&mut self, card: Card) {
        self.cards.push(card);
    }
    fn remove(&mut self, card: Card) -> Result<(), &'static str> {
        let i = self
            .cards
            .iter()
            .position(|&e| e == card)
            .ok_or("That card is not in your hand")?;
        self.cards.swap_remove(i);
        Ok(())
    }
    fn len(&self) -> usize {
        self.cards.len()
    }
    fn random(&self) -> Option<&Card> {
        self.cards.choose(&mut rand::thread_rng())
    }
    /// Make a new Hand that contains only cards of the requested suit
    fn filter_by_suit(&self, suit: Suit) -> Self {
        Self {
            cards: self
                .cards
                .iter()
                .filter(|c| c.suit().expect("I shouldn't have jokers in my hand") == suit)
                .copied()
                .collect(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct PlayerIndex(usize);
impl PlayerIndex {
    fn next(self, num_players: usize) -> Self {
        Self((self.0 + 1) % num_players)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Play {
    Play(Card),
    Draw,
}

#[derive(Eq, PartialEq)]
pub enum Phase {
    Play,
    Momentum,
}

#[derive(Debug)]
pub enum GameOutcome {
    Loss,
    Win,
}

pub enum PlayOutcome {
    Continue,
    End(GameOutcome),
}

pub struct Game {
    mad_science_tokens: i8,
    progress: [i8; NUM_SUITS],
    path_lengths: [PathLength; NUM_SUITS],
    path_length_info: [PathLengthInfo; NUM_SUITS],
    library: Library,
    discard: Discard,
    hands: Vec<Hand>,
    turn: PlayerIndex,
    phase: Phase,
}
impl Game {
    pub fn add_player(&mut self) {
        self.hands.push(Hand::default());
        for _ in 0..STARTING_CARDS {
            self.draw_for_player(PlayerIndex(self.hands.len() - 1));
        }
    }
    /// # Errors
    ///
    /// Will return `Err` on invalid plays, like trying to draw during Play phase,
    /// or trying to play a card that's not in your hand.
    pub fn play(&mut self, play: Play) -> Result<PlayOutcome, &'static str> {
        match play {
            Play::Play(card) => self.play_card(card),
            Play::Draw => self.draw_for_momentum(),
        }
    }

    #[must_use]
    pub fn current_player_hand(&self) -> &Hand {
        &self.hands[self.turn.0]
    }
    fn player_hand_mut(&mut self, pi: PlayerIndex) -> &mut Hand {
        &mut self.hands[pi.0]
    }
    fn current_player_hand_mut(&mut self) -> &mut Hand {
        self.player_hand_mut(self.turn)
    }

    fn play_card(&mut self, card: Card) -> Result<PlayOutcome, &'static str> {
        let momentum = self.apply_card(card)?;
        if self.phase == Phase::Play && momentum {
            self.phase = Phase::Momentum;
            Ok(PlayOutcome::Continue)
        } else {
            Ok(self.end_of_turn())
        }
    }
    fn draw_for_momentum(&mut self) -> Result<PlayOutcome, &'static str> {
        if self.phase != Phase::Momentum {
            return Err("You don't have momentum");
        }
        self.draw_for_player(self.turn);
        Ok(self.end_of_turn())
    }

    fn draw_for_player(&mut self, pi: PlayerIndex) {
        loop {
            if let Some(card) = self.library.draw(&mut self.discard) {
                if card.is_joker() {
                    self.remove_mad_science_token();
                    self.discard.discard(card);
                } else {
                    self.player_hand_mut(pi).add(card);
                    break;
                }
            } else {
                println!("Library ran out of cards");
            }
        }
    }
    fn remove_mad_science_token(&mut self) {
        loop {
            self.mad_science_tokens -= 1;
            if self.mad_science_tokens != 0 {
                break;
            }
        }
    }
    fn make_progress(&mut self, card: Card) {
        let rank = card.rank().expect("Can't play jokers").0;
        if rank < 6 {
            let roll = rand::thread_rng().gen_range(1..=6);
            if roll > rank {
                self.remove_mad_science_token();
            }
        }
        self.progress[usize::from(card.suit().expect("Can't play jokers").0)] += 1;
    }
    fn forecast(&mut self, card: Card) {
        let suit = usize::from(card.suit().expect("Can't play jokers").0);
        self.path_length_info[suit].reveal_random(self.path_lengths[suit]);
    }
    // Returns whether or not this play grants momentum
    fn apply_card(&mut self, card: Card) -> Result<bool, &'static str> {
        self.current_player_hand_mut().remove(card)?;
        if card.rank().expect("Can't play jokers").is_face() {
            self.forecast(card);
        } else {
            self.make_progress(card);
        }
        let suits_match = self
            .discard
            .top()
            .map_or(false, |dis| dis.suit() == card.suit());
        self.discard.discard(card);
        Ok(suits_match)
    }
    fn valid(&self) -> bool {
        108 == (self.library.len()
            + self.discard.len()
            + self.hands.iter().map(Hand::len).sum::<usize>())
    }
    fn roll_mad_science(&mut self) -> PlayOutcome {
        let mut tokens = std::iter::from_fn(|| Some(rand::thread_rng().gen_bool(0.5)))
            .take(usize::try_from(self.mad_science_tokens.abs()).expect("wat?"));
        let keep_going = if self.mad_science_tokens > 0 {
            tokens.any(|t| !t)
        } else {
            tokens.all(|t| !t)
        };
        if keep_going {
            PlayOutcome::Continue
        } else {
            PlayOutcome::End(self.final_score())
        }
    }
    fn final_score(&self) -> GameOutcome {
        if self
            .progress
            .iter()
            .zip(self.path_lengths.iter())
            .any(|(&prog, len)| prog >= len.0.value().try_into().expect("wat?"))
        {
            GameOutcome::Win
        } else {
            GameOutcome::Loss
        }
    }
    fn end_of_turn(&mut self) -> PlayOutcome {
        assert!(self.valid());
        self.phase = Phase::Play;
        self.turn = self.turn.next(self.hands.len());
        if self.turn.0 == 0 {
            if let PlayOutcome::End(game_outcome) = self.roll_mad_science() {
                return PlayOutcome::End(game_outcome);
            }
        }
        self.draw_for_player(self.turn);
        assert!(self.valid());
        PlayOutcome::Continue
    }
}
impl Default for Game {
    fn default() -> Self {
        Self {
            mad_science_tokens: STARTING_MAD_SCIENCE_TOKENS,
            progress: [STARTING_PROGRESS; NUM_SUITS],
            path_lengths: std::iter::from_fn(|| Some(PathLength::random()))
                .take(NUM_SUITS)
                .collect::<Vec<_>>()
                .try_into()
                .expect("wat?"),
            path_length_info: [PathLengthInfo::default(); NUM_SUITS],
            library: Library::new(shuffled(
                [
                    deck(WithOrWithoutJokers::WithJokers),
                    deck(WithOrWithoutJokers::WithJokers),
                ]
                .concat(),
            )),
            discard: Discard::default(),
            hands: vec![],
            turn: PlayerIndex(0),
            phase: Phase::Play,
        }
    }
}

pub struct Player(Box<dyn FnMut(&Game) -> Play>);
impl Player {
    #[must_use]
    pub fn new<T>(f: T) -> Self
    where
        T: FnMut(&Game) -> Play + 'static,
    {
        Self(Box::new(f))
    }
}

#[must_use]
pub fn random_player(draw_chance: f64) -> Player {
    Player(Box::new(move |game: &Game| -> Play {
        match game.phase {
            Phase::Play => Play::Play(
                *game
                    .current_player_hand()
                    .random()
                    .expect("I always have a card to play because I just drew one"),
            ),
            Phase::Momentum => {
                if rand::thread_rng().gen_bool(draw_chance) {
                    Play::Draw
                } else {
                    match game.current_player_hand().random() {
                        Some(card) => Play::Play(*card),
                        None => Play::Draw,
                    }
                }
            }
        }
    }))
}

/// When available, make plays that grant momentum.
#[must_use]
pub fn momentum_player(mut fallback: Player) -> Player {
    Player(Box::new(move |game: &Game| -> Play {
        if game.phase == Phase::Play {
            if let Some(suit) = game.discard.top().and_then(Card::suit) {
                if let Some(card) = game.current_player_hand().filter_by_suit(suit).random() {
                    return Play::Play(*card);
                }
            }
        }
        fallback.0(game)
    }))
}

/// # Errors
///
/// Will return `Err` on invalid plays, like trying to draw during Play phase,
/// or trying to play a card that's not in your hand.
pub fn play(mut game: Game, mut players: Vec<Player>) -> Result<GameOutcome, &'static str> {
    game.draw_for_player(game.turn);
    loop {
        if let PlayOutcome::End(game_outcome) = game.play(players[game.turn.0].0(&game))? {
            return Ok(game_outcome);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_length_info_random_reveal() {
        let length = PathLength(Rank(7));
        let mut pli = PathLengthInfo::default();
        for _ in 0..12 {
            let old_pli = PathLengthInfo::clone(&pli);
            match pli.reveal_random(length) {
                None => panic!("Nothing revealed?"),
                Some(r) => {
                    assert!(!old_pli.is_showing(r));
                    assert!(pli.is_showing(r));
                }
            }
            assert_eq!(pli.0.count_ones(), 1 + old_pli.0.count_ones());
        }
        assert!(pli.reveal_random(length).is_none());
    }

    #[test]
    fn test_deck() {
        use WithOrWithoutJokers::*;
        let d = deck(WithoutJokers);
        let rank_sum: u32 = d
            .iter()
            .map(Card::rank)
            .flatten()
            .map(|r| u32::from(r.value()))
            .sum();
        assert_eq!(rank_sum, 364);
        let _dj = deck(WithJokers);
    }

    #[test]
    fn test_library() {
        let mut lib = Library::new(vec![Card(7)]);
        let mut dis = Discard::default();
        dis.discard(Card(8));
        dis.discard(Card(9));
        assert_eq!(lib.draw(&mut dis), Some(Card(7)));
        assert_eq!(lib.draw(&mut dis), Some(Card(8)));
        assert_eq!(lib.draw(&mut dis), None);
    }

    #[test]
    fn test_hand() {
        let mut h = Hand::default();
        assert!(h.remove(Card(4)).is_err());
        h.add(Card(4));
        assert!(h.remove(Card(3)).is_err());
        assert!(h.remove(Card(4)).is_ok());
        assert!(h.remove(Card(4)).is_err());
    }

    #[test]
    fn test_game() {
        for num_players in 1..10 {
            let players: Vec<_> = std::iter::from_fn(|| Some(momentum_player(random_player(0.5))))
                .take(num_players)
                .collect();
            let mut game = Game::default();
            for _ in 0..num_players {
                game.add_player();
            }
            assert!(play(game, players).is_ok());
        }
    }
}
