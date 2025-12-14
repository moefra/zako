use hone::node::Persistent;

use crate::{context::BuildContext, intern::InternedString};

impl Persistent<BuildContext> for InternedString {
    type Persisted = String;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        ctx.interner().resolve(self).to_string()
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        ctx.interner().get_or_intern(p.as_str())
    }
}

impl Persistent<BuildContext> for Vec<InternedString> {
    type Persisted = Vec<String>;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        self.iter()
            .map(|s| ctx.interner().resolve(s).to_string())
            .collect()
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        p.into_iter()
            .map(|s| ctx.interner().get_or_intern(s.as_str()))
            .collect()
    }
}

impl Persistent<BuildContext> for Option<InternedString> {
    type Persisted = Option<String>;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        self.map(|s| ctx.interner().resolve(&s).to_string())
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        p.map(|s| ctx.interner().get_or_intern(s.as_str()))
    }
}
