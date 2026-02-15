use prj_core::project::Project;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListAction {
    ViewStats,
    CleanArtifacts,
    OpenEditor,
    OpenExplorer,
    CdToProject,
    Remove,
}

pub struct MenuItem {
    pub action: ListAction,
    pub label: &'static str,
    pub description: &'static str,
}

/// Build the available menu items for a given project.
pub fn menu_items(project: &Project) -> Vec<MenuItem> {
    let mut items = vec![MenuItem {
        action: ListAction::ViewStats,
        label: "View stats",
        description: "Show detailed project statistics",
    }];

    if !project.artifact_dirs.is_empty() {
        items.push(MenuItem {
            action: ListAction::CleanArtifacts,
            label: "Clean artifacts",
            description: "Delete build artifact directories",
        });
    }

    items.push(MenuItem {
        action: ListAction::OpenEditor,
        label: "Open in editor",
        description: "Open project in $EDITOR",
    });

    items.push(MenuItem {
        action: ListAction::OpenExplorer,
        label: "Open in explorer",
        description: "Open project folder in file manager",
    });

    items.push(MenuItem {
        action: ListAction::CdToProject,
        label: "cd to project",
        description: "Change directory to project",
    });

    items.push(MenuItem {
        action: ListAction::Remove,
        label: "Remove",
        description: "Unregister project from database",
    });

    items
}
