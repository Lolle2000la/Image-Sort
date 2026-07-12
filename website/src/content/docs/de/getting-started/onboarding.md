---
title: Erste Schritte & Einrichtung
description: Eine Kurzanleitung für die ersten Schritte mit Media Sort nach dem Start.
---

Wenn Sie Media Sort zum ersten Mal starten, öffnet sich die Anwendung direkt in der Medienraster-Ansicht. Das erwartet Sie und so richten Sie Ihren Arbeitsbereich ein.

## Startverhalten

Beim ersten Start öffnet Media Sort automatisch das systemseitige **Bilder**-Verzeichnis (z. B. `~/Bilder` unter Linux, `~/Pictures` unter macOS). Wenn Sie dort keine Dateien haben, ist das Raster leer – verwenden Sie die Ordnerauswahl, um ein anderes Verzeichnis zu wählen.

Falls Sie zuvor die ältere WPF-Version (v2.x) verwendet haben, **migriert** Media Sort Ihre Einstellungen (angepinnte Ordner, Dark Mode, Hotkeys, Fensterposition) **still und automatisch** aus der alten `config.json` in das neue `config.toml`-Format. Ein manuelles Eingreifen ist nicht erforderlich.

## 1. Quellordner auswählen

Um ein Verzeichnis mit den zu sortierenden Mediendateien (Bilder, Videos, Audio) auszuwählen:

- Klicken Sie in der linken Seitenleiste unter **Ordner** auf die Schaltfläche **Öffnen**, um den nativen Ordnerauswahldialog zu öffnen.
- Alternativ drücken Sie `O`, um den nativen Ordnerauswahldialog zu öffnen.
- Media Sort scannt den ausgewählten Ordner automatisch und findet rekursiv alle unterstützten Formate, die in einem strukturierten Raster dargestellt werden.

## 2. Angepinnte Ordner einrichten

Angepinnte Ordner sind Ihre Zielverzeichnisse zum Sortieren. Durch das Anpinnen von Ordnern weisen Sie ihnen Kurzbefehle zu, mit denen Sie ausgewählte Medien mit einem einzigen Tastendruck verschieben können.

- Navigieren Sie mithilfe des Ordnerbaums auf der linken Seite zu Ihrem Zielverzeichnis.
- Wählen Sie einen Ordner aus und drücken Sie `F`, um ihn anzupinnen, oder nutzen Sie das Panel „Angepinnte Ordner“, um Zielverzeichnisse hinzuzufügen.
- Sobald sie angepinnt sind, werden diesen Ordnern Schnellwahltasten zugewiesen: Drücken Sie `Alt + 1` bis `Alt + 9`, um die ausgewählte Mediendatei direkt in den entsprechenden angepinnten Ordner zu verschieben.

## 3. Ihre Konfiguration anpassen

Öffnen Sie den **Einstellungen**-Dialog über die linke Seitenleiste, um Folgendes zu konfigurieren:
- **Design** – wählen Sie zwischen Dark, Light, Dracula, Nord, Catppuccin und weiteren
- **Tastenkürzel** – belegen Sie alle Shortcuts nach Ihren Wünschen neu
- **Sprache** – wechseln Sie zwischen Deutsch, Englisch und Japanisch
- **GIF-Animation** – schalten Sie die animierte GIF-Wiedergabe im Raster und in der Vorschau ein oder aus
