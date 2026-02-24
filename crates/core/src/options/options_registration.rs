use crate::options::OptionsDoc;

inventory::collect!(OptionsRegistration);

/// Auto-registration entry for an options type.
pub struct OptionsRegistration {
    /// Return documentation metadata for this options type.
    pub doc_metadata: fn() -> &'static OptionsDoc,
    /// Register the partial type with the DI container.
    pub register: fn(&mut super::OptionsProvider, &mut di::ServiceCollection),
}

impl OptionsRegistration {
    /// Collect documentation metadata from all registered option types, sorted by name.
    #[must_use]
    pub fn get_all() -> Vec<&'static OptionsDoc> {
        let mut docs: Vec<_> = inventory::iter::<OptionsRegistration>
            .into_iter()
            .map(|r| (r.doc_metadata)())
            .collect();
        docs.sort_by_key(|d| d.name);
        docs
    }
}
