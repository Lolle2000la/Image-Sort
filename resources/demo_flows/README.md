# Demo Flows Guide

This directory contains JSON automation files defining interactive workflows. These demo flows are used by the simulator to run automated tests or render presentation videos of the application in action.

## Flow Structure

A flow file is a JSON object with two main fields:
- `flow_name` (string): The user-facing name of the demo.
- `steps` (array): A chronological list of automation steps.

### Example Flow

```json
{
  "flow_name": "Sorting Workflow",
  "steps": [
    {
      "delay_ms": 1500,
      "target": { "type": "Widget", "value": { "id": "Unsorted" } },
      "keycap_label": "Enter",
      "message": { "action": "open_folder", "relative_path": "Unsorted" }
    },
    {
      "delay_ms": 2500,
      "target": { "type": "Widget", "value": { "id": "media_card_0" } },
      "keycap_label": "Click",
      "message": { "action": "select_entry", "index": 0 }
    }
  ]
}
```

---

## Anatomy of a Step

Each step in the `steps` array contains the following configuration properties:

| Property | Type | Description |
| :--- | :--- | :--- |
| `delay_ms` | integer | Delay (in milliseconds) before executing this step. |
| `target` | object | The element to position the virtual cursor on (see [Targets](#targets)). |
| `keycap_label` | string (optional) | Text displayed in the keycaster overlay (e.g. `"Enter"`, `"Click"`, `"M"`). |
| `message` | object | The action payload to invoke (see [Actions](#actions)). |

### Targets

The virtual cursor moves automatically to the target's coordinates. Two types of targets are supported:

1. **Widget Targets** (Recommended): Resolves coordinates dynamically at runtime using the widget's unique ID.
   ```json
   "target": { "type": "Widget", "value": { "id": "move_btn" } }
   ```
2. **Coordinate Targets**: Hardcoded absolute pixel coordinates. Should be avoided as they depend on the layout and resolution.
   ```json
   "target": { "type": "Coordinate", "value": { "x": 500.0, "y": 450.0 } }
   ```

### Actions

The `message` object defines the action payload, which must contain an `"action"` field. The following actions are supported:

| Action | Additional Fields | Description |
| :--- | :--- | :--- |
| `"go_right"` | None | Select the next media file (e.g. right arrow). |
| `"go_left"` | None | Select the previous media file (e.g. left arrow). |
| `"move_active"` | None | Move the active media file to the selected folder. |
| `"copy_active"` | None | Copy the active media file to the selected folder. |
| `"trigger_rename"` | None | Open the rename modal for the active media file. |
| `"open_settings"` | None | Open the settings dialog. |
| `"close_settings"` | None | Close the settings dialog. |
| `"toggle_dark_mode"`| None | Set theme to Dark mode. |
| `"focus_search"` | None | Focus the search bar text field. |
| `"quit"` | None | Exits the application immediately. |
| `"open_folder"` | `relative_path` (string) | Open a folder at a relative path from the demo root. |
| `"select_entry"` | `index` (integer) | Select a media file entry at a specific index. |
| `"search_query"` | `query` (string) | Type a query into the search bar. |
| `"folder_selected"`| `relative_path` (string) | Select a folder in the folder tree navigation panel. |

---

## Known Widget IDs

When targeting widgets, you can use the following pre-defined widget IDs:

- **Folders in Folder Tree**: The folder tree nodes are given IDs matching their directory names. For example, to target a folder named `Images` or `Unsorted`, use:
  - `"Images"`
  - `"Unsorted"`
- **Media Cards (Media Grid)**: Cards in the grid are named sequentially based on their index:
  - `"media_card_0"`, `"media_card_1"`, etc.
- **Buttons**:
  - `"move_btn"`: The primary "Move to Folder" button.
  - `"copy_btn"`: The "Copy to Folder" button.
  - `"settings_btn"`: The button in the control panel to open settings.
  - `"close_settings_btn"`: The button to close/dismiss the settings dialog.
- **Scrollables**:
  - `FOLDER_TREE_SCROLLABLE_ID`: The folder tree scrollable container.
  - `MEDIA_GRID_SCROLLABLE_ID`: The media grid scrollable container.
- **Search**:
  - `SEARCH_INPUT_ID`: The search input text field.

---

## Best Practices & Tips

1. **Use Widget IDs**: Always use `"type": "Widget"` targets with a known ID rather than absolute pixel coordinates. This ensures that the demo animations remain correct if the window size changes.
2. **Ensure Generous Delays**: Animating the cursor toward a target takes time. Ensure you have at least `1500` to `2500` ms of delay on steps that perform click actions to let the cursor align smoothly before triggering the message.
3. **Finish with Quit**: When designing a flow meant for headless video rendering, always end the steps with a `"quit"` action. This allows the headless exporter to terminate gracefully and close the ffmpeg encoder process.
4. **Mock Media Generation**: During a demo run, the simulator generates temporary placeholder files (e.g. under `/tmp/media_sort_demo_*`) including directories named `Unsorted` and `Images` to ensure the flow steps successfully find their paths. Keep this structure in mind when designing paths for `open_folder` and `folder_selected`.
