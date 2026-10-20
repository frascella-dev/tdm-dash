# 02 · Architettura del sistema

## In breve

tdm-dash è diviso in due computer, ognuno col compito che sa fare meglio:

| Parte | Compito | Perché |
|---|---|---|
| **Raspberry Pi Pico (RP2040)**, microcontrollore | Conta gli impulsi di ruota e bobina, legge spie, pulsanti e tensioni, **custodisce i km** in una memoria FRAM, comanda la retroilluminazione | È pronto in pochi millisecondi dall'accensione, misura i tempi con precisione al microsecondo, non si blocca e non ha un file system da rovinare |
| **Raspberry Pi Zero 2 W**, computer Linux | Disegna l'interfaccia grafica sul display | Ha la potenza per grafica fluida a 800×480; con Linux minimale si avvia in pochi secondi |

I due comunicano via **USB** con pacchetti piccoli e protetti da CRC. Il Pi non scrive mai sulla scheda SD durante l'uso: se si gira la chiave di colpo non si rovina nulla.

## Schema a blocchi

```mermaid
flowchart LR
  subgraph MOTO["Impianto TDM 850 (3VD)"]
    KEY["+12V sotto chiave (Br)"]
    SIG["Spie: folle Sb · abbaglianti Y · frecce Ch/Dg · luci L"]
    COIL["Negativo bobina (contagiri)"]
    NTC["Sensore acqua NTC"]
  end
  subgraph PWR["Alimentazione protetta"]
    F["Fusibile 2A"] --> RP["P-MOSFET anti-inversione"] --> TVS["TVS SMCJ24A"] --> LC["Filtro LC"] --> BUCK["Buck 50V→5V D36V28F5"]
  end
  KEY --> F
  subgraph IO["Condizionamento segnali"]
    OPTO["Opto EL817 ×5"]
    TACH["Zener + H11L1"]
    ADC["Partitori / pull-up + clamp"]
  end
  SIG --> OPTO
  COIL --> TACH
  NTC --> ADC
  KEY --> ADC
  WHEEL["Sensore induttivo LJ12A3 (bulloni disco)"] --> OPTO
  LUX["TEMT6000 luce ambiente"] --> ADC
  BTN["Pulsanti MODE / SET"] --> MCU
  OPTO --> MCU
  TACH --> MCU
  ADC --> MCU
  MCU["RP2040 Pico · Rust embassy · FRAM km"] -- "USB CDC (postcard+COBS+CRC)" --> PI["Pi Zero 2 W · Buildroot · Rust/Slint"]
  BUCK --> MCU
  BUCK --> PI
  PI -- DPI --> LCD["LCD 4,3 pollici 800×480 1000 nit"]
  MCU -- "PWM retroilluminazione" --> BL["Boost TPS61169"] --> LCD
  SIG --> LED["LED passivi di backup"]
```

## Il percorso di un dato: dalla ruota alla lancetta

```mermaid
flowchart LR
  A["Bullone del disco<br/>passa davanti al sensore"] --> B["LJ12A3: uscita NPN<br/>a massa"]
  B --> C["Optoaccoppiatore<br/>12 V → 3,3 V"]
  C --> D["RP2040: fronte con<br/>timestamp in µs"]
  D --> E["PulseMeter: media su<br/>giri interi di ruota"]
  E --> F["FRAM: +distanza<br/>ogni 10 m"]
  E --> G["Telemetry 50 Hz<br/>via USB"]
  G --> H["Pi: velocità = distanza/tempo,<br/>mediana, filtro"]
  H --> I["Margine di legge<br/>×1,03 +1 km/h"]
  I --> J["Lancetta (molla)<br/>e cifre (isteresi)"]
```

Perché la media "su giri interi": i bulloni del disco possono non essere distanziati in modo perfettamente uguale. Misurando sempre un numero intero di giri di ruota, l'errore di spaziatura si annulla.

## Software

```mermaid
flowchart TB
  subgraph PI["Raspberry Pi Zero 2 W"]
    direction TB
    FW["Firmware GPU (start_cd.elf)"] --> K["Kernel Linux 6.12 ridotto<br/>DRM vc4 · DPI · USB ACM · watchdog"]
    K --> INIT["BusyBox init"]
    INIT --> APP["tdm-dash (Rust)<br/>logica + UI Slint"]
    APP --> DRM["DRM/KMS<br/>doppio buffer + page-flip"]
  end
  subgraph PICO["Raspberry Pi Pico"]
    direction TB
    E["Rust + embassy-rp"] --> T1["Task impulsi<br/>(alta priorità)"]
    E --> T2["Task ADC e ingressi"]
    E --> T3["Task FRAM"]
    E --> T4["Task USB"]
    E --> T5["Task retroilluminazione"]
  end
  T4 <-- "USB CDC" --> APP
```

