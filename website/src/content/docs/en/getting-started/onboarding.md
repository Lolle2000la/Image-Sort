---
title: First Run Onboarding
description: A quick guide to getting started with Media Sort on your first run.
---

Welcome to Media Sort! When you launch the application for the first time, you can configure your workspace in a few simple steps. Here is how to set up your workflow for maximum sorting speed.

## 1. Select a Root Directory

When you start Media Sort, it will prompt you to select a root directory. This is the main directory containing the media files (images, videos, audio) you want to sort.

- Click on the folder icon or use the native folder picker dialog to select your target folder.
- Media Sort will automatically scan the folder and recursively find all supported formats, presenting them in a beautiful, structured grid.

## 2. Set Up Pinned Folders

Pinned folders are your target destinations. By pinning folders, you assign them quick action shortcuts (such as moving or copying selected media with a single keystroke).

- Navigate to your target directory using the folder tree panel on the left.
- Right-click a folder to pin it, or use the Pinned Folders panel to add target directories.
- Once pinned, these folders are assigned a quick-key macro (by default `Move/Copy to Pinned Folder 1-9`).

## 3. Background Prefetching & Caching

To keep navigation completely seamless, Media Sort implements a smart prefetching and caching system:

- **Thumbnail Prefetching:** The app spawns background worker tasks that eagerly generate thumbnails for upcoming files.
- **LRU Cache:** Images and thumbnails are stored in a high-speed Least Recently Used (LRU) cache (up to 200 items for thumbnails, 20 items for full-resolution previews).
- **Asynchronous watch:** A filesystem watcher automatically updates the view if any files are added, modified, or deleted outside the application.

Sit back and let the background worker populate the cache so you can browse at lightning speed!
