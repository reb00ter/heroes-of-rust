#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Gold,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceBag {
    pub gold: u32,
}

impl ResourceBag {
    #[must_use]
    pub fn gold(amount: u32) -> Self {
        Self { gold: amount }
    }

    pub fn add(&mut self, other: &ResourceBag) {
        self.gold += other.gold;
    }

    pub fn subtract(&mut self, other: &ResourceBag) -> Result<(), InsufficientFunds> {
        if self.gold < other.gold {
            return Err(InsufficientFunds);
        }
        self.gold -= other.gold;
        Ok(())
    }

    #[must_use]
    pub fn can_afford(&self, cost: &ResourceBag) -> bool {
        self.gold >= cost.gold
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsufficientFunds;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_resources() {
        let mut bag = ResourceBag::gold(100);
        bag.add(&ResourceBag::gold(50));
        assert_eq!(bag.gold, 150);
    }

    #[test]
    fn subtract_sufficient() {
        let mut bag = ResourceBag::gold(100);
        assert!(bag.subtract(&ResourceBag::gold(40)).is_ok());
        assert_eq!(bag.gold, 60);
    }

    #[test]
    fn subtract_insufficient() {
        let mut bag = ResourceBag::gold(10);
        assert_eq!(bag.subtract(&ResourceBag::gold(20)), Err(InsufficientFunds));
        assert_eq!(bag.gold, 10); // не изменился
    }

    #[test]
    fn can_afford() {
        let bag = ResourceBag::gold(100);
        assert!(bag.can_afford(&ResourceBag::gold(100)));
        assert!(!bag.can_afford(&ResourceBag::gold(101)));
    }
}
