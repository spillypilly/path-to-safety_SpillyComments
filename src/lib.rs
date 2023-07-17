use rand::Rng;

pub const NUM_RANKS: usize = 13;
pub const NUM_SUITS: usize = 4;
pub const NUM_JOKERS: usize = 2;
pub const NUM_CARDS: usize = NUM_RANKS * NUM_SUITS + NUM_JOKERS;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rank(u8);
impl Rank {
    #[must_use]
    pub fn value(&self) -> u8 {
        self.0 + 1
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

#[derive(Clone, Copy)]
pub struct PathLength(Rank);

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
                // TODO: Shuffle
            }
        }
        self.cards.pop()
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
}
