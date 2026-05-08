# RULES — Bao la Kiswahili

> **Status: SKELET (fase 0).**  Geen Rust-code uit `crates/bao-engine` mag op een uitspraak in dit document leunen totdat de betreffende sectie de status `CONFIRMED` draagt en de gebruiker de bronsynthese expliciet heeft goedgekeurd. Lege of `TBD`-secties zijn blokkers voor `cargo` werk in fase 1.

Dit document is de **enige** waarheid over Bao-regels in deze repo. De Rust-engine, de UI en de training-pipeline verwijzen naar genummerde regels hier in commentaren (bv. `// see RULES.md §5.3`). Bij bronconflict is een hier vastgelegde keuze leidend; de afwijzing wordt in §A (conflict-log) genoteerd.

## Hoe deze file te lezen

Per regelgebied:
1. **Statement** — de gekozen interpretatie, in één paragraaf.
2. **Sources** — tabel met bronvermelding per claim.
3. **Conflicts** — voetnoot-style flags wanneer bronnen botsen; volledige uitwerking in §A.
4. **Status** — `TBD` (nog niet ingevuld), `DRAFT` (ingevuld, niet bevestigd), `CONFIRMED` (door user geaccepteerd).

Variant-specifieke regels worden in twee kolommen weergegeven (Kiswahili | Kujifunza) zodra Kujifunza-bronnen geraadpleegd zijn. Per user-keuze (zie plan, Open Questions §1 → opgelost in CLAUDE.md) is Kujifunza een feature-flag op dezelfde engine; verwacht dus dat de meeste regels gedeeld zijn en alleen specifieke regels (overgang namu→mtaji, mogelijk nyumba-bestaan) verschillen.

## Bronnen

