# 0009 · Display da 7" 800×480 da 1000 nit in DPI

**Stato:** accettata · ottobre 2026. Sostituisce la scelta del Crystalfontz CFAF800480H0-043SN da 4,3" fatta in fase di pianificazione.

## Contesto

Il 4,3" è stato giudicato troppo piccolo per la moto. Le alternative valutate:

| Opzione | Luminosità | Collegamento | Note |
|---|---|---|---|
| Barre allungate 7,8–8,8" (Waveshare 7,9" 400×1280, Topway 8,8" 480×1920) | 550–600 nit | HDMI | Nessuna barra acquistabile da privati supera i 600 nit; nascono verticali (rotazione via software); fino a 2,4 volte i pixel di 800×480; la 8,8" è larga 252 mm |
| Barre 7,84" da 850 nit | 850 nit | MIPI DSI | Solo su preventivo, richiedono una scheda con DSI e un driver del pannello scritto da noi |
| 7" 1024×600 (Newhaven NHD-7.0-HDMI-HR-RSXP) | 1100 nit | HDMI | ≈ 100–160 €, 1,6 volte i pixel, mini-HDMI da bloccare |
| **7" 800×480 (Newhaven NHD-7.0-800480AF-ASXP)** | **1000 nit** | **DPI** | ≈ 78 €, stessi pixel del progetto iniziale |

## Decisione

**Newhaven NHD-7.0-800480AF-ASXP**, collegato al Pi Zero 2 W in DPI tramite l'Adafruit DPI TFT Kippah. Dati dal [datasheet](https://newhavendisplay.com/content/specs/NHD-7.0-800480AF-ASXP.pdf):

| Dato | Valore |
|---|---|
| Pannello | 7" IPS, 800×480, normalmente nero, trasmissivo, antiriflesso |
| Luminosità | 1000 cd/m² tipica, 800 minima, a 180 mA |
| Area attiva / ingombro | 154,2 × 85,6 mm / 165 × 104 mm, 133 ppi |
| Temperatura di funzionamento | −20 … +70 °C |
| Interfaccia | RGB parallelo 24 bit, FFC 40 poli passo 0,5 mm (Molex 54104-4031), driver Sitronix ST7277, modo DE/SYNC/SYNC-DE |
| Piedinatura | 1 LED-K, 2 LED-A, 3 GND, 4 VDD 3,3 V, 5–12 R0–R7, 13–20 G0–G7, 21–28 B0–B7, 29 GND, 30 DCLK, 31 DISP, 32 HSYNC, 33 VSYNC, 34 DE, 35 BIST_EN, 36 GND, 37–40 NC |
| Alimentazione logica | 3,3 V, 55 mA tipici |
| Retroilluminazione | 180 mA (135–225) a 9,3 V (8,3–10,2), **nessun driver a bordo** |
| Timing | DCLK 23–27 MHz (25 tipico); orizzontale 800 + 4/8/8 (impulso/back/front porch tipici); verticale 480 + 4/16/16 |

La piedinatura è lo standard a 40 poli per pannelli RGB, la stessa usata dal Kippah. Il Pi pilota il pannello a 18 bit (RGB666): i due bit bassi di ogni colore sono collegati dal Kippah.

**Retroilluminazione:** TPS61169 comandato in PWM dal Pico (ADR 0006), con resistenza di regolazione ≈ 1,1–1,2 Ω per circa 170–180 mA. `DISP` va tenuto alto e `BIST_EN` basso.

## Conseguenze

- **Software e design invariati:** la risoluzione resta 800×480, quindi il carico del renderer software non cambia e l'artifact di design resta valido. Cambiano solo le dimensioni fisiche: tutto è 1,6 volte più grande che sul 4,3".
- **Consumo** più alto: ≈ 2 W per il display invece di ≈ 0,8 W. Il convertitore da 3,2 A basta.
- **Contenitore:** più grande, almeno 175 × 115 mm di fronte con la cornice. Va verificato lo spazio dietro il cupolino durante i rilievi.
- **Costo** del display: da ≈ 40 € (compresa spedizione) a ≈ 78 €. Totale stimato ≈ 190–240 €.
- **Da verificare:**
  - se il driver di retroilluminazione del Kippah regge 180 mA
  - se la resistenza della breakout TPS61169 si può sostituire
- **Configurazione Linux:** `vc4-kms-dpi-generic` con clock 25 MHz e i timing sopra (milestone M4).
