![Screenshot from the user interface of Image Sort](https://github.com/Lolle2000la/Image-Sort/blob/master/screenshot_1.12.0.png?raw=true)

## A quick description
Image sort is an open source app that lets you sort your folders in an fast fashion. It let's you select an folder and move the images around as you like. You get one image after another and the choice to move it to one of the subfolders (or the hostfolder, that means the folder containing the currently selected folder) or skip it. You can also revert your last action (skip/move).

## Why Image sort?
* It's tiny (around 3MB)
* It's fast
* It's completely controllable via keyboard, with a one-click shortcut for nearly anything. Read the [help](help.md) for more details.

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
   
   _Read the [help](help.md) page for more details_

## Install
Go to the [releases](https://github.com/Lolle2000la/Image-Sort/releases) page, and download and run the installer of the newest version. Then follow the instructions in the installer.

## Updating
Image sort includes an updater that checks for updates every time the apps runs. When a new version is available, it will ask you if you want to do the update. If yes, the updater will close the app and run the update. Once finished, you can start the app again.

## Contributing
If you want to contribute, you should fork this repository, make your changes and then make a pull request.

## Requirements
* .NET Framework 4.7.1 or higher
* Windows 7 service pack 1 of higher

## Build-Requirements
* .NET tooling
* Visual Studio 2017
* (optional) For building the installer, you need the [Microsoft Visual Studio 2017 Installer Projects](https://marketplace.visualstudio.com/items?itemName=VisualStudioProductTeam.MicrosoftVisualStudio2017InstallerProjects)
