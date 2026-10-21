# 13 · Manutenzione

Il quadro non ha parti in movimento e il suo software non si "sporca": la scheda SD è in sola lettura. La manutenzione riguarda soprattutto il sensore, i connettori e le tenute.

## Controlli periodici

| Quando | Cosa | Come |
|---|---|---|
| Ogni uscita | Spie e quadro all'accensione | Il test delle spie all'avvio deve accendere tutto; le frecce devono lampeggiare anche sui LED di backup |
| Ogni 1000 km | Sensore ruota | Distanza dai bulloni 1–2 mm, dadi del sensore serrati, cavo integro lungo la forcella |
| Ogni 5000 km o ogni anno | Connettori | Superseal e DTM: niente ossido, guarnizioni al loro posto. Un velo di grasso dielettrico sulle guarnizioni, non sui contatti |
| Ogni anno | Contenitore | Finestra senza crepe, guarnizione EPDM elastica, viti serrate. Niente condensa all'interno; se c'è, controlla la valvola di sfiato |
| Ogni anno | Copia di sicurezza | Copia `tdm-dash.toml` dalla scheda SD sul PC; annota i km (sono nella FRAM, non sulla SD) |
| Dopo un lavaggio con idropulitrice | Contenitore | Non dirigere il getto sul contenitore né sul connettore del sensore |

## Cambio gomme

La circonferenza della gomma cambia tra marche, misure e usura (fino al 2–3%). Dopo un cambio della gomma **anteriore**:

1. Se la misura è la stessa (110/80-18), lascia il profilo com'è per i primi 300 km.
2. Poi rifai la calibrazione della circonferenza col GPS (capitolo 10).
3. Con una misura diversa, calcola subito una circonferenza provvisoria. Ti serve il diametro della gomma, che dipende da cerchio (pollici), larghezza (mm) e spalla (%):
   - **Diametro:** cerchio × 25,4 + 2 × larghezza × spalla / 100. Per una 110/80-18: 18 × 25,4 + 2 × 110 × 0,80 = 633 mm.
   - **Circonferenza:** diametro × 3,1416 × 0,97 (schiacciamento sotto carico): circa 1930 mm.
   - Poi calibra col GPS.

Smontando la ruota anteriore, il sensore resta sulla forcella. Al rimontaggio controlla la distanza dai bulloni: dipende dalla posizione del disco.

## Cambio dei dischi o dei bulloni

Il sensore legge le teste dei bulloni del disco anteriore, che devono essere **d'acciaio magnetico**. Con dischi nuovi:
- **Bulloni:** contali, e se il numero cambia aggiorna `pulses_per_rev` nella sezione `[speed]` del profilo.
- **Bulloni inox (non magnetici):** il sensore li vede male. Usa bulloni d'acciaio, oppure incolla sotto le teste delle rondelle d'acciaio.
- **Distanza:** controlla di nuovo la distanza del sensore.

## Aggiornamenti

- **Immagine del Pi:** si riscrive la scheda SD con la nuova versione (capitolo 09). Prima copia `tdm-dash.toml` e rimettilo sulla partizione di boot dopo la scrittura. I km non si perdono: sono nella FRAM.
- **Firmware del Pico:** con `tdmctl bootsel` dall'immagine di sviluppo, o col pulsante BOOTSEL se il contenitore è aperto (capitolo 09). Immagine e firmware della **stessa release** vanno aggiornati insieme: se il protocollo cambia, il quadro mostra "Firmware del modulo non compatibile".

## Sostituzione di componenti

| Componente | Cosa resta e cosa rifare |
|---|---|
| Scheda SD | Riscrivi l'immagine e copia il profilo: nient'altro |
| Raspberry Pi Zero 2 W | Sposta la scheda SD sul nuovo Pi |
| Pico | Carica il firmware sul nuovo Pico. Km e impostazioni restano nella FRAM |
| FRAM | I km si perdono: imposta l'ultimo valore noto con `tdmctl init-odo` (capitolo 08). Per questo conviene annotare i km ogni tanto |
| Display | Controlla i tempi in `config.txt` se cambia il modello |
| Sensore ruota | Stesso modello: solo la regolazione della distanza |
| Sensore dell'acqua | Rifai la curva a bagnomaria se cambia la marca (capitolo 10) |

## Rimessaggio invernale

- **Batteria:** a moto ferma per mesi staccala, o tienila col mantenitore di carica. Il quadro non consuma a chiave tolta: è alimentato solo sotto chiave.
- **Contenitore:** controlla che sia asciutto all'interno prima di coprire la moto.
- **Alla ripresa:** accendi a motore spento e verifica spie, tensione e temperatura (sensore a temperatura ambiente).
