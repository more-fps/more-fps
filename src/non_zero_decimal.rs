use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NonZeroDecimal(Decimal);

impl NonZeroDecimal {
    pub fn try_new<T>(value: T) -> Option<Self>
    where
        T: TryInto<Decimal>,
    {
        let value = value.try_into().ok()?;
        if value == Decimal::ZERO {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn get(&self) -> &Decimal {
        &self.0
    }
}

impl std::ops::Deref for NonZeroDecimal {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for NonZeroDecimal {
    type Error = rust_decimal::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let decimal = Decimal::from_str_exact(s)?;
        Ok(Self(decimal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_zero_decimal() {
        assert_eq!(
            NonZeroDecimal::try_new(12).map(|d| d.get().clone()),
            Some(Decimal::from_str_exact("12").unwrap())
        );
    }
    #[test]
    fn zero_decimal() {
        assert_eq!(NonZeroDecimal::try_new(0), None);
    }
}
