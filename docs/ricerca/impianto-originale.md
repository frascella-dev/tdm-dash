# Ricerca: impianto elettrico originale della TDM 850 (3VD)

Raccolta dei dati sull'impianto elettrico e sul quadro strumenti della prima serie della Yamaha TDM 850 (1991–1995). È la base su cui si progetta il quadro digitale. Il capitolo [01 del manuale](../manuale/01-impianto-originale.md) spiega gli stessi dati in forma più leggibile.

Ogni dato è marcato:

| Marcatura | Significato |
|---|---|
| **[VERIFICATO]** | Confermato da almeno due fonti indipendenti, o da una fonte ufficiale (manuale Yamaha, Haynes), o misurato sulla moto |
| **[DA VERIFICARE]** | Una sola fonte, o una fonte che riguarda un modello simile (4TX, TRX 850, XTZ 750) |
| **[IPOTESI]** | Deduzione nostra o convenzione generale Yamaha; nessuna fonte specifica |

Quando un dato viene misurato sulla moto si aggiorna la marcatura e si registra la misura in [`rilievi-sulla-moto.md`](rilievi-sulla-moto.md).

## 1. Identificazione del modello

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Codice prima serie | **3VD**, 1991–1995, albero a 360° | [VERIFICATO] | [Wikipedia](https://en.wikipedia.org/wiki/Yamaha_TDM850), [CMSNL](https://www.cmsnl.com/yamaha-tdm850-1992-3vd4-europe-223vd300e2_model37920/partslist/) |
| Variante 1992 Europa | **3VD4** (3VD1/3VD2 1991, 3VD5 1993, 3VD7 1994, 3VD9/3VDA 1995) | [VERIFICATO] | CMSNL, [oem-parts.hu](https://oem-parts.hu/en/parts/yamaha/motor-robogo/700cc-1099cc/tdm-850/tdm850-3vd5-1993/meter) |
| Altri codici prima serie | 4EP (Giappone 1992–94), 4CM (Svezia), 4DT (USA) | [DA VERIFICARE] | [monotaro](https://www.monotaro.com/bp/YAMAHA/c-60/b-7477/bk-438/), CMSNL, [tdm-forum t=33812](https://tdm-forum.net/viewtopic.php?t=33812) |
| Seconda serie | **4TX** dal 1996, albero a 270° (stesso motore della TRX 850). Impianto diverso, **non** usare i suoi schemi senza confronto | [VERIFICATO] | [manuale di servizio 1996](https://www.manualslib.com/manual/927445/Yamaha-Tdm850-1996.html) |

Il numero di telaio della moto conferma la variante esatta.

## 2. Quadro strumenti originale

| Elemento | Presente? | Note | Stato | Fonte |
|---|---|---|---|---|
| Tachimetro | Sì, **meccanico a cavo** | Rinvio a vite senza fine sul mozzo anteriore | [VERIFICATO] | [Haynes p.190](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=190), [tdm-forum t=16997](https://tdm-forum.net/viewtopic.php?t=16997) |
| Contachilometri totale e parziale | Sì, meccanici, dentro il tachimetro | Si perdono con la sostituzione: fotografare il valore prima | [VERIFICATO] | idem |
| Contagiri | Sì, elettronico | Codice 3VD-83540-00 | [VERIFICATO] | [bike-parts-yam](https://www.bike-parts-yam.com/yamaha-motorcycle/850-MOTO/1995/TDM/TDM850/SPEEDOMETER/28_2762-2762/B39/0/7260) |
| Termometro acqua | Sì, a lancetta | Forse integrato nel corpo del contagiri | [VERIFICATO] presenza, [DA VERIFICARE] posizione | [Haynes p.102](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=102), [manuale FR](https://www.manualslib.fr/manual/1046161/Yamaha-Tdm850.html) |
| Indicatore benzina / spia riserva | **No** | Solo rubinetto OFF/ON/RES. Arrivano con il modello 1999 | [VERIFICATO] | [TDMitalia FAQ](https://www.tdmitalia.net/?u=faq/tecniche002), [Haynes p.192](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=192) |
| Spia olio | **No** | Livello con astina sul tappo | [VERIFICATO] | [Haynes p.38](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=38), [tdm-forum t=2164](https://tdm-forum.net/viewtopic.php?t=2164) |
| Orologio | **No** | | [VERIFICATO] | idem |
| Spia folle | Sì, verde | | [VERIFICATO] | [tdm-forum t=27514](https://tdm-forum.net/viewtopic.php?t=27514) |
| Spia abbaglianti | Sì, blu | | [VERIFICATO] | idem |
| Spia frecce | Sì, **una sola** per entrambi i lati | Collegata tra le due linee delle frecce | [VERIFICATO] | [TDMFWiki LED-Blinker](https://wiki.tdm-forum.net/index.php/LED-Blinker_an_die_TDM), [tdm-forum t=29044](https://tdm-forum.net/viewtopic.php?t=29044) |
| Spia cavalletto laterale | Non trovata | | [IPOTESI] assente | — |
| Lampadine | T10 a cuneo, 12 V 3,4 W (spie e illuminazione) | Il forum riporta anche 2–3 W | [DA VERIFICARE] | [manuale 4TX 1996](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf), [tdm-forum t=4338](https://tdm-forum.net/viewtopic.php?t=4338) |
| Fissaggio | Due gommini antivibranti, distanziale, clip e dadi. Per i dadi inferiori va tolto il cupolino | | [VERIFICATO] | [oem-parts.hu](https://oem-parts.hu/en/parts/yamaha/motor-robogo/700cc-1099cc/tdm-850/tdm850-3vd5-1993/meter), [Haynes p.189](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=189) |
| Cablaggio del quadro | Cord set 3VD-83509-00 | Uno usato fornisce i connettori femmina giusti per l'adattatore | [VERIFICATO] codice | bike-parts-yam |
| Dimensioni del quadro | Non trovate | Da misurare | — | — |

## 3. Tachimetro e ruota anteriore

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Trasmissione | Cavo dal rinvio sul mozzo anteriore. Il rinvio ha linguette che si innestano nel mozzo e fa parte del pacco distanziali sull'asse | [VERIFICATO] | [Haynes p.190](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=190) |
| Rapporto del cavo | Non trovato per la TDM. Lo standard giapponese è 1400 giri cavo/km | [IPOTESI] | [xjbikes](https://xjbikes.com/forums/threads/gear-ratios-for-speedometer-and-tachometer.14114/) |
| Pneumatico anteriore | 110/80-18 | [VERIFICATO] | [Rider](https://ridermagazine.com/2007/05/22/retrospective-yamaha-tdm-850-1992-1993/), [TDMitalia](https://www.tdmitalia.net/?u=lemoto%2Fdatitdm850-91) |
| Pneumatico posteriore | 150/70-17 | [VERIFICATO] | idem |
| Circonferenza nominale anteriore | 457,2 + 2 × 88 = 633,2 mm di diametro → **1989 mm** | [IPOTESI] calcolo | — |
| Circonferenza di rotolamento | **1950–1970 mm** (schiacciamento sotto carico) | [IPOTESI] stima; si calibra col GPS | — |
| Bulloni dei dischi anteriori | Numero e materiale da rilevare (servono al sensore induttivo) | — | — |

Con il sensore induttivo sui bulloni il rapporto del cavo non serve: si misurano direttamente gli impulsi per giro di ruota.

## 4. Contagiri

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Sorgente del segnale | Negativo del primario della bobina, in parallelo alla centralina TCI. Il 3VD **non** ha un'uscita contagiri dedicata (la 4TX sì) | [VERIFICATO] | [tdm-forum t=6889](https://tdm-forum.net/viewtopic.php?t=6889), [t=7095](https://tdm-forum.net/viewtopic.php?t=7095), [t=33812](https://tdm-forum.net/viewtopic.php?t=33812) |
| Bobina | Una sola, 3VD-82310-00 (poi 3LD-82310-09), due cavi candela (scintilla persa) | [VERIFICATO] | [Yambits](https://yambits.co.uk/tdm850-ignition-coil-1991-1995-p-86771.html) |
| Resistenza bobina | Primario 2,38–3,22 Ω, secondario 12–18 kΩ | [DA VERIFICARE] | [tdm-forum t=32766](https://tdm-forum.net/viewtopic.php?t=32766) |
| Impulsi per giro | **1** (bicilindrico a 360°: "scoppia ogni giro come un due tempi") | [DA VERIFICARE] | tdm-forum t=6889 |
| Frequenze | 16,7 Hz a 1000 giri, 133 Hz a 8000 giri | [IPOTESI] calcolo | — |
| Forma del segnale | ≈ +12 V a riposo, ≈ 0 V mentre la bobina si carica, poi un picco induttivo di **alcune centinaia di volt** a ogni scintilla, con oscillazioni | [IPOTESI] comportamento tipico TCI | [Acewell](https://www.acewell-meter.com/wp-content/uploads/2016/02/ACE-27-28XX-EN-V5-manual-20150512.pdf), [Koso RX2N](https://www.manualslib.com/manual/1885543/Koso-Rx2n.html?page=16) |
| Colore del filo | Arancione (O) o grigio (Gy) | [IPOTESI] | [xjbikes](https://xjbikes.com/forums/threads/wiring-question-xj650-tach.125606/), [tenere.co.uk](https://tenere.co.uk/forum/viewtopic.php?p=27246) |
| Zona rossa | ≈ 8000 giri. Potenza massima a 7500 giri | [DA VERIFICARE] | [motorcyclespecs](https://www.motorcyclespecs.co.za/model/yamaha/yamaha_tdm850%2091.htm) |

## 5. Fili e connettori al quadro

Codice colori Yamaha: B nero, Br marrone, Ch cioccolato, Dg verde scuro, G verde, Gy grigio, L blu, O arancione, P rosa, R rosso, Sb azzurro, W bianco, Y giallo. Due lettere separate da `/` indicano filo e riga (es. `L/W` = blu con riga bianca). Fonte: [manuale TDM '96/'97](https://manualmachine.com/yamaha/tdm850/8415354-user-manual/).

| Funzione | Colore probabile | Logica | Stato | Fonte |
|---|---|---|---|---|
| +12 V sotto chiave | Br | +12 V con chiave ON | [DA VERIFICARE] | [supplemento TDM '99](https://tdm-trx-tracer.nl/forums/applications/core/interface/file/attachment.php?id=2543), [trx850.com](http://www.trx850.com/phpBB_forum/viewtopic.php?t=10256) |
| Massa | B | — | [IPOTESI] convenzione | — |
| Illuminazione quadro | L | +12 V con le luci di posizione | [IPOTESI] convenzione | — |
| Spia folle | Sb | La lampada riceve +12 V; l'interruttore del folle **chiude a massa** → attiva bassa | [IPOTESI] convenzione; interruttore a sinistra vicino al pignone ([t=36168](https://tdm-forum.net/viewtopic.php?t=36168)) | — |
| Spia abbaglianti | Y | +12 V con abbaglianti accesi → attiva alta | [IPOTESI] convenzione | — |
| Freccia sinistra / destra | Ch / Dg | +12 V lampeggiante; la spia unica è collegata **tra** Ch e Dg | [VERIFICATO] la spia unica; [DA VERIFICARE] i colori sul 3VD | [t=29044](https://tdm-forum.net/viewtopic.php?t=29044) |
| Contagiri | O o Gy | Negativo bobina | [IPOTESI] | sezione 4 |
| Sensore acqua | ? | Segnale verso la NTC a massa | [DA VERIFICARE] | — |
| Connettore | Si parla di un **6 poli** dietro gli strumenti, marca sconosciuta | — | [DA VERIFICARE] | [t=2164](https://tdm-forum.net/viewtopic.php?t=2164) |
| +12 V permanente al quadro | Probabilmente assente (non c'è orologio) | — | [IPOTESI] | — |

**Conseguenze per il progetto:**
- Il folle è attivo basso: tolta la lampadina, la linea resta flottante quando non si è in folle, quindi l'ingresso deve alimentare il proprio optoaccoppiatore da +12 V.
- Le frecce vanno lette con **due ingressi separati**, entrambi attivi a +12 V, ciascuno con il proprio carico di richiamo verso massa.
- Le stesse logiche usa anche Koso ("neutral (−)", "high beam (+12V)"), [manuale RX2](https://media.motoblouz.com/uploads/products_document/79537/notice-compteur-koso-rx2.pdf).

## 6. Lampeggiatore

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Tipo | Relè a 3 contatti, sensibile al carico | [DA VERIFICARE] | [TDMFWiki](https://wiki.tdm-forum.net/index.php/LED-Blinker_an_die_TDM) |
| Effetto della rimozione della spia | Il carico della spia (3,4 W) è piccolo rispetto alle frecce (2 × 21 W per lato): la frequenza non dovrebbe cambiare | [IPOTESI] | — |

## 7. Temperatura acqua

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Strumento | Lancetta, nessuna spia (la spia arriva nel 1999) | [VERIFICATO] | [Haynes p.102](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=102) |
| Sensore | Nel corpo del termostato, un filo, NTC verso massa | [DA VERIFICARE] | idem |
| Resistenza | **47–53 Ω a 80 °C**, **26–29 Ω a 100 °C** (una copia dice 26–30 Ω) | [DA VERIFICARE] | idem |
| Altri punti della curva | Non trovati: si misurano a bagnomaria (20/40/60/80/90/100 °C) | — | — |
| Alternativa | Adattatore Koso M14×1,5 per sensore proprio, indicato per TDM 850 | [DA VERIFICARE] | [kedo](https://www.kedo.com/koso-adaptor-m14x1-5-15mm-for-temp-sensor-oil-water-1-8-x28tpi.html) |

## 8. Ricarica e batteria

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Alternatore | Rotore a magneti permanenti, trifase, tre fili bianchi, ≈ 280 W (il forum dice anche 300–340 W) | [DA VERIFICARE] | [tdm-forum t=34705](https://tdm-forum.net/viewtopic.php?t=34705) |
| Regolatore | Shindengen SH629A-12 (3VD-81960-00/-01), a derivazione (shunt) | [VERIFICATO] | [officine08](https://www.officine08.com/p67451_regulator-yamaha-tdm-850.html), [eBay](https://www.ebay.de/itm/143232889593) |
| Punto debole noto | Regolatore e connettore dello statore che si bruciano | [VERIFICATO] | forum vari |
| Tensione di ricarica | 14,0–14,5 V sopra 1200–2000 giri | [DA VERIFICARE] | forum |
| Batteria | **YTX12-BS**, 12 V 10 Ah | [VERIFICATO] | [Yuasa](https://www.motoracingshop.com/it/batteria-yuasa-per-yamaha-tdm-850-91-95-ytx12-bs-da-12v-10ah-152x88x131.html) |
| Fusibili | Fari 15 A, segnali 15 A, accensione 7,5 A (dati 4TX) | [DA VERIFICARE] sul 3VD | [manuale 4TX](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf) |
| Disturbi | Un regolatore guasto può danneggiare l'elettronica del contagiri; un morsetto positivo allentato fa ballare le lancette | [VERIFICATO] | [tenere.co.uk](https://www.tenere.co.uk/forum/viewtopic.php?t=1479), [tdm-forum t=18602](https://tdm-forum.net/viewtopic.php?t=18602) |

## 9. Carburante

| Dato | Valore | Stato | Fonte |
|---|---|---|---|
| Serbatoio | 18 L, di cui 3,5 L di riserva (la 4TX: 20 L / 3,1 L) | [VERIFICATO] | [manuale d'uso 3VD](https://www.scribd.com/document/696775450/User-manual-TDM850) |
| Sensore | Nessuno | [VERIFICATO] | sezione 2 |

L'autonomia si può solo **stimare** dai km percorsi dall'ultimo pieno (funzione prevista dopo la v1.0).

## 10. Documentazione di riferimento

**Manuali:**
- **Manuale d'uso 3VD (3VD-28199)**, con schema elettrico pieghevole nelle copie cartacee: [manualslib](https://www.manualslib.com/manual/1937486/Yamaha-Tdm850.html), [Scribd](https://www.scribd.com/document/696775450/User-manual-TDM850), [archive.org (1995)](https://archive.org/details/YamahaTDM850OwnersManual1995), [edizione francese](https://www.manualslib.fr/manual/1046161/Yamaha-Tdm850.html).
- **Haynes 3540** (TDM850, TRX850, XTZ750), con schemi elettrici a colori: [manualslib](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html).
- **Manuali di servizio 4TX** (simili, non uguali): [rv96](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf), ['97](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20'97%20Service%20Manual.pdf), [supplemento 1997](https://archive.org/details/printermanual-yamaha-tdm-850-factory-1997-service-manual-supplement).
- **Manuale d'officina 3VD** (aftermarket, PDF): [retromotorcyclemanuals](https://retromotorcyclemanuals.com/en-us/products/1991-1995-yamaha-tdm850-tdm-850-service-manual).
- **Schema di collegamento Ignitech per TDM 850**, utile per i colori ai pin della TCI: [ignitech](https://www.ignitech.cz/en/vyrobky/tci/mototci.htm).

**Fonti italiane:** [tdmitalia.net (dati 1991)](https://www.tdmitalia.net/?u=lemoto%2Fdatitdm850-91), [guida "Fai da te" Motosprint](https://www.tdmitalia.net/documenti/manuali/FaidateTDM850.pdf). Attenzione: il "libretto uso e manutenzione" su [ManualMachine](https://manualmachine.com/yamaha/tdm850/10910352-user-manual/) è della **4TX**, non del 3VD.

**Forum:** [tdm-forum.net, sezione elettrica](https://tdm-forum.net/viewforum.php?f=21), [Carpe TDM](http://www.carpe-tdm.net/ipb/index.php?showtopic=36935), [tdm-trx-tracer.nl](https://tdm-trx-tracer.nl/forums/forum/4-tdm/).

## 11. Progetti simili

- **Koso, Acewell, motogadget** montati su TDM e TRX: [trx850.com t=10550](http://www.trx850.com/phpBB_forum/viewtopic.php?t=10550), [tdm-forum t=35804](https://tdm-forum.net/viewtopic.php?t=35804), [t=20197](https://tdm-forum.net/viewtopic.php?t=20197).
- **Convertitori dal cavo al segnale elettronico:** Daytona dichiara compatibili TDM850 e TRX850 ([caferacergarage](https://caferacergarage.com/product/daytona-speed-pulse-converter/)); Koso "JIS Type A" con filetto M12×1,0 ([RevZilla](https://www.revzilla.com/motorcycle/koso-speed-signal-converter-for-koso-speedometers)).
- **Contagiri da bobina su altre moto:** il negativo della bobina dà letture più stabili di un filo avvolto sul cavo candela ([kzrider](https://www.kzrider.com/forum/4-electrical/98375-update-on-acewell-digital-speedo-tach)); una resistenza da 1 kΩ con un condensatore aiuta ([wildguzzi](https://wildguzzi.com/forum/index.php?topic=94616.0)).
- Non è stato trovato nessun progetto fai-da-te (Arduino o simili) per la TDM.

## 12. Discrepanze tra le fonti

| Tema | Discrepanza | Come la risolviamo |
|---|---|---|
| Indicatore benzina | Alcune recensioni lo citano, ma si riferiscono al modello 1999 | Il 3VD non lo ha |
| Lampadine | 3,4 W (manuale 1996), 1,4/2 W (1999), 2–3 W (forum) | Ininfluente: le togliamo |
| Serbatoio | 18 L / 3,5 L (3VD) contro 20 L / 3,1 L (4TX) | Usiamo il dato 3VD |
| Alternatore | 280 W contro 300–340 W | Ininfluente: il quadro consuma ≈ 5 W |
| Filo contagiri XTZ 750 | Verde/giallo in un post, arancione → grigio in un altro | Si misura sulla moto |
| Anni del 4EP | 1992–94 Giappone contro un'inserzione 1999 | Ininfluente |
