use rand::Rng;

pub const NUM_RANKS: u8 = 13;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rank(u8);

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
        let showing = u8::try_from(self.0.count_ones()).expect("There aren't that many bits");
        let not_showing = NUM_RANKS - showing;
        if not_showing <= 1 {
            return None;
        }

        let mut show = rand::thread_rng().gen_range(0..not_showing - 1);
        for i in 0..NUM_RANKS {
            let r = Rank(i);
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
}
