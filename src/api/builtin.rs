pub mod zmake {
    use std::str::FromStr;
    use crate::api::id::{ArtifactId, GroupId, Ident, QualifiedArtifactId};
    use semver::Version;
    use std::sync::LazyLock;

    pub static KAWAYI_GROUP_ID: LazyLock<GroupId> = LazyLock::new(|| {
        GroupId::from_str("moe.kawayi").unwrap()
    });

    pub static ZMAKE_ARTIFACT_ID: LazyLock<ArtifactId> =
        LazyLock::new(|| ArtifactId::from((*KAWAYI_GROUP_ID).clone(), Ident::from("zmake").unwrap()));

    pub static ZMAKE_QUALIFIED_ARTIFACT_ID: LazyLock<QualifiedArtifactId> = LazyLock::new(|| {
        QualifiedArtifactId::from(
            (&*ZMAKE_ARTIFACT_ID).clone(),
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        )
    });

    pub static ZMAKE_V1V0V0: LazyLock<QualifiedArtifactId> = LazyLock::new(|| {
        QualifiedArtifactId::from(
            (&*ZMAKE_ARTIFACT_ID).clone(),
            Version::parse("1.0.0").unwrap(),
        )
    });
}

pub mod target_type{
    use std::str::FromStr;
    use crate::api::id::{ArtifactId, GroupId, Id, Ident, QualifiedArtifactId, TargetType};
    use semver::Version;
    use std::sync::LazyLock;
    use crate::api::builtin::zmake::ZMAKE_V1V0V0;

    pub static INITIALIZE: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.initialize").unwrap())
    });
    pub static CLEAN: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.clean").unwrap())
    });
    pub static BUILD: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.build").unwrap())
    });
    pub static TEST: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.test").unwrap())
    });
    pub static PACKAGE: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.package").unwrap())
    });
    pub static INSTALL: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.install").unwrap())
    });
    pub static DEPLOY: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("kawayi.moe:zmake@1.0.0#target_type.deploy").unwrap())
    });
}

pub mod architecture{
    use std::str::FromStr;
    use std::sync::LazyLock;
    use crate::api::id::{Architecture, Id, TargetType};

    pub static X64: LazyLock<Architecture> = LazyLock::new(|| {
        Architecture(Id::from_str("kawayi.moe:zmake@1.0.0#architecture.x64").unwrap())
    });

    pub static ARM64: LazyLock<Architecture> = LazyLock::new(|| {
        Architecture(Id::from_str("kawayi.moe:zmake@1.0.0#architecture.arm64").unwrap())
    });
}

pub mod os{
    use std::str::FromStr;
    use std::sync::LazyLock;
    use crate::api::id::{Id, Os};

    pub static WINDOWS: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("kawayi.moe:zmake@1.0.0#os.windows").unwrap())
    });
    pub static LINUX: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("kawayi.moe:zmake@1.0.0#os.linux").unwrap())
    });
    pub static MACOS: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("kawayi.moe:zmake@1.0.0#os.macos").unwrap())
    });
}

pub mod tool_type{
    use std::str::FromStr;
    use std::sync::LazyLock;
    use crate::api::id::{Id, TargetType, ToolType};

    pub static ARCHIVER: LazyLock<ToolType> = LazyLock::new(|| {
        ToolType(Id::from_str("kawayi.moe:zmake@1.0.0#tool_type.archiver").unwrap())
    });
}