use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct MetadataPanelState {
    pub panel_expanded: bool,
    pub current: Option<BTreeMap<String, BTreeMap<String, String>>>,
    pub dragging_divider: bool,
}
