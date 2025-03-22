# Bäumige Beiträge

Im Rahmen der Challenge [Bäumige Direktzahlungen](https://hack.farming.opendata.ch/project/151) an den Open Farming Hackdays 2025 entwickelt.

# Voraussetzungen
* Rust installieren mit Rustup

# Getting Started
* Ordner `data` anlegen im Projektverzeichnis
* Download SWISSTLM3D_2025.gpkg
* (optional überschreiben) Erstellen eines Shapefiles `outline_liebegg.shp` mit zugehörigen files im Ordner `data` (ohne Inhalt, wird verwendet für Dimensionen des Ausschnitts)
* (optional überschreiben) export eines Bildes `background_liebegg.png` im `data/`-Verzeichnis von `https://wms.geo.admin.ch/?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=2651438.9502558867,1243180.509409728,2651848.0412982227,1243333.9185506043&CRS=EPSG:2056&WIDTH=1024&HEIGHT=384&LAYERS=ch.swisstopo.pixelkarte-farbe&FORMAT=image/png`
* `cargo run`