| Sigla | Bron | Status |
|-------|------|--------|
| **R** | Russ — exacte editie nog te bevestigen (Open Question #3 in plan); waarschijnlijk *Mancala Games* (1984) door Laurence Russ. Het oorspronkelijke plan citeerde "*Bao: A Game for Two Players*" — die titel kon niet eenduidig worden teruggevonden | TBD |
| **dV** | Alex de Voogt, etnografisch werk over Bao (Zanzibar/Lamu) | TBD |
| **BS** | Bao Society / FIDE Mind Sports toernooiregels | TBD |
| **G** | `geziefer/baolakiswahili` — werkende BGA-implementatie door Alexander Rühl, gebruikt als gedragsoracle. `states.inc.php` is de state-graph; `baolakiswahili.game.php` (~100KB) bevat de regelafhandeling | DRAFT (begin observaties beschikbaar) |

Per-bron citaat-extracten leven in `docs/rules-sources/`.

## Notatie (intern aan dit document)

- **Spelers**: South (kant onder, rij 1–2 in BAN) en North (kant boven, rij 3–4).
- **Kolommen**: `a`–`h`, links→rechts vanuit South.
- **Rijen**: `1` = South-mbele, `2` = South-nyuma, `3` = North-nyuma, `4` = North-mbele.
- **Pit-coördinaat**: `<col><row>`, bv. `d1` = vierde kolom van links in South-mbele.
- **Richting**: `>` = klokwijzers vanuit het bord-perspectief (South's mbele naar rechts), `<` = tegenklokwijzers.

Zie `CLAUDE.md` voor de Swahili termen (kete, shimo, vichwa, kimbi, nyumba, mbele, nyuma, ghala, namu, mtaji, kula, takata, endelea, hamna, mkononi, zamu, mchezaji). De volgende termen zijn ontdekt via bron G en nog niet in CLAUDE.md verwerkt — fase-0 actie:

| Term | Vermoedelijke betekenis | Bron | Status |
|------|--------------------------|------|--------|
| `kunamua` | Werkwoord van namu (als spelactie) | G state 10–11 | TBD |
| `safari` | Speler-keuze tijdens namu-capture; vermoedelijk welke kant kete "op reis gaan" | G state 13 | TBD |
| `kichwa-selectie` | Speler-keuze welk vichwa te gebruiken na namu-capture | G state 12 | TBD |

## 1. Bord-topologie

### 1.1 Layout
- Het bord telt 4 rijen × 8 kolommen = **32 vichwa totaal**, gelijk verdeeld: **16 per speler** (2 rijen × 8 kolommen).
- Mbele = front row (richting opponent); nyuma = back row (eigen kant).
- Kichwa: extreme kolommen van mbele (`a` en `h`). 4 kichwa-posities op het bord (2 per speler).
- Kimbi: positie naast kichwa in mbele. Of dat `b` en `g` zijn (één-na-extreem) of overlapt met kichwa zelf is bron-afhankelijk; CLAUDE.md's glossary noemt kichwa als "first or last" en kimbi als "ultimate and penultimate", wat onderling consistent te maken is op meerdere manieren — TBD per bron.
- Nyumba: vaste positie per kant, kolomindex TBD per bron.

**Sources**: TBD
**Status**: DRAFT (geometrie staat vast in literatuur; nyumba-positie is bron-afhankelijk)

### 1.2 Nyumba-positie

| Variant | Kolom-index van nyumba (South en North spiegelend) | Bron |
|---------|---------------------------------------------------|------|
| Kiswahili | TBD | TBD |
| Kujifunza | n.v.t. of TBD | TBD |

**Status**: TBD

### 1.3 Sow-richting topologie (next_pit-functie)

De volgorde van vichwa langs een sow-pad volgt een lus over beide eigen rijen, met **richtingsomkering bij kichwa**. De plan-versie zegt dat sowing wraps van mbele naar nyuma bij de kichwa, maar de exacte specificatie verschilt per bron. TBD.

**Status**: TBD

## 2. Beginpositie

> Blokkerende edit K1 uit de review. Geen engine-werk vóór deze tabel ingevuld is.

### 2.1 Bao la Kiswahili

| Pit | Kete bij start van namu | Bron |
|-----|--------------------------|------|
| Ghala South | TBD | TBD |
| Ghala North | TBD | TBD |
| Nyumba South | TBD | TBD |
| Nyumba North | TBD | TBD |
| Aangrenzende vichwa zuidelijk | TBD | TBD |
| Aangrenzende vichwa noordelijk | TBD | TBD |
| Overige vichwa | TBD | TBD |
| **Som invariant** | **64** | algemeen |

### 2.2 Bao la Kujifunza

Kujifunza begint typisch direct in een mtaji-achtige fase met alle kete reeds op het bord (geen ghala-fase). Te bevestigen.

| Pit | Kete bij begin | Bron |
|-----|----------------|------|
| Alle vichwa | TBD | TBD |

**Status**: TBD

### 2.3 Beginspeler

| Variant | Wie begint | Beperkingen op eerste zet | Bron |
|---------|-----------|----------------------------|------|
| Kiswahili | TBD | TBD | TBD |
| Kujifunza | TBD | TBD | TBD |

**Status**: TBD

## 3. Spelfases

### 3.1 Namu (eerste fase, alleen Kiswahili)

In namu plaatst een speler bij elke beurt één kete uit zijn ghala in een eigen mbele-shimo. Of er aansluitend een kula plaatsvindt en hoe de daaropvolgende sow verloopt is bron-afhankelijk.

**Sources**: TBD
**Status**: TBD

### 3.2 Mtaji (tweede fase Kiswahili / enige fase Kujifunza)

In mtaji kiest de speler een eigen niet-leeg shimo, kiest een richting, en sowt alle kete uit dat shimo. Capture- en endelea-regels gelden zoals in §5–§7.

**Sources**: TBD
**Status**: TBD

### 3.3 Overgang namu → mtaji (Kiswahili)

| Conditie | Bron |
|----------|------|
| Beide ghala leeg | TBD |
| Eén ghala leeg + ? | TBD |
| Plyn-cap | TBD |

**Status**: TBD

## 4. Zet-types

| Type | Variant | Beschrijving | Voorwaarden | BAN | Bron |
|------|---------|--------------|--------------|-----|------|
| Namu kula | Kiswahili-namu | Plaatsing met directe capture | TBD | `N:d1>` of `N:d1<` | TBD |
| Namu takata | Kiswahili-namu | Plaatsing zonder capture | TBD (alleen toegestaan als geen kula-zet bestaat?) | `N:d1~` | TBD |
| Mtaji | beide | Sowing met capture aan einde | landen op eigen mbele met opponent-mbele-tegenoverliggend niet-leeg | `e1>` of `e1>*` | TBD |
| Takata | beide | Sowing zonder capture | alleen toegestaan als geen mtaji-zet bestaat? | `e1>` zonder `*` | TBD |

**Mandatory-kula-regel**: bestaat voor mtaji (alle bronnen consistent). Voor namu: TBD per bron.

**Status**: TBD

## 5. Sowing

### 5.1 Algemene sow-loop

1. Speler kiest bron-shimo (en richting bij mtaji; bij namu plaatst men slechts één kete uit ghala).
2. Bij mtaji: pak alle kete uit het bron-shimo.
3. Beweeg één-voor-één in de gekozen richting, één kete per stap, tot de hand leeg is. Volg de topologie uit §1.3 (kichwa-omkering).
4. Het laatste shimo bepaalt vervolgactie: lege landing → einde zet (mogelijk takata); niet-lege landing met capture-conditie → kula (§6); niet-lege landing zonder capture-conditie → endelea (§7).

**Sources**: TBD
**Status**: TBD

### 5.2 Nyumba tijdens sowing

Of de sow-loop nyumba overslaat, in nyumba moet stoppen, of de speler een keuze geeft, hangt af van of nyumba "fed" is (zie §8). De drie mogelijke regels:

| Toestand | Actie bij aankomst |
|----------|--------------------|
| Pre-fed (immune) | TBD: stop / skip / choice |
| Post-fed | TBD: stop / skip / choice |
| Vernietigd | normaal shimo (geen speciale behandeling) |

**Sources**: TBD
**Status**: TBD

## 6. Capture (kula)

### 6.1 Kula in mtaji

Conditie (volgens oorspronkelijk plan, te valideren): laatste kete landt in eigen mbele en het tegenoverliggende opponent-mbele-shimo is niet-leeg. Dan worden de kete uit dat opponent-shimo (en mogelijk opponent-nyuma erachter) genomen en in een eigen rij gesowt vanaf een specifiek startpunt.

| Element | Specificatie | Bron |
|---------|--------------|------|
| Welke shimo wordt gepakt | TBD: alleen tegenoverliggende mbele, of ook nyuma? | TBD |
| Waar worden de gepakte kete heen gesowt | TBD: vanaf eigen kichwa? eigen kimbi? bronshimo? | TBD |
| Mag speler richting kiezen | TBD; bij sommige bronnen verplicht naar kichwa, bij andere keuze | TBD |

**Sources**: TBD
**Status**: TBD

### 6.2 Kula in namu

Conditie (te valideren): bij plaatsen van één kete uit ghala in een eigen mbele-shimo waarvan het tegenoverliggende opponent-mbele-shimo niet-leeg is, worden die opponent-kete genomen en gesowt. Bron-conflict over of de speler een richting kiest of dat de kolom de richting forceert (Open Question #4 in plan).

**Sources**: TBD
**Status**: TBD

### 6.3 Kichwa-selectie sub-state

`geziefer` heeft een aparte state (12 `kunamuaCaptureSelection`) waarin de speler na een capture-trigger moet kiezen welke kichwa te gebruiken voor de daaropvolgende sow. Bevestig of dat een echte regel-keuze is of een UI-detail.

**Sources**: G (state 12)
**Status**: TBD (Open Question #10 in plan)

### 6.4 Safari-beslissing

`geziefer` state 13 `safariDecision` is nog onverklaard. Vermoedelijk een variant van richting-keuze of een keuze om méér dan de standaard kete-set te kapen ("op safari gaan"). Onderzoek vereist via de bron-code.

**Sources**: G (state 13)
**Status**: TBD (Open Question #9 in plan)

## 7. Endelea (relay sowing)

### 7.1 Conditie

Wanneer de hand leeg raakt en de laatste kete in een **niet-leeg** shimo wordt gedropt (dus dat shimo bevat ≥ 2 kete na de drop) **én** de capture-conditie níet vervuld is, gaat de zet door als endelea: pak alle kete uit dat shimo en sow vanaf daar in dezelfde richting.

**Sources**: TBD
**Status**: TBD

### 7.2 Terminatie

Endelea eindigt bij óf een capture óf een lege landing. Edge case: een hand kan in theorie meerdere laps cyclen; de implementatie hanteert een 256-hop hard cap (zie plan §2.4) als sanity-check; legitiem maximum is veel lager.

**Sources**: algemeen / plan
**Status**: DRAFT (regel zelf is bron-onafhankelijk; alleen de hop-cap is implementatiekeuze)

## 8. Nyumba

### 8.1 Bestaan en positie

Geldt alleen voor Kiswahili (en Kujifunza? — TBD). Vaste positie per speler (zie §1.2).

### 8.2 Initiële vulling

| Veld | Waarde | Bron |
|------|--------|------|
| Initiële kete in nyumba | typisch 6 (te bevestigen) | TBD |

### 8.3 Immuniteit en feeding

`nyumba` heeft een speciale capture-immune status totdat hij voor de eerste keer "gevoed" wordt. Wat "voeden" exact betekent (een kete erin droppen tijdens sow? alleen tijdens namu? alleen tijdens een specifieke zet?) is bron-afhankelijk.

| Element | Specificatie | Bron |
|---------|--------------|------|
| Wat "feeding" inhoudt | TBD | TBD |
| Wanneer immuniteit eindigt | TBD | TBD |
| Sowing pre-fed: stop, skip, of keuze? | TBD | TBD |
| Sowing post-fed: stop, skip, of keuze? | TBD | TBD |

**Status**: TBD (kritiek; zie plan-risico "Regel-correctheid")

### 8.4 Vernietiging

Wanneer nyumba tijdens een sow leeggehaald en niet hervuld wordt, verliest hij in sommige tradities zijn speciale status permanent. TBD per bron.

**Status**: TBD

## 9. Wincondities

### 9.1 Hamna

Een speler wint wanneer alle opponent-mbele-vichwa leeg zijn na een gemaakte zet (kula heeft alle kete uit opponent-mbele weggehaald). Te valideren of "alle mbele leeg" of "geen legale zet voor opponent" de daadwerkelijke winnaar-conditie is.

**Sources**: TBD
**Status**: DRAFT

### 9.2 Mkononi

Een speler wint hamna tijdens namu (vóór de overgang naar mtaji). In sommige tradities geeft dit extra prestige maar verandert niets aan de score; in andere is het de enige manier om in namu te winnen. TBD.

**Sources**: TBD
**Status**: TBD

### 9.3 Stalemate

Wanneer de actieve speler in mtaji geen legale zet meer heeft. Uitkomst: verlies (oorspronkelijk plan) of remise (sommige bronnen). Open Question #5 in plan.

| Bron | Verlies / remise / anders |
|------|---------------------------|
| R | TBD |
| dV | TBD |
| BS | TBD |
| G | TBD |

**Status**: TBD

## 10. Bao la Kujifunza — verschillen met Kiswahili

> Per user-keuze: minimaal — Kujifunza is een feature-flag op de engine, geen aparte UI of ELO-pool. CLAUDE.md is overeenkomstig bijgewerkt.

| Element | Kiswahili | Kujifunza | Bron |
|---------|-----------|-----------|------|
| Bestaat namu-fase | ja | nee (start direct in mtaji) | algemeen, te bevestigen |
| Beginpositie | zie §2.1 | zie §2.2 | TBD |
| Nyumba-mechaniek | aanwezig | TBD (mogelijk afwezig of vereenvoudigd) | TBD |
| Mandatory kula | aanwezig | TBD | TBD |
| Wincondities | hamna, mkononi, stalemate | hamna; mkononi vervalt zonder namu | logisch gevolg |

**Status**: TBD

## A. Conflict-log

Wanneer twee of meer bronnen onverenigbaar verschillen op een specifieke regel, wordt het conflict hier vastgelegd met de gekozen interpretatie en de afwijzing.

| ID | Conflict-onderwerp | Bron-uitspraken | Gekozen interpretatie | Reden | User-bevestigd |
|----|---------------------|------------------|------------------------|-------|----------------|
| C1 | TBD | TBD | TBD | TBD | nee |

(Bij invulling: één rij per conflict. ID's zijn permanent — de Rust-code mag eraan refereren via `// see RULES.md §A C1`.)

## B. Open Questions die dit document moet oplossen vóór fase 1

(Spiegeling van Open Questions uit het plan, alleen die het regelwerk raken.)

- **#2** Initiële bordpositie per variant — invullen in §2.
- **#3** Russ-editie bevestigen — invullen in `Bronnen`-tabel.
- **#4** Namu-kula richtingskeuze (forced of player choice) — invullen in §6.2.
- **#5** Stalemate-uitkomst (verlies of remise) — invullen in §9.3.
- **#9** Wat is "safari"? — invullen in §6.4.
- **#10** Kichwa-selectie als sub-state? — invullen in §6.3.
- **#11** Hus blijft out-of-scope (geen sectie nodig).
