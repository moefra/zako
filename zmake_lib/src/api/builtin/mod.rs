pub mod c;
mod cpp;

use std::str::FromStr;
    use crate::api::id::{Architecture, ArtifactId, GroupId, Id, Ident, Os, QualifiedArtifactId, TargetType, ToolName, ToolType};
    use semver::Version;
    use std::sync::LazyLock;

/* Basical information about zmake */
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


/* Build stages */
    pub static INITIALIZE: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.initialize").unwrap())
    });
    pub static CLEAN: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.clean").unwrap())
    });
    pub static BUILD: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.build").unwrap())
    });
    pub static TEST: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.test").unwrap())
    });
    pub static PACKAGE: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.package").unwrap())
    });
    pub static INSTALL: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.install").unwrap())
    });
    pub static DEPLOY: LazyLock<TargetType> = LazyLock::new(|| {
        TargetType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.target_type.deploy").unwrap())
    });

/* Architectures */
    pub static X64: LazyLock<Architecture> = LazyLock::new(|| {
        Architecture(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.architecture.x64").unwrap())
    });

    pub static ARM64: LazyLock<Architecture> = LazyLock::new(|| {
        Architecture(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.architecture.arm64").unwrap())
    });


/* Operating systems */
    pub static WINDOWS: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.os.windows").unwrap())
    });
    pub static LINUX: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.os.linux").unwrap())
    });
    pub static MACOS: LazyLock<Os> = LazyLock::new(|| {
        Os(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.os.macos").unwrap())
    });

/* Common tools */
    pub static ARCHIVER: LazyLock<ToolType> = LazyLock::new(|| {
        ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_type.archiver").unwrap())
    });

pub static DOWNLOADER: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_type.downloader").unwrap())
});

pub static LINKER: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_type.linker").unwrap())
});

/// link.exe
pub static LINK_EXE: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_name.link_exe").unwrap())
});

pub static LD: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_name.ld").unwrap())
});

pub static BSD_TAR: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_name.bad_tar").unwrap())
});

pub static CURL: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_name.curl").unwrap())
});

pub static GIT: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.tool_name.git").unwrap())
});
