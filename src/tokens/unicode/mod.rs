//! The Unicode tables, one module per version of the Unicode character database.
//!
//! Each `vNN` module holds the tables `generate-classes.sh` generated for that UCD
//! version, and the `ucd-NN` Cargo feature selects it. Only the selected version is
//! compiled: the others cost nothing in build time or in the binary.
//!
//! Cargo features are additive, so two of them can end up enabled at once when several
//! dependents of this crate ask for different versions. The highest one then wins,
//! which at least makes the result the same whatever the order they were enabled in.
//! A build that enables none of them, which is what `default-features = false` gives,
//! gets the newest version rather than failing to compile.
//!
//! Adding a version means generating its tables with `generate-classes.sh`, declaring
//! the feature in `Cargo.toml`, and adding a block below. The guards are what make the
//! selection unique: the newest version is taken when its own feature is enabled or
//! when no older one is, and every older version when its feature is enabled and no
//! newer one is. So a new version takes over the `not(any(..))` fallback from the one
//! it displaces, which then joins the list every older guard excludes.

// The newest version, and the fallback when no feature names one.
#[cfg(any(feature = "ucd-17", not(feature = "ucd-16")))]
pub(super) mod v17;
#[cfg(any(feature = "ucd-17", not(feature = "ucd-16")))]
pub(super) use v17 as active;

#[cfg(all(feature = "ucd-16", not(feature = "ucd-17")))]
pub(super) mod v16;
#[cfg(all(feature = "ucd-16", not(feature = "ucd-17")))]
pub(super) use v16 as active;

#[cfg(test)]
mod tests {
    /// The selection above is `cfg`, so a version wired to the wrong feature would go
    /// unnoticed: the crate would still compile, against the wrong tables. Enabling no
    /// feature at all has to reach the newest version rather than none of them, which
    /// would not compile, or two, which would not either.
    #[test]
    fn the_active_version_is_the_highest_enabled_feature() {
        let expected = if cfg!(feature = "ucd-16") && !cfg!(feature = "ucd-17") {
            "16.0.0"
        } else {
            "17.0.0"
        };

        assert_eq!(expected, super::active::UCD_VERSION);
    }
}
