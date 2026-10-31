//! The identity of a world on disk: a short lowercase slug and the file names derived from it.

use thiserror::Error;

/// The longest slug, so file names stay short on every file system.
const MAX_LEN: usize = 32;

/// The slug of the world that existed before save slots (ADR 0017): its file is `world.save`.
pub const LEGACY_ID: &str = "world";

/// Why a name cannot become a world id.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorldIdError {
    #[error("a world name needs at least one letter or digit")]
    Empty,
}

/// A world's slug: lowercase ASCII letters, digits and single dashes. It is the only thing that
/// reaches a file name, so a typed name can never escape the save folder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldId(String);

impl WorldId {
    /// The slug for a name the player typed: lowercased, anything else becomes a dash.
    ///
    /// # Errors
    /// Returns [`WorldIdError::Empty`] when the name has no letter or digit.
    pub fn from_name(name: &str) -> Result<Self, WorldIdError> {
        let mut slug = String::new();
        for character in name.chars().map(|c| c.to_ascii_lowercase()) {
            if character.is_ascii_lowercase() || character.is_ascii_digit() {
                slug.push(character);
            } else if !slug.ends_with('-') && !slug.is_empty() {
                slug.push('-');
            }
        }
        slug.truncate(MAX_LEN);
        let slug = slug.trim_end_matches('-');
        if slug.is_empty() {
            return Err(WorldIdError::Empty);
        }
        Ok(Self(slug.to_owned()))
    }

    /// The id read back from a save file name, `None` when the name is not a slug this type
    /// would have made.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        let id = Self::from_name(slug).ok()?;
        (id.0 == slug).then_some(id)
    }

    /// The slug of the world from before slots.
    #[must_use]
    pub fn legacy() -> Self {
        Self(LEGACY_ID.to_owned())
    }

    /// This id with a number after it, for a second world with the same name.
    #[must_use]
    pub fn numbered(&self, number: u32) -> Self {
        let base = self.0.trim_end_matches('-');
        let keep = MAX_LEN.saturating_sub(number.to_string().len() + 1);
        let base: String = base.chars().take(keep).collect();
        Self(format!("{}-{number}", base.trim_end_matches('-')))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn save_file(&self) -> String {
        format!("{}.save", self.0)
    }

    #[must_use]
    pub fn backup_file(&self) -> String {
        format!("{}.save.bak", self.0)
    }

    /// Holds the name the player gave the world, as plain UTF-8 text.
    #[must_use]
    pub fn name_file(&self) -> String {
        format!("{}.name", self.0)
    }
}

impl std::fmt::Display for WorldId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(name: &str) -> WorldId {
        WorldId::from_name(name).expect("a valid name")
    }

    #[test]
    fn a_name_becomes_a_lowercase_slug() {
        assert_eq!(id("My First World!").as_str(), "my-first-world");
    }

    #[test]
    fn runs_of_symbols_become_one_dash_and_the_ends_are_trimmed() {
        assert_eq!(id("  --Isla   &  Mar--  ").as_str(), "isla-mar");
    }

    #[test]
    fn a_name_without_letters_or_digits_is_rejected() {
        assert_eq!(WorldId::from_name(" !? "), Err(WorldIdError::Empty));
        assert_eq!(WorldId::from_name(""), Err(WorldIdError::Empty));
    }

    #[test]
    fn a_name_cannot_escape_the_save_folder() {
        let slug = id("../../etc/passwd");

        assert_eq!(slug.as_str(), "etc-passwd");
        assert!(!slug.save_file().contains('/'));
    }

    #[test]
    fn a_long_name_is_cut_without_a_trailing_dash() {
        let slug = id(&format!("{} tail", "a".repeat(31)));

        assert_eq!(slug.as_str(), "a".repeat(31));
    }

    #[test]
    fn a_numbered_id_stays_within_the_limit() {
        let numbered = id(&"b".repeat(40)).numbered(12);

        assert_eq!(numbered.as_str().len(), MAX_LEN);
        assert!(numbered.as_str().ends_with("-12"));
    }

    #[test]
    fn the_legacy_world_keeps_its_file_name() {
        assert_eq!(WorldId::legacy().save_file(), "world.save");
        assert_eq!(WorldId::legacy().backup_file(), "world.save.bak");
    }

    #[test]
    fn only_slugs_read_back_as_ids() {
        assert_eq!(WorldId::from_slug("my-world"), Some(id("my world")));
        assert_eq!(WorldId::from_slug("My World"), None);
        assert_eq!(WorldId::from_slug("a--b"), None);
    }
}
