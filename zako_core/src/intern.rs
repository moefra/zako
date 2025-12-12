pub type InternedString = ::lasso::Spur;
pub type Interner = dyn ::lasso::Interner<InternedString>;

pub trait Internable<C> {
    type Interned;

    fn intern(self, context: &C) -> Self::Interned;
    fn deintern(interned: Self::Interned, context: &C) -> Self;
}
