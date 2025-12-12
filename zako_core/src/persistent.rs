use hone::node::Persistent;

use crate::{context::BuildContext, intern::InternedString};

impl Persistent<BuildContext> for InternedString {
    type Persisted = String;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        ctx.interner().resolve(self)
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        ctx.interner().get_or_intern(p)
    }
}

impl Persistent<BuildContext> for Vec<InternedString> {
    type Persisted = Vec<String>;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        self.iter().map(|s| ctx.interner().resolve(s)).collect()
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        p.into_iter()
            .map(|s| ctx.interner().get_or_intern(s))
            .collect()
    }
}
