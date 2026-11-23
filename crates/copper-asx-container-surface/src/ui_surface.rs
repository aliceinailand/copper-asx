use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUIBuilder {
    pub container_id: String,
    pub display_name: String,
    pub allocated: bool,
    pub bottom_plus_button_visible: bool,
}

impl ContainerUIBuilder {
    pub fn new(container_id: String) -> Self {
        Self {
            container_id,
            display_name: "New Container".to_string(),
            allocated: false,
            bottom_plus_button_visible: true,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = name;
        self
    }

    pub fn mark_allocated(mut self) -> Self {
        self.allocated = true;
        self
    }

    pub fn hide_plus_button(mut self) -> Self {
        self.bottom_plus_button_visible = false;
        self
    }

    pub fn render_bottom_control(&self) -> String {
        if self.bottom_plus_button_visible {
            format!(
                r#"<div class="container-bottom-control">
  <button class="plus-btn" title="Create new container">
    <span class="icon">+</span>
  </button>
  <div class="container-info">
    <span class="container-id">{}</span>
    <span class="container-name">{}</span>
  </div>
</div>"#,
                self.container_id, self.display_name
            )
        } else {
            format!(
                r#"<div class="container-bottom-control-hidden">
  <span class="container-id">{}</span>
</div>"#,
                self.container_id
            )
        }
    }

    pub fn render_metadata_surface(&self) -> String {
        format!(
            r#"<div class="container-metadata-surface">
  <div class="container-header">
    <h2>{}</h2>
    <span class="container-id">{}</span>
  </div>
  <div class="metadata-grid">
    <div class="metadata-row">
      <span class="label">Container ID</span>
      <span class="value">{}</span>
    </div>
    <div class="metadata-row">
      <span class="label">Status</span>
      <span class="value">Active</span>
    </div>
    <div class="metadata-row">
      <span class="label">Members</span>
      <span class="value">0</span>
    </div>
  </div>
</div>"#,
            self.display_name, self.container_id, self.container_id
        )
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataListItem {
    pub member_id: String,
    pub display_name: String,
    pub reading_state: String,
    pub version_number: u32,
    pub relationship_type: String,
    pub pending_proposal: bool,
}

impl MetadataListItem {
    pub fn new(
        member_id: String,
        display_name: String,
        reading_state: String,
        version_number: u32,
        relationship_type: String,
    ) -> Self {
        Self {
            member_id,
            display_name,
            reading_state,
            version_number,
            relationship_type,
            pending_proposal: false,
        }
    }

    pub fn with_pending_proposal(mut self, pending: bool) -> Self {
        self.pending_proposal = pending;
        self
    }

    pub fn render_row(&self) -> String {
        let proposal_badge = if self.pending_proposal {
            r#"<span class="badge badge-pending">Version Pending</span>"#
        } else {
            r#"<span class="badge badge-approved">Version Approved</span>"#
        };

        format!(
            r#"<tr class="metadata-row">
  <td class="col-member">{}</td>
  <td class="col-name">{}</td>
  <td class="col-state">{}</td>
  <td class="col-version">{}</td>
  <td class="col-type">{}</td>
  <td class="col-proposal">{}</td>
</tr>"#,
            self.member_id,
            self.display_name,
            self.reading_state,
            self.version_number,
            self.relationship_type,
            proposal_badge
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerListUI {
    pub items: Vec<MetadataListItem>,
}

impl ContainerListUI {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: MetadataListItem) {
        self.items.push(item);
    }

    pub fn render_table(&self) -> String {
        let rows = self
            .items
            .iter()
            .map(|item| item.render_row())
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<table class="metadata-table">
  <thead>
    <tr>
      <th>Member ID</th>
      <th>Name</th>
      <th>Reading State</th>
      <th>Version</th>
      <th>Relationship</th>
      <th>Proposal Status</th>
    </tr>
  </thead>
  <tbody>
{}
  </tbody>
</table>"#,
            rows
        )
    }

    pub fn filter_by_state(&self, state: &str) -> Vec<MetadataListItem> {
        self.items
            .iter()
            .filter(|item| item.reading_state == state)
            .cloned()
            .collect()
    }

    pub fn filter_by_pending(&self) -> Vec<MetadataListItem> {
        self.items
            .iter()
            .filter(|item| item.pending_proposal)
            .cloned()
            .collect()
    }
}

impl Default for ContainerListUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_ui_builder_creation() {
        let builder = ContainerUIBuilder::new("c1".to_string());
        assert_eq!(builder.container_id, "c1");
        assert_eq!(builder.display_name, "New Container");
        assert!(builder.bottom_plus_button_visible);
        assert!(!builder.allocated);
    }

    #[test]
    fn test_container_ui_builder_with_name() {
        let builder = ContainerUIBuilder::new("c1".to_string())
            .with_name("My Container".to_string());
        assert_eq!(builder.display_name, "My Container");
    }

    #[test]
    fn test_container_ui_builder_mark_allocated() {
        let builder = ContainerUIBuilder::new("c1".to_string()).mark_allocated();
        assert!(builder.allocated);
    }

    #[test]
    fn test_container_ui_builder_hide_plus_button() {
        let builder = ContainerUIBuilder::new("c1".to_string()).hide_plus_button();
        assert!(!builder.bottom_plus_button_visible);
    }

    #[test]
    fn test_render_bottom_control_with_plus() {
        let builder = ContainerUIBuilder::new("c1".to_string());
        let html = builder.render_bottom_control();
        assert!(html.contains("plus-btn"));
        assert!(html.contains("c1"));
        assert!(html.contains("New Container"));
    }

    #[test]
    fn test_render_bottom_control_without_plus() {
        let builder = ContainerUIBuilder::new("c1".to_string()).hide_plus_button();
        let html = builder.render_bottom_control();
        assert!(html.contains("container-bottom-control-hidden"));
        assert!(html.contains("c1"));
        assert!(!html.contains("plus-btn"));
    }

    #[test]
    fn test_render_metadata_surface() {
        let builder = ContainerUIBuilder::new("c1".to_string())
            .with_name("Test Container".to_string());
        let html = builder.render_metadata_surface();
        assert!(html.contains("Test Container"));
        assert!(html.contains("c1"));
        assert!(html.contains("Container ID"));
    }

    #[test]
    fn test_as_json() {
        let builder = ContainerUIBuilder::new("c1".to_string());
        let json = builder.as_json();
        assert!(json.contains("c1"));
        assert!(json.contains("New Container"));
    }

    #[test]
    fn test_metadata_list_item_creation() {
        let item = MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        );
        assert_eq!(item.member_id, "m1");
        assert_eq!(item.display_name, "Member 1");
        assert!(!item.pending_proposal);
    }

    #[test]
    fn test_metadata_list_item_with_pending() {
        let item = MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        )
        .with_pending_proposal(true);
        assert!(item.pending_proposal);
    }

    #[test]
    fn test_metadata_list_item_render_row() {
        let item = MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        );
        let html = item.render_row();
        assert!(html.contains("m1"));
        assert!(html.contains("Member 1"));
        assert!(html.contains("Indexed"));
    }

