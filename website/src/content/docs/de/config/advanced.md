---
title: Erweiterte Einstellungen
description: Details zu erweiterten Konfigurationsoptionen, Dateilöschprofilen und Ordnerverhalten in Media Sort.
---

Diese Seite dokumentiert die erweiterten Konfigurationseigenschaften und architektonischen Designs von Ordneroperationen und Dateilöschungen.

## Ordnerprofile & Verhalten

Media Sort ermöglicht es Ihnen, das Verhalten beim Öffnen lokaler Ordner anzupassen:

### 1. Zuletzt geöffneten Ordner wiederherstellen
Standardmäßig merkt sich die Anwendung Ihren letzten Arbeitsbereich und stellt ihn beim nächsten Start wieder her.
- **Konfigurationsschlüssel:** `general.reopen_last_opened_folder` (Boolean)
- **Verhalten:** Wenn `true`, speichert die App den Pfad unter `general.last_opened_folder` und öffnet ihn beim Start. Auf `false` setzen, wenn die App immer mit dem Ordnerauswahldialog starten soll.

### 2. Ordnerbaum-Breite
- **Konfigurationsschlüssel:** `general.folder_tree_width` (Integer)
- **Verhalten:** Definiert die Standardbreite des linken Ordnernavigations-Panels in Pixeln.

---

## Dateilöschung & System-Papierkorb

Ein Kernprinzip des Designs von Media Sort ist **Sorgenfreies Sortieren**, das sich auf zerstörungsfreie Löschungen verlässt, um sofortiges **Rückgängigmachen / Wiederholen** zu unterstützen.

### Funktionsweise von Löschungen

Wenn Sie eine Datei löschen (durch Drücken von `Pfeiltaste Unten`), wird sie nicht dauerhaft gelöscht. Stattdessen wird sie in den nativen Papierkorb Ihres Betriebssystems verschoben:

- **Windows:** Wird mithilfe der nativen Shell-API in den Windows-Papierkorb verschoben.
- **macOS:** Wird über die Cocoa-API `NSFileManager` in den Papierkorb verschoben.
- **Linux:** Wird gemäß der Freedesktop.org-Papierkorbspezifikation in den Papierkorb des Benutzers verschoben.

### Rückgängigmachen (Wiederherstellung)

Da die Dateien in den nativen Papierkorb verschoben werden, können sie wieder an ihren ursprünglichen Speicherort zurückgebracht werden, wenn Sie die Taste `Rückgängig` (`Q`) drücken:

1. **Zustandserhaltung:** Media Sort behält ein Referenz-Handle (`TrashRestoreHandle`) mit dem ursprünglichen Pfad und den Dateidetails bei.
2. **Dropping & Flushing:** Solange die Anwendung läuft, verbleiben die Handles im Undo-Stack. Wenn Sie die Anwendung beenden, wird der Undo-Stack geleert und die Handles verworfen. Beim Verwurf wird die Löschung "geflusht" (endgültig an den System-Papierkorb übergeben; die Datei kann nicht mehr über die In-App-Schaltfläche "Rückgängig" wiederhergestellt werden, befindet sich aber weiterhin im System-Papierkorb des Betriebssystems).
