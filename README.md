# Image-Sort
Sorts your image at high speed

![Screenshot of the UI of the image](https://github.com/Lolle2000la/Image-Sort/blob/master/ImageSort_screenshot_1.9.0.png)

## How to use
1. Open the app, select a folder to sort and finished. The app will now show you the images one by one for you. You must then choose the folder you want the image to sort into (subfolders of the selected folder) and press the move button. You can also skip the image if you don't want to move it.
2. If you want to you can use the keyboard allone for all the tasks:
   #### The important stuff
   * F2 for selecting the folder
   * F3 to create a new folder
   * F5 to open the currently selected image in the explorer
   * Enter to enter the currently selected folder
   * Escape to leave the current folder
   * up- and down-arrow-keys to select a folder to move to
   * right-arrow-key to move the image into the selected folder (or one folder upwards if you select "..")
   * left-arrow-key to skip it
   * ctlr + left-arrow-key to revert the last action done (move/skip)
   * ctrl+s toggles the search bar, allowing for quick searches for the folder you need. Close hide the search bar again to resume using the arrow keys to sort images. You can also achieve this by pressing the "Search" button.
   #### The not-so-important stuff
   * F4 to change the resolution in which the images should get loaded (default: 1000 pixel, smaller = less RAM usage and faster loading speed). Pressing F4 will move your focus to the text box, so that you can type in your preferred resolution. Pressing Enter or Escape, as well as moving the focus away restores normal keyboard input behavior.

## Requirements
* .NET Framework 4.7.1
* Windows 7 Sevice Pack 1 or higher

## Build-Prerequisites
* Visual Studio 2017
* (optional) For building the installer, you need the [Microsoft Visual Studio 2017 Installer Projects](https://marketplace.visualstudio.com/items?itemName=VisualStudioProductTeam.MicrosoftVisualStudio2017InstallerProjects)