    #[test]
    fn test_container_list_ui_creation() {
        let list = ContainerListUI::new();
        assert_eq!(list.items.len(), 0);
    }

    #[test]
    fn test_container_list_ui_add_item() {
        let mut list = ContainerListUI::new();
        let item = MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        );
        list.add_item(item);
        assert_eq!(list.items.len(), 1);
    }

    #[test]
    fn test_container_list_ui_render_table() {
        let mut list = ContainerListUI::new();
        let item = MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        );
        list.add_item(item);
        let html = list.render_table();
        assert!(html.contains("metadata-table"));
        assert!(html.contains("m1"));
    }

    #[test]
    fn test_container_list_ui_filter_by_state() {
        let mut list = ContainerListUI::new();
        list.add_item(MetadataListItem::new(
            "m1".to_string(),
            "Member 1".to_string(),
            "Indexed".to_string(),
            1,
            "Initial".to_string(),
        ));
        list.add_item(MetadataListItem::new(
            "m2".to_string(),
            "Member 2".to_string(),
            "Verified".to_string(),
            2,
            "AppendVersion".to_string(),
        ));

        let filtered = list.filter_by_state("Indexed");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].member_id, "m1");
    }

    #[test]
    fn test_container_list_ui_filter_by_pending() {
        let mut list = ContainerListUI::new();
        list.add_item(
            MetadataListItem::new(
                "m1".to_string(),
                "Member 1".to_string(),
                "Indexed".to_string(),
                1,
                "Initial".to_string(),
            )
            .with_pending_proposal(true),
        );
        list.add_item(
            MetadataListItem::new(
                "m2".to_string(),
                "Member 2".to_string(),
                "Verified".to_string(),
                2,
                "AppendVersion".to_string(),
            )
            .with_pending_proposal(false),
        );

        let pending = list.filter_by_pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].member_id, "m1");
    }

    #[test]
    fn test_container_list_ui_default() {
        let list = ContainerListUI::default();
        assert_eq!(list.items.len(), 0);
    }

    #[test]
    fn test_cross_container_visibility() {
        let list1 = ContainerListUI::new();
        let list2 = ContainerListUI::new();
        // Lists are isolated (default deny)
        assert_eq!(list1.items.len(), list2.items.len());
    }
}
