![Screenshot from the user interface of Image Sort](https://github.com/Lolle2000la/Image-Sort/raw/master/Image-Sort-Screenshot.gif)

## Installation

You can install Image Sort ~~either using the [Microsoft Store](https://www.microsoft.com/store/apps/9PGDK9WN8HG6) (Windows 10 only)~~ or the installer. 
If you are not sure whether you should run the x86 or the x64 installer, please refer to this [link](https://support.microsoft.com/de-de/help/15056/windows-32-64-bit-faq).

| x86                                                                                              | x64                                                                                              |
|--------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| [download](https://github.com/Lolle2000la/Image-Sort/releases/latest/download/ImageSort.x86.msi) | [download](https://github.com/Lolle2000la/Image-Sort/releases/latest/download/ImageSort.x64.msi) |

**I can't update the Microsoft Store build anymore. I don't know why and the support isn't helpful, but I will cease development on it. Please install the regular .msi-Version from the releases to keep up with new versions.**

## A quick description

Image Sort is an open source app that lets you sort your folders in an fast fashion. It let's you open an folder and move the images in it around as you like.

## Why Image Sort?

- It's fast
- It's completely controllable via keyboard, with a one-click shortcuts being there for nearly anything.
- It gives you all the control you need without sacrificing speed

## How to use

> The central philosophy behind Image Sort's design is speed. For that reason the ideal way to use this app is not to
leave the keyboard. However, you can of course use the app in any way you wish and ideally it should still help you 
sort your images fast.

When you open the app, you are presented with your pictures folder already being open. When you want to open another
folder, open it by pressing the "Open folder" button or the 'O' key.

### Central workflow

On the left you have your folders. It's a tree, so you can access all your sub-folders. You can also pin folders,
either the selected one ("Pin selected") or a manually picked one  ("Pin"), making them easier to access, but also
making it possible to f.e. sort images from one folder into others on other disks.

Then, select an image and choose whether you want to move the image to the selected folder or want to delete it (moving it
to recycle bin).

But maybe you accidentally delete or move an image and think 'Oh, why did I do that? Can I undo this?'. Yes, you can.
Simply press undo. You would not believe how much of a pain this is to achieve for the recycle bin. But it was worth it,
I hope.

### The keyboard is your friend

Why not up you sorting game? It's really easy and doesn't take a lot. For the most things, what action is triggered by
what key is noted on the control itself (e.g. 'Open Folder *(o)*'). However, how you should place your hands on your
keyboard is not obvious.

You navigate through the folders on the left by using the WASD keys. Gamers already now that scheme, but to anyone else,
they basically work like the arrow keys, with W being up, A being left, S being down and D being right. So you use WASD
like you do the arrow keys but with your left hand. Ideally you want to put the middle finger on the W/S keys, the
ring finger on the A and the index finger on the D key.

Meanwhile, the actual arrow keys are in use by your right hand. The left and right keys navigate through the images.
The up arrow key moves the current image to the selected folder and the down arrow key deletes the image (moves to recycle bin). *The buttons on the right doing the same are placed in the way the arrow keys are bound.*

Often used actions are close to these two key-groups, while less often used actions may be more distanced.

For example, the keybindings for Undo/Redo are Q and E respectively, because the are easily accessible from your ring/index fingers. Q is undo, E is redo. What this allows you is to do these actions without a lot of friction, hopefully
allowing you to sort your images really quickly without annoying pauses or slowdowns because you have to change from
the keyboard to the mouse or the other way around.

On the other hand, actions like "Open folder" are usually not that often used, so they're placed on the more distanced and often more expressive shortcut (like 'O', for the aforementioned "Open folder" action).

In general, you should learn this basic position, but aside from that only learn the shortcuts you really need. It can
be nice to select a new folder with 'O' but if you do that once a day and do not see value in learning that particular
shortcut for just that few uses, just ignore it. It doesn't hurt to move your hands off your keyboard every once in a
while. Do not feel pressured into doing everything with the keyboard just because someone told you how great that is.
It's your choice to see what works out best for you!

## Install

Go to the [releases](https://github.com/Lolle2000la/Image-Sort/releases) page, and download and run the installer of the newest version. Then follow the instructions in the installer.

## Updating

Image Sort includes an updater that checks for updates every time the apps runs. When a new version is available, it will ask you if you want to do the update. If yes, the updater will close the app and run the update. Once finished, you can start the app again.

You can also turn the update checker off, if you prefer to stay on the current version.

## Privacy Policy

Read the [Privacy Policy](privacy_policy.md) page for details on what data we collect.

## Contributing

If you want to contribute, you should fork this repository, make your changes and then make a pull request.

## Requirements
* Windows 7 Service Pack 1 or higher

## Build-Prerequisites
* [.NET Core SDK 3.1](https://dotnet.microsoft.com/download/dotnet-core/3.1)
* (optional) Visual Studio 2019
* (optional) For building the installer, you need [WiX Toolset](https://wixtoolset.org/) 3.11 or higher
