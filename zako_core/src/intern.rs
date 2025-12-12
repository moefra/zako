use crate::context::BuildContext;

pub type InternedString = ::lasso::Spur;
pub type Interner = dyn ::lasso::Interner<InternedString>;

pub trait Internable<C> {
    type Interned;

    fn intern(self, context: &C) -> Self::Interned;
    fn resolve(interned: &Self::Interned, context: &C) -> Self;
}

impl Internable<BuildContext> for String {
    type Interned = InternedString;

    fn intern(self, context: &BuildContext) -> InternedString {
        context.interner().get_or_intern(self)
    }

    fn resolve(interned: &InternedString, context: &BuildContext) -> String {
        context.interner().resolve(interned).to_string()
    }
}

impl Internable<BuildContext> for Vec<String> {
    type Interned = Vec<InternedString>;

    fn intern(self, context: &BuildContext) -> Vec<InternedString> {
        self.into_iter()
            .map(|s| context.interner().get_or_intern(s))
            .collect()
    }

    fn resolve(interned: &Vec<InternedString>, context: &BuildContext) -> Vec<String> {
        interned
            .iter()
            .map(|s| context.interner().resolve(s))
            .collect()
    }
}
