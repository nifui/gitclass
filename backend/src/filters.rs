//Holding file for filtering stuff. Will work on later.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Range<T> {
    pub lower: T,
    pub upper: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Condition<T> {
    Range(Range<T>),
    EqualTo(T),
    LessThan(T),
    GreaterThan(T),
}
impl<T: PartialOrd> Condition<T> {
    pub fn matches(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        match self {
            Self::Range(range) => value >= &range.lower && value <= &range.upper,
            Self::EqualTo(expected) => value == expected,
            Self::LessThan(bound) => value < bound,
            Self::GreaterThan(bound) => value > bound,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StudentFilter {
    pub grade: Option<Condition<i32>>,
    pub name: Option<Condition<String>>,
}
pub struct ClassFilter {}
