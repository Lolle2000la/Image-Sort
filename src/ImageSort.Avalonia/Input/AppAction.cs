namespace ImageSort.Avalonia.Input;

public enum AppAction
{
    // Image Navigation
    NextImage,
    PreviousImage,

    // Action History
    Undo,
    Redo,

    // Image Operations
    MoveImageToCurrentSelectedFolder, // Action for the folder currently selected in the main folder view/grid (e.g. Up Arrow)
    DeleteImage,
    RenameImage,

    // Folder Tree Navigation/Selection
    SelectNextFolderInTree,         // e.g., S key (moves selection down in the folder tree)
    SelectPreviousFolderInTree,     // e.g., W key (moves selection up in the folder tree)
    ExpandSelectedTreeFolder,       // e.g., D key (expands the selected folder in the tree)
    CollapseSelectedTreeFolderOrGoToParent, // e.g., A key (collapses selected folder or navigates to parent of current working folder)
    SetSelectedTreeFolderAsCurrent, // e.g., Enter key (makes the folder highlighted in the tree the current working folder)

    // Pinned Folder Image Move Operations
    MoveImageToPinnedFolder1,
    MoveImageToPinnedFolder2,
    MoveImageToPinnedFolder3,
    MoveImageToPinnedFolder4,
    MoveImageToPinnedFolder5,
    MoveImageToPinnedFolder6,
    MoveImageToPinnedFolder7,
    MoveImageToPinnedFolder8,
    MoveImageToPinnedFolder9,
    MoveImageToPinnedFolder0,       // For the 10th pinned folder

    // UI Control/Focus
    FocusSearchBox,
    ToggleMetadataPanel,
    // Future: FocusFolders, FocusImages, FocusMetadata if direct focus commands are needed

    // Application Level
    OpenFolderDialog,               // Triggers the 'Open Folder...' functionality (e.g. O key)
    PinCurrentFolder,               // Pins the current working folder (e.g. P key)
    PinSelectedTreeFolder,          // Pins the folder highlighted in the tree (e.g. F key)
    UnpinSelectedTreeFolder,        // Unpins the folder highlighted in the tree (e.g. U key)
    CreateFolderInSelectedTreeFolder, // Creates a new folder inside the one highlighted in the tree (e.g. C key)
    
    // Settings (placeholder for now)
    // OpenSettings, 
}
