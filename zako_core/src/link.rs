use camino::Utf8Path;

/// reflink or hard-link the file
pub fn ref_or_hard_link_file(from: &Utf8Path, to: &Utf8Path) -> std::io::Result<()> {
    let result = reflink_copy::reflink(from, to);

    if result.is_err() {
        std::fs::hard_link(from, to)?;

        // copy maybe too slow, so we even wont try
    }

    Ok(())
}
