# Image-Sort 
Sorts your image at high speed

[![Build status](https://ci.appveyor.com/api/projects/status/i72q6f479ah9d4vw/branch/master?svg=true)](https://ci.appveyor.com/project/Lolle2000la/image-sort/branch/master)

<a href='//www.microsoft.com/store/apps/9PGDK9WN8HG6?ocid=badge'><img src='https://assets.windowsphone.com/85864462-9c82-451e-9355-a3d5f874397a/English_get-it-from-MS_InvariantCulture_Default.png' alt='English badge' width="200"/></a>

![Screenshot from the user interface of Image Sort](./Image-Sort-Screenshot.png)

# Currently, all work is being focused on version 2.0. Below refers to version 1.X.

If you want to contribute to 2.0, you need the [.NET Core 3.1 Windows SDK](https://dotnet.microsoft.com/download/dotnet-core/3.1).
You can run the tests in ImageSort.UnitTests with 'dotnet test' and run the Windows app in ImageSort.WPF with 'dotnet run'.
Of course you can also just launch the solution in Visual Studio. That requires Visual Studio 2019 updated to the newest version. 

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
   
## Privacy Policy
Read the [Privacy Policy](https://imagesort.org/privacy_policy.html) page for details on what data we collect.

## Requirements
* .NET Framework 4.7.2
* Windows 7 Sevice Pack 1 or higher

## Build-Prerequisites
* Visual Studio 2017
* (optional) For building the installer, you need the [Microsoft Visual Studio 2017 Installer Projects](https://marketplace.visualstudio.com/items?itemName=VisualStudioProductTeam.MicrosoftVisualStudio2017InstallerProjects)
