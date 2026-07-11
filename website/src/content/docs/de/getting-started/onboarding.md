---
title: Erste Schritte & Einrichtung
description: Eine Kurzanleitung für die ersten Schritte mit Media Sort nach dem Start.
---

Willkommen bei Media Sort! Wenn Sie die Anwendung zum ersten Mal starten, können Sie Ihren Arbeitsbereich in wenigen einfachen Schritten einrichten. Hier erfahren Sie, wie Sie Ihren Workflow für maximale Sortiergeschwindigkeit optimieren.

## 1. Quellordner auswählen

Beim Start von Media Sort werden Sie aufgefordert, ein Stammverzeichnis (Root) auszuwählen. Dies ist der Hauptordner, der die zu sortierenden Mediendateien (Bilder, Videos, Audio) enthält.

- Klicken Sie auf das Ordnersymbol oder verwenden Sie den nativen Datei-Auswahldialog, um Ihren Zielordner auszuwählen.
- Media Sort scannt den Ordner automatisch und findet rekursiv alle unterstützten Formate, die in einem strukturierten Raster dargestellt werden.

## 2. Angepinnte Ordner einrichten

Angepinnte Ordner sind Ihre Sortierziele. Indem Sie Ordner anpinnen, weisen Sie ihnen Tastatur-Shortcuts zu (z. B. um ausgewählte Medien mit einem einzigen Tastendruck zu verschieben oder zu kopieren).

- Navigieren Sie mithilfe des Ordnerbaums auf der linken Seite zum gewünschten Zielverzeichnis.
- Klicken Sie mit der rechten Maustaste auf einen Ordner, um ihn anzupinnen, oder nutzen Sie das Panel "Angepinnte Ordner", um Verzeichnisse hinzuzufügen.
- Sobald sie angepinnt sind, werden diesen Ordnern Schnellwahltasten zugewiesen (standardmäßig `Verschieben/Kopieren in angepinnten Ordner 1-9`).

## 3. Hintergrund-Prefetching & Cache

Um die Navigation absolut flüssig zu halten, implementiert Media Sort ein intelligentes Vorabruf- und Caching-System:

- **Miniaturansicht-Prefetching:** Die App startet Hintergrund-Worker, die vorausschauend Miniaturansichten für die nächsten Dateien generieren.
- **LRU-Cache:** Bilder und Miniaturansichten werden in einem schnellen LRU-Cache (Least Recently Used) gespeichert (bis zu 200 Elemente für Miniaturansichten, 20 Elemente für hochauflösende Vorschauen).
- **Asynchroner Watcher:** Ein Dateisystem-Watcher aktualisiert die Ansicht automatisch, wenn Dateien außerhalb der App hinzugefügt, geändert oder gelöscht werden.

Lehnen Sie sich zurück und lassen Sie die Hintergrund-Worker den Cache füllen, damit Sie blitzschnell navigieren können!
