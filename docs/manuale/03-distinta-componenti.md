# 03 · Distinta componenti

Prezzi indicativi di **ottobre 2026**, IVA inclusa dove il negozio la mostra. I prezzi dei negozi esteri (Crystalfontz, Adafruit, Pololu) non includono spedizione, IVA all'importazione e dogana: per un ordine dagli USA conta circa il 30–40% in più. Le voci "stima" non sono state verificate su un negozio.

## Riepilogo

| Gruppo | Costo stimato |
|---|---|
| Elettronica principale (Pi, Pico, display, memoria) | 70–95 € |
| Ingressi, protezioni, alimentazione | 20–40 € |
| Sensore, cablaggi, connettori | 15–25 € |
| Contenitore e minuteria | 15–25 € |
| **Totale** | **≈ 150–190 €** compresa la spedizione del display |
| Attrezzi consigliati (non inclusi) | oscilloscopio ≈ 40 €, Debug Probe ≈ 12 € |

Il costo più grande è il display da 1000 nit. Un display non leggibile al sole (HyperPixel 4.0, ≈ 45 €) costerebbe uguale e funzionerebbe peggio: non è consigliato.

## 1. Elettronica principale

| # | Componente | Codice / modello | Q.tà | Prezzo | Dove | Alternativa | Note |
|---|---|---|---|---|---|---|---|
| 1 | Computer | **Raspberry Pi Zero 2 W** | 1 | 17–20 € | [Kubii](https://www.kubii.com/en/raspberry-pi-zero/3455-raspberry-pi-zero-2-w-5056561800004.html), [The Pi Hut](https://thepihut.com/products/raspberry-pi-zero-2) | — | Temperatura −20…+70 °C. **Spesso esaurito**: mettere un avviso di disponibilità. Senza header saldato |
| 2 | Scheda microSD | Industriale pSLC 8–16 GB (es. SanDisk Industrial) | 1 | 10–15 € (stima) | Mouser, Farnell, Amazon | Una buona microSD normale per lo sviluppo | Resiste meglio a vibrazioni e cadute di tensione |
| 3 | Display | **Crystalfontz CFAF800480H0-043SN**, 4,3" 800×480, IPS, 1000 cd/m², RGB 40 pin | 1 | ≈ 20 € (21–24 $) + spedizione 15–25 € | [Crystalfontz](https://www.crystalfontz.com/product/cfaf800480h0043sn-800x480-4-point-3-inch-tft-display) | Waveshare 5DP-CAPLCD-H (HDMI, 800 nit) | −20…+70 °C. Retroilluminazione ≈ 16–19 V 40 mA. **Verificare la piedinatura** con l'adattatore prima di ordinare |
| 4 | Adattatore DPI | **Adafruit DPI TFT Kippah** | 1 | ≈ 17 € (19,95 $) | [Adafruit](https://www.adafruit.com/product/2453) | Adattatore FPC proprio + TPS61169 | Ha già il driver della retroilluminazione, ma senza dimmerazione PWM: per la luminosità automatica si usa il TPS61169 (riga 5) |
| 5 | Driver retroilluminazione | **TPS61169** su breakout Adafruit | 1 | ≈ 4 € (4,50 $) | [Adafruit](https://www.adafruit.com/product/6354) | — | Boost fino a 40 V, ingresso PWM dal Pico |
| 6 | Microcontrollore | **Raspberry Pi Pico** (RP2040) | 1 | ≈ 4 € | The Pi Hut, Kubii (Pico 2: [The Pi Hut](https://thepihut.com/products/raspberry-pi-pico-2)) | Pico 2 (RP2350), stesso firmware | Pico 2: −20…+85 °C |
| 7 | Memoria contachilometri | **FRAM MB85RC256V** (32 KB, I²C) su breakout | 1 | 5–9 € | [Adafruit](https://www.adafruit.com/product/1895), [eBay](https://www.ebay.de/itm/294830793064) | — | 10¹² scritture, 10 anni a 85 °C |
| 8 | Sensore luce | **TEMT6000** su breakout | 1 | ≈ 2 € (stima) | AliExpress, Amazon | — | Per luminosità automatica e tema giorno/notte |

## 2. Alimentazione e protezioni

| # | Componente | Codice / modello | Q.tà | Prezzo | Note |
|---|---|---|---|---|---|
| 9 | Fusibile | Portafusibile stagno + lama mini 2 A | 1 | ≈ 2 € | Il più vicino possibile al punto di prelievo del +12 V |
| 10 | Protezione dall'inversione | P-MOSFET ≥ 40 V, ≥ 5 A, es. **IRF4905** o SQJ401EP + zener 12 V gate + resistenza 10 kΩ | 1 | ≈ 1 € | Caduta trascurabile, a differenza di un diodo |
| 11 | Soppressore di transienti | **TVS SMCJ24A** (1500 W) | 1 | ≈ 0,50 € | Clamp ≈ 39 V: il buck deve reggere almeno 50 V |
| 12 | Filtro | Induttanza 10–22 µH ≥ 2 A + elettrolitico 100 µF 50 V + ceramico 1 µF 50 V | 1 | ≈ 1,50 € | |
| 13 | Convertitore 12→5 V | **Pololu D36V28F5** (ingresso fino a 50 V, 3,2 A) | 1 | ≈ 19 € (21,95 $) | [Pololu](https://www.pololu.com/product/3782), [The Pi Hut](https://thepihut.com/products/pololu-5v-3-2a-step-down-voltage-regulator-d36v28f5). Alternativa economica: modulo LM2596**HV** da 60 V (≈ 4 €, attenzione ai falsi). **Non usare MP1584** (28 V max) |
| 14 | Riserva per il Pico | Schottky 1N5819 + elettrolitico 470 µF 10 V | 1 | ≈ 0,50 € | Tiene acceso il Pico > 50 ms per salvare i km |

## 3. Ingressi

| # | Componente | Codice | Q.tà | Prezzo | Uso |
|---|---|---|---|---|---|
| 15 | Optoaccoppiatore | **EL817** (o PC817) | 6 | ≈ 1,50 € | Folle, abbaglianti, freccia SX, freccia DX, illuminazione, sensore ruota |
| 16 | Optoaccoppiatore a soglia | **H11L1** | 1 | ≈ 1,50 € | Contagiri (uscita a trigger di Schmitt) |
| 17 | Diodi | 1N4148 ×8, 1N4007 ×2, BAT54S ×3 | — | ≈ 1 € | Protezioni e antiparallelo sugli opto |
| 18 | Zener | 15 V 1 W (1N4744A) ×1 | 1 | ≈ 0,20 € | Contagiri |
| 19 | Resistenze | Assortimento 1/4 W 1%; **22 kΩ 1 W ×2** (≥ 350 V) per il contagiri | — | ≈ 3 € | |
| 20 | Condensatori | Ceramici 100 nF ×15, 10 nF ×3 | — | ≈ 1 € | |
| 21 | Pulsanti | Pulsanti stagni IP67, 12–16 mm, momentanei | 2 | ≈ 4 € | MODE e SET |
| 22 | LED di backup | LED 3 mm: verde ×2 (frecce), blu ×1 (abbaglianti), verde ×1 (folle) + resistenze 1,5 kΩ e 1N4148 | — | ≈ 1 € | Funzionano anche a elettronica spenta |

## 4. Sensore di velocità e cablaggi

| # | Componente | Codice | Q.tà | Prezzo | Note |
|---|---|---|---|---|---|
| 23 | Sensore di velocità | **LJ12A3-4-Z/BX** induttivo NPN NA, 6–36 V, portata 4 mm | 1 | ≈ 6 € | [AZ-Delivery](https://az-delivery.de/en/products/lj12a3-4-z-bx-induktiver-naherungssensor). Vede solo l'acciaio: verificare i bulloni con una calamita |
| 24 | Staffa del sensore | Lamierino di alluminio 3 mm, o stampa 3D in ASA/PA-CF | 1 | ≈ 3 € | Distanza 1–2 mm dalle teste dei bulloni |
| 25 | Connettore al quadro | **Superseal 1.5** 6–8 poli, maschio + femmina + terminali + tappi | 1 | ≈ 5 € (stima) | Stagno |
| 26 | Connettore del sensore | **Deutsch DTM** 3 poli | 1 | ≈ 4 € (stima) | |
| 27 | Cablaggio quadro originale usato | **3VD-83509-00** (cord set) o connettore corrispondente | 1 | 5–15 € (usato, stima) | Dà il connettore giusto per l'adattatore senza tagliare la moto |
| 28 | Cavo | Cavo automotive 0,5 mm² FLRY in più colori, cavo schermato 3 poli per il sensore, doppino per l'USB | — | ≈ 8 € | |
| 29 | Ferrite | Anello di ferrite a scatto per il cavo USB | 1 | ≈ 1 € | |

## 5. Costruzione e contenitore

| # | Componente | Q.tà | Prezzo | Note |
|---|---|---|---|---|
| 30 | Millefori a doppia faccia 7×9 cm, header e zoccoli a tulipano | 1 | ≈ 3 € | PCB dedicato solo dopo 500 km di prova |
| 31 | Filamento **ASA** chiaro (≈ 150 g) + nero opaco per la cornice | — | ≈ 5 € | ASA resiste a UV e calore (≈ 93 °C); PETG invecchia peggio al sole |
| 32 | Finestra in policarbonato o PMMA antiriflesso 2 mm | 1 | ≈ 4 € (stima) | |
| 33 | Guarnizione in EPDM, sfiato traspirante (tipo Gore) | 1 | ≈ 3 € (stima) | Evita la condensa |
| 34 | Inserti filettati a caldo M3, viti inox | — | ≈ 3 € | |
| 35 | Vernice per tropicalizzazione (conformal coating) | 1 | ≈ 6 € | Protegge dall'umidità |

## 6. Strumenti per lo sviluppo (facoltativi)

| Componente | Prezzo | Uso |
|---|---|---|
| Raspberry Pi Debug Probe | ≈ 12 € | Debug del firmware (defmt via RTT) |
| Oscilloscopio economico (DSO portatile) | ≈ 40 € | Forma d'onda del contagiri, fortemente consigliato |
| Alimentatore da banco 0–30 V | 30–50 € | Collaudo al banco delle protezioni |
| Sensore temperatura acqua di ricambio | 10–20 € | Misura della curva a bagnomaria senza svuotare l'impianto |

## Ordine consigliato

1. Subito: Pi Zero 2 W (per la disponibilità), display e Kippah (spedizione lunga), Pico, FRAM.
2. Dopo i [rilievi sulla moto](../ricerca/rilievi-sulla-moto.md): sensore di velocità, connettori, cablaggio usato, staffa (dipendono da bulloni e piedinatura).
3. Dopo la conferma dello schema (capitolo 04): componenti passivi, protezioni, millefori.