| Componente | Scelta | Dove nel repo |
|---|---|---|
| Sistema operativo | Buildroot 2026.08, rootfs EROFS in sola lettura, nessun udev, nessuna rete nell'immagine di rilascio | `os/` |
| Interfaccia | Rust + Slint, renderer software, piattaforma DRM propria | `crates/tdm-dash/` |
| Logica (filtri, avvisi, stati) | Rust, senza dipendenze dall'hardware, testata sul PC | `crates/tdm-logic/` |
| Protocollo | postcard + COBS + CRC-16, crate condiviso | `crates/tdm-proto/` |
| Misure e contachilometri | Rust `no_std`, **lo stesso codice** nel firmware e nel simulatore | `crates/tdm-core/` |
| Firmware | Rust + embassy-rp | `firmware/` |
| Profili moto | TOML | `profiles/` |

## Avvio

```mermaid
sequenceDiagram
  autonumber
  participant K as Chiave
  participant M as Pico (MCU)
  participant P as Pi Zero 2 W
  participant D as Display
  K->>M: +12 V
  Note over M: ≈ 10 ms: legge FRAM,<br/>conta già impulsi e km
  K->>P: +5 V
  Note over P: firmware GPU, kernel,<br/>init (obiettivo < 4 s)
  P->>D: primo frame (retroilluminazione ancora spenta)
  P->>M: GetInfo
  M-->>P: Hello (versione, configurazione, km)
  P->>M: SetConfig (se diversa dal profilo)
  P->>M: SetBacklight (dissolvenza 300 ms)
  M->>D: PWM retroilluminazione
  Note over P,D: self-test: lancette 0 → fondo scala → 0
  loop ogni 20 ms
    M-->>P: Telemetry
  end
  loop ogni 250 ms
    P->>M: Heartbeat
  end
```

Mentre Linux si avvia il microcontrollore lavora già: se si parte subito, i km percorsi nei primi secondi non vanno persi.

## Stati dell'interfaccia

```mermaid
stateDiagram-v2
  [*] --> Splash: primo page-flip
  Splash --> SelfTest: collegamento ok, velocità < 5
  Splash --> Guida: collegamento ok, velocità ≥ 5<br/>(riavvio in marcia)
  Splash --> Guida: nessun collegamento dopo 3 s<br/>(avviso Collegamento perso)
  SelfTest --> Guida: 1,2 s, o velocità ≥ 5
  state Guida {
    [*] --> Principale
    Principale --> Viaggio: MODE breve
    Viaggio --> Diagnostica: MODE breve (se abilitata)
    Diagnostica --> Principale: MODE breve
    Viaggio --> Principale: MODE breve
  }
  Guida --> Impostazioni: SET lungo, velocità < 3
  Impostazioni --> Guida: uscita, 20 s di inattività o velocità ≥ 5
  Guida --> Spento: PowerFail
  Impostazioni --> Spento: PowerFail
  Spento --> [*]
```

Gli **avvisi** non sono stati, ma uno strato sopra la schermata attiva:
- **Critico** (fascia rossa): collegamento perso, temperatura critica, sovratensione.
- **Attenzione** (banner ambra), da silenziare per 5 minuti con SET: temperatura alta, mancata ricarica, batteria bassa.
- **Informazione** (icona): freccia dimenticata accesa, stima benzina.

## Spegnimento e cadute di tensione

```mermaid
sequenceDiagram
  participant K as Chiave / batteria
  participant M as Pico (MCU)
  participant P as Pi
  K->>M: tensione < 7 V per 2 ms
  M->>M: salva km in FRAM (≈ 1 ms, riserva 470 µF)
  M-->>P: PowerFail
  M->>M: spegne retroilluminazione
  Note over P: rootfs in sola lettura:<br/>può perdere corrente in qualsiasi momento
```

Il Pi non ha bisogno di uno spegnimento ordinato. La FRAM sopporta circa 10¹² scritture: anche scrivendo ogni 10 m, in 100 000 km si fanno 10⁷ scritture, una frazione minima della durata.

## Sicurezza di funzionamento ("fail-dark")

| Guasto | Cosa succede | Perché |
|---|---|---|
| L'app del Pi si blocca | Dopo 2 s senza Heartbeat il Pico spegne la retroilluminazione; il watchdog riavvia il Pi | Meglio uno schermo nero che una velocità ferma e sbagliata |
| Il collegamento USB cade | Dopo 0,3 s cifre attenuate; dopo 1 s "---" e avviso critico; riconnessione automatica | I dati vecchi non devono sembrare validi |
| Il Pico si riavvia | Il Pi se ne accorge (uptime che riparte), azzera i filtri | Nessun salto delle lancette |
| La FRAM è illeggibile | Il Pico funziona, segnala il guasto, non sovrascrive i dati | Non si perde lo storico dei km |
| Elettronica spenta o guasta | I **LED di backup** sulle linee a 12 V mostrano frecce, abbaglianti e folle | Spie obbligatorie sempre visibili |

## Estensibilità: i profili moto

Tutto ciò che dipende dalla moto sta in un file TOML (`profiles/`):
- impulsi per giro di ruota e circonferenza
- impulsi per giro motore, zona rossa
- curva del sensore della temperatura, soglie della batteria
- quali ingressi sono collegati e con che logica
- layout e tema

Il file in uso si trova sulla partizione `/boot` della scheda SD, leggibile e modificabile da qualsiasi PC. Per un'altra moto basta un profilo nuovo; se il profilo ha un errore, il quadro usa quello integrato e lo segnala.
