---
title: Medienkompatibilitäts-Matrix
description: Unterstützte Dateiformate, Metadaten-Extraktion und Such-Filteroptionen in Media Sort.
---

Media Sort ist ein voll ausgestatteter Organizer, der eine Vielzahl von Medienformaten unterstützt. Er extrahiert umfangreiche Metadaten aus Ihren Dateien, sodass Sie Details inspizieren und diese über die Suchleiste filtern können.

## Unterstützte Formate

Media Sort teilt Dateien in drei verschiedene Medientypen ein:

### 1. Bilder
Unterstützte native Bildformate (dekodiert über das reine Rust-Crate `image`):
- **Formate:** JPEG/JPG, PNG, WebP, BMP, TIFF, TGA, Farbfeld (FF), AVIF, DDS, OpenEXR (EXR), HDR, ICO, QOI und PNM (PBM, PGM, PPM, PAM).
- **GIF-Handhabung:** Animierte GIFs werden intern verarbeitet und an die Video-Render-Pipeline weitergeleitet, um Wiedergabe-, Pause- und Vorschau-Schleifen im Raster und im Vorschau-Panel zu unterstützen.

### 2. Video & Container
Unterstützte Videoformate (dynamisch verarbeitet über systemgebundenes `libmpv` und zugrunde liegende FFmpeg-Demuxer):
- **Unterstützte Formate (z. B. unter CachyOS/Linux mit aktuellem mpv v0.41.0):** MP4, MKV, WebM, AVI, MOV, WMV, FLV, M4V, MPEG/MPG, TS, VOB, 3GP, OGM und animierte GIFs.
- **Dynamische Abhängigkeit:** Die Kompatibilität von Codecs und Containerformaten hängt vollständig von der installierten Version von `libmpv` und der zugrunde liegenden FFmpeg-Kompilierung (z. B. `libavcodec`/`libavformat`) auf dem Host-System ab.

### 3. Audio
Unterstützte native Audioformate (dekodiert über Symphonia / Rodio):
- **Formate:** MP3, FLAC, OGG (Vorbis), WAV, AAC, M4A (MPEG-4 Audio / ALAC), WMA, OPUS und AIFF.

---

## Metadaten-Extraktion

Wenn Sie eine Mediendatei auswählen, zeigt das Metadaten-Panel Details an, die direkt aus der Dateistruktur gelesen werden:

| Medientyp | Extrahierte Metadaten-Attribute | Wichtigste Library |
| :--- | :--- | :--- |
| **Bilder** | EXIF-Felder (Kameramodell, Belichtungszeit, ISO, Blendenzahl, Aufnahmedatum, GPS-Koordinaten, Dimensionen) | `kamadak-exif` |
| **Audio** | Audio-Tags (Titel, Künstler, Album, Jahr, Genre, Tracknummer, Bitrate, Dauer) | `id3` / `metaflac` / `mp4ameta` |
| **Video** | Parameter der Videodatei (Codec, Auflösung, Bitrate, Framerate, Dauer) | Eigener Parser |

---

## Suchfilter

Sie können die aktuelle Dateiliste über die Suchleiste (Fokus mit Taste `I`) durchsuchen und filtern:

- **Dateinamensuche:** Geben Sie einen Teil des Dateinamens ein, um das Raster sofort zu filtern (z. B. die Suche nach „Urlaub“).
- **Dateiendungsfilter:** Filtern Sie Dateien nach ihrer Dateiendung, da Endungen Teil des Dateinamens sind (z. B. die Eingabe von `.mp4` oder `.flac`, um nur Dateien dieses Typs anzuzeigen).

*Hinweis: Der Suchfilter vergleicht Eingaben unabhängig von Groß- und Kleinschreibung ausschließlich mit dem Dateinamen. Interne Metadatenfelder (wie EXIF-Tags oder Audio-Künstler) werden bei der Suche nicht abgefragt.*

---

## Leistung & Caching

Um die Navigation und das Durchsuchen absolut flüssig zu halten, implementiert Media Sort ein intelligentes Vorabruf- und Caching-System im Hintergrund:

- **Miniaturansicht-Prefetching:** Startet vorausschauend Hintergrund-Worker, um Miniaturansichten für die nächsten Dateien in Ihrem aktuellen Verzeichnis zu generieren.
- **LRU-Cache:** Speichert Bilder und Miniaturansichten in einem schnellen LRU-Cache (Least Recently Used) mit einer Kapazität von bis zu 200 Miniaturansichten und 20 hochauflösenden Vorschauen für sofortigen Zugriff.
- **Asynchroner Dateisystem-Watcher:** Überwacht Ihr aktives Verzeichnis automatisch auf Änderungen (Hinzufügungen, Löschungen oder Änderungen), die außerhalb der App vorgenommen werden, und aktualisiert das UI-Raster dynamisch.

