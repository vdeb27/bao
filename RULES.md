# RULES — Bao la Kiswahili

> **Status: SKELET met geziefer-DRAFT (fase 0).**  Geen Rust-code uit `crates/bao-engine` mag op een uitspraak in dit document leunen totdat de betreffende sectie de status `CONFIRMED` draagt en de gebruiker de bronsynthese expliciet heeft goedgekeurd. `DRAFT (geziefer-only)` betekent: één bron geconsulteerd, tegen-bronnen nodig vóór `CONFIRMED`.

Dit document is de **enige** waarheid over Bao-regels in deze repo. De Rust-engine, de UI en de training-pipeline verwijzen naar genummerde regels hier in commentaren (bv. `// see RULES.md §5.3`). Bij bronconflict is een hier vastgelegde keuze leidend; de afwijzing wordt in §A (conflict-log) genoteerd.

## Hoe deze file te lezen

Per regelgebied:
1. **Statement** — de gekozen interpretatie, in één paragraaf.
2. **Sources** — tabel met bronvermelding per claim.
3. **Conflicts** — voetnoot-style flags wanneer bronnen botsen; volledige uitwerking in §A.
4. **Status** — `TBD` (nog niet ingevuld), `DRAFT` (ingevuld, niet bevestigd), `CONFIRMED` (door user geaccepteerd).

Variant-specifieke regels worden in twee kolommen weergegeven (Kiswahili | Kujifunza). Per user-keuze (zie plan, Open Questions §1 → opgelost in CLAUDE.md) is Kujifunza een feature-flag op dezelfde engine.

## Bronnen

| Sigla | Bron | Status |
|-------|------|--------|
| **R** | Russ — exacte editie nog te bevestigen (Open Question #3 in plan); waarschijnlijk *Mancala Games* (1984) door Laurence Russ. Het oorspronkelijke plan citeerde "*Bao: A Game for Two Players*" — die titel kon niet eenduidig worden teruggevonden | TBD |
| **dV** | Alex de Voogt, etnografisch werk over Bao (Zanzibar/Lamu) | TBD |
| **BS** | Bao Society / FIDE Mind Sports toernooiregels | TBD |
| **G** | `geziefer/baolakiswahili` — werkende BGA-implementatie door Alexander Rühl, gebruikt als gedragsoracle. Volledig geanalyseerd; bevindingen in `docs/rules-sources/geziefer_bga.md` | DRAFT (extracted) |

Per-bron citaat-extracten leven in `docs/rules-sources/`.

## Notatie (intern aan dit document)

- **Spelers**: South (kant onder, rij 1–2 in BAN) en North (kant boven, rij 3–4).
- **Kolommen**: `a`–`h`, links→rechts vanuit South.
- **Rijen**: `1` = South-mbele, `2` = South-nyuma, `3` = North-nyuma, `4` = North-mbele.
- **Pit-coördinaat (BAN)**: `<col><row>`, bv. `d1` = vierde kolom van links in South-mbele.
- **Field-index (geziefer-conventie)**: per speler 0..16, waarbij 0=ghala, 1..8=eigen mbele links→rechts, 9..16=eigen nyuma rechts→links. Field 9 = direct boven field 8; field 16 = direct boven field 1.
- **Richting**: `>` = klokwijzers vanuit het bord-perspectief (South's mbele naar rechts), `<` = tegenklokwijzers.

Zie `CLAUDE.md` voor de Swahili termen. De volgende termen zijn ontdekt via bron G:

| Term | Betekenis | Bron | Status |
|------|-----------|------|--------|
| `kunamua` | Werkwoord van namu (de namu-zet uitvoeren) | G | DRAFT (G-only) |
| `safari` | Speler-keuze tijdens een capture-sow: of de eigen functional nyumba "geplunderd" wordt om de sow voort te zetten. Niet-namu-related zoals oorspronkelijk vermoed — alleen tijdens mtaji of namu-capture-sow | G regel 1408–1428, 1745–1756 | DRAFT (G-only) |
| `kichwa-selectie` | Speler-keuze welk kichwa-shimo (field 1 of 8) gebruikt wordt voor de capture-sow. Deterministisch in 4 van 5 condities; alleen player-choice bij middle-capture met onbepaalde richting | G regel 442–482 | DRAFT (G-only) |
| `kutakatia` | Mtaji-fase blokkeermechanisme: na bepaalde takata-zetten wordt een opponent-veld "gemarkeerd" zodat het de volgende beurten alleen-via-capture-aanraakbaar is | G regel 630–697 | DRAFT (G-only) |
| `tax` (taxing nyumba) | Geziefer-specifiek: namu-takata vanaf eigen functional nyumba haalt slechts 2 kete eruit i.p.v. de hele inhoud | G regel 1168–1177 | DRAFT (G-only, mogelijk huisregel) |

## 1. Bord-topologie

### 1.1 Layout
- Het bord telt 4 rijen × 8 kolommen = **32 vichwa totaal**, gelijk verdeeld: **16 per speler** (2 rijen × 8 kolommen).
- Mbele = front row (richting opponent); nyuma = back row (eigen kant).
- Kichwa: extreme kolommen van mbele (`a` en `h`). 4 kichwa-posities op het bord (2 per speler).
- Kimbi: positie naast kichwa in mbele. Of dat `b` en `g` zijn (één-na-extreem) of overlapt met kichwa zelf is bron-afhankelijk; CLAUDE.md's glossary noemt kichwa als "first or last" en kimbi als "ultimate and penultimate" — TBD per bron. Geziefer behandelt fields 1–2 als "left kichwa/kimbi" en 7–8 als "right kichwa/kimbi" voor kichwa-selectie-logica.
- Nyumba: vaste positie per kant — zie §1.2.

**Sources**: G (bord-comment regel 21–27)
**Status**: DRAFT (G-only; kichwa/kimbi-precieze positionering nog te bevestigen)

### 1.2 Nyumba-positie

| Variant | Kolom-index van nyumba | Bron |
|---------|------------------------|------|
| Kiswahili (South) | field 5 = kolom `e` (5e van links) | G regel 308–314 |
| Kiswahili (North) | field 4 = kolom `e` vanuit Norths perspectief, fysiek = kolom `d` van het bord (4e van Souths links) | G regel 308–314 |
| Kujifunza | n.v.t. (geen nyumba-mechaniek) | G regel 850–875 (variant-conditioneel) |

Mirroring is per-speler; beide spelers zien hun eigen nyumba op "field 5" of "field 4" afhankelijk van wie South/North is. Zie `docs/rules-sources/geziefer_bga.md §1` voor field-encoding-uitleg.

**Status**: DRAFT (G-only)

### 1.3 Sow-richting topologie (next_pit-functie)

Sowing is een **simpele ringbuffer** over fields 1..16 met richting +1 of −1 en wrap (0→16, 17→1). Geen aparte regel voor kichwa-omkering: de fysieke "U-bocht" in mbele→nyuma volgt automatisch uit de field-numbering (field 8 +1 → field 9 = nyuma direct erboven; field 1 −1 → field 16 = nyuma boven field 1).

**Code**: G regel 582–590 (`getNextField`).

**Status**: DRAFT (G-only)

## 2. Beginpositie

### 2.1 Bao la Kiswahili

| Field | Speler 1 (South) | Speler 2 (North) | Totaal |
|-------|------------------|------------------|--------|
| 0 (ghala) | 22 | 22 | 44 |
| 1–4 | 0,0,0,0 | 0,2,2,**6** | (zie nyumba) |
| 5 | **6** (nyumba) | 0 | |
| 6 | 2 | 0 | |
| 7 | 2 | 0 | |
| 8 | 0 | 0 | |
| 9–16 | alle 0 | alle 0 | 0 |
| **Per-speler totaal** | 22 + 6 + 2 + 2 = 32 | 22 + 6 + 2 + 2 = 32 | 64 |

Visualisatie (Souths perspectief; Norths bord is 180° gespiegeld):
```
North-nyuma:   .  .  .  .  .  .  .  .   (alle 0)
North-mbele:   .  2  2  6  .  .  .  .   (nyumba=field 4)
                                          (Norths field 1 ligt rechts; gespiegeld)
South-mbele:   .  .  .  .  6  2  2  .   (nyumba=field 5)
South-nyuma:   .  .  .  .  .  .  .  .   (alle 0)

Ghala South: 22 kete
Ghala North: 22 kete
```

**Sources**: G regel 112–134 (`setupNewGame` Kiswahili branch)
**Status**: DRAFT (G-only)

### 2.2 Bao la Kujifunza

Alle 16 vichwa per speler bevatten 2 kete; ghala = 0. Geen namu-fase. Het spel begint direct in mtaji-state.

| Field | Speler 1 | Speler 2 |
|-------|----------|----------|
| 0 (ghala) | 0 | 0 |
| 1–16 | alle 2 | alle 2 |
| **Per-speler totaal** | 32 | 32 |

**Sources**: G regel 135–145 (`setupNewGame` Kujifunza/else branch)
**Status**: DRAFT (G-only)

### 2.3 Beginspeler

Wie begint wordt door de runtime bepaald (in BGA: door BGA-framework). Geen specifieke restrictie op de eerste zet anders dan de mandatory-kula-regel (§4).

**Sources**: G regel 95
**Status**: DRAFT (G-only)

## 3. Spelfases

### 3.1 Namu (eerste fase, alleen Kiswahili)

In namu plaatst een speler bij elke beurt één kete uit zijn ghala in een eigen mbele-shimo. De verdere afhandeling is afhankelijk van of die plaatsing een capture triggert (§6.2). Als een capture mogelijk is, mag de speler alleen een capturing-zet kiezen (mandatory kula).

**Sources**: G regel 1701–1743 (`argKunamuaMoveSelection`), regel 1134–1163 (executeMove namu branch)
**Status**: DRAFT (G-only)

### 3.2 Mtaji (tweede fase Kiswahili / enige fase Kujifunza)

In mtaji kiest de speler een eigen vichwa met ≥2 kete, kiest een richting (+1 of −1), en sowt alle kete uit dat shimo. Capture- en endelea-regels gelden zoals in §5–§7.

**Sources**: G regel 1758–1788, regel 335–440 (move-generation), regel 1267–1401 (executeMove mtaji branch)
**Status**: DRAFT (G-only)

### 3.3 Overgang namu → mtaji (Kiswahili)

**Conditie**: BEIDE ghala's tegelijk leeg, gecheckt na elke namu-zet.

```php
// G regel 1875–1880
if (VARIANT_KISWAHILI && 
    $board[$playerLast][0]["count"] == 0 && $board[$playerNext][0]["count"] == 0) {
    // switch to '2nd' phase
}
```

Geen ply-cap; geen "één ghala leeg" overgang.

**Sources**: G regel 1875–1893
**Status**: DRAFT (G-only)

## 4. Zet-types

| Type | Variant | Beschrijving | Voorwaarden | BAN | Bron |
|------|---------|--------------|--------------|-----|------|
| Namu kula | Kiswahili-namu | Plaatsing van ghala-kete + capture-sow vanaf gekozen kichwa | Eigen mbele-pit `i` (≥1 kete vóór drop) ∧ opponent mbele `i` (≥1 kete) bestaat | `N:d1>` of `N:d1<` (richting volgt uit kichwa-keuze) | G regel 1153–1162, 484–505 |
| Namu takata | Kiswahili-namu | Plaatsing van ghala-kete + sow zonder capture | Geen kula-zet beschikbaar, plus nyumba-restricties (§8.5) | `N:d1~` | G regel 507–567, 1163–1266 |
| Namu tax | Kiswahili-namu | Speciaal: vanaf functional nyumba haal slechts 2 kete | Functional nyumba is enige niet-lege mbele OR (zie §8.5) | TBD | G regel 1168–1177 |
| Mtaji | beide | Sow met capture aan einde | 2..15 kete in source ∧ landing ≤8 ∧ landing in own mbele non-empty ∧ opponent same column non-empty | `e1>` of `e1>*` | G regel 335–394 |
| Takata | beide | Sow zonder capture | Geen mtaji-zet beschikbaar; mbele-vichwa met ≥2 kete eerst, anders nyuma | `e1>` zonder `*` | G regel 396–440 |

**Mandatory-kula**: bevestigd actief in zowel namu als mtaji (G regel 1031–1035, 1063–1067, 1710–1715, 1763–1769).

**Status**: DRAFT (G-only)

## 5. Sowing

### 5.1 Algemene sow-loop

1. Speler kiest source (en richting).
2. In namu (Kiswahili): één kete uit ghala wordt geplaatst in source mbele-pit. Bij capture: zet stopt na placement, wacht op kichwa-keuze. Bij non-capture: sow van de inhoud van het source-pit (na placement) in gekozen richting.
3. In mtaji: pak alle kete uit source-pit (behalve voor functional-nyumba-tax). Sow.
4. Sow distribueert één kete per stap in gekozen richting via `getNextField`.
5. Termination per type:
   - **Takata** (geen capture): empty landing → einde zet. Non-empty landing met `count > 1` na drop → endelea (§7).
   - **Capture-sow** (na kichwa-keuze): zelfde plus mid-sow checks voor `continueCapture` (volgende capture mogelijk) en `decideSafari` (eigen functional nyumba) — zie §6.

**Sources**: G regel 1183–1264 (Kiswahili-namu sow), regel 1481–1530 (capture-sow), regel 1322–1396 (Kujifunza/2nd-phase sow)
**Status**: DRAFT (G-only)

### 5.2 Nyumba tijdens sowing

| Toestand | Actie bij aankomst |
|----------|--------------------|
| Functional (≥6, in bezit) tijdens namu-takata-sow | als bron: alleen 2 kete uit (tax); als doorgangs-pit: gewoon doorgaan tenzij endelea zou triggeren — dan stop (G regel 1249–1264) |
| Functional tijdens capture-sow | als landing met `count > 1` → safari-decision (§6.4) |
| Non-functional (<6, in bezit) | normaal pit, geen speciale behandeling |
| Vernietigd | normaal pit |

**Sources**: G regel 1168–1177, 1249–1264, 1514–1522
**Status**: DRAFT (G-only; tax-regel mogelijk huisregel)

## 6. Capture (kula)

### 6.1 Kula in mtaji

**Conditie**: source `i` heeft 2..15 kete, sow in gekozen richting, landing `j` voldoet aan:
- `j ≤ 8` (in eigen mbele)
- eigen mbele `j` had vóór drop ≥1 kete (dus ≥2 erna)
- opponent mbele `j` heeft ≥1 kete

**Capture-actie**: kete uit opponent's mbele `j` worden gepakt. Geen kete uit nyuma worden in deze code gepakt — alleen mbele.

**Sow van gepakte kete**: vanaf gekozen kichwa (zie §6.3), in de richting die door kichwa-keuze geïmpliceerd wordt.

**Sources**: G regel 335–394, 1278–1302, 1429–1530
**Status**: DRAFT (G-only)

### 6.2 Kula in namu

**Conditie**: voor mbele-veld `i`: countPlayer ≥ 1 vóór de namu-drop ∧ countOpponent ≥ 1.

Na de namu-plaatsing van één kete is de pit ≥ 2; daarna gaat de sow-flow precies zoals mtaji-capture: sow vanaf gekozen kichwa.

**Richtingskeuze (Open Question #4)**: in geziefer is de richting in namu-kula NIET op het moment van veld-keuze gekozen — de speler kiest pas indirect via kichwa-selectie (§6.3). Bij captures in middle-mbele (cols 3–6) heeft de speler dan een echte keuze (left vs right kichwa); bij rand-captures (cols 1, 2, 7, 8) wordt de richting geforceerd.

**Sources**: G regel 484–505, 1153–1162, 442–482
**Status**: DRAFT (G-only)

### 6.3 Kichwa-selectie (state 12)

Na een capture-trigger (zowel namu als mtaji) wordt bepaald welke kichwa-pit (field 1 of field 8) als startpunt voor de capture-sow dient. Logica uit `getPossibleKichwas`:

| `captureField` (mbele waar capture plaatsvindt) | `moveDirection` van vorige zet | Beschikbare kichwa(s) |
|------------------|------------------|------------------------|
| 1 of 2 (left kichwa/kimbi) | irrelevant | LEFT (field 1) |
| 7 of 8 (right kichwa/kimbi) | irrelevant | RIGHT (field 8) |
| 3..6 (middle) | +1 (clockwise vanuit eerdere zet) | LEFT |
| 3..6 (middle) | −1 (counter-clockwise) | RIGHT |
| 3..6 (middle) | 0 (geen vorige direction; namu-kula) | **BOTH (player kiest)** |

In 4 van 5 condities is kichwa-keuze deterministisch; alleen middle-capture-with-no-prior-direction (typisch: namu-kula in cols 3–6) is een echte player-keuze. Dat pleit voor een **sub-state** `Phase::AwaitKichwa` in plaats van een Move-veld dat altijd gezet moet zijn.

**Sources**: G regel 442–482
**Status**: DRAFT (G-only)

### 6.4 Safari (state 13) — beslissing tijdens capture-sow

**Trigger**: tijdens een capture-sow (na kichwa-selectie, in de do-while loop) komt de sow uit in de **eigen functional nyumba** met `count > 1` na drop. De zet stopt; speler moet kiezen.

**Keuzes**:
- **STOP**: zet eindigt; nyumba blijft intact. State = `nextPlayer`.
- **DOORGAAN ("go on safari")**: nyumba wordt geleegd, gemarkeerd als destroyed, sow continues met de geleegde kete. Notation krijgt `+`.

```php
// G regel 1418–1428
$count = $board[$player][$sourceField]["count"];
$board[$player][$sourceField]["count"] = 0;
array_push($moves, "emptyActive_" . $sourceField);
// ... continues into the same do-while sow loop
$this->checkAndMarkDestroyedNyumba($player, $sourceField);
```

**Vereist**: een sub-state `Phase::AwaitSafari` in onze FSM en een `Move::Safari(bool)` actie. Geen namu-mechanisme — alleen tijdens een capture-sow die toevallig in eigen nyumba uitkomt.

**Sources**: G regel 1408–1428, 1514–1522, 1745–1756
**Status**: DRAFT (G-only)

## 7. Endelea (relay sowing)

### 7.1 Conditie

Wanneer de hand leeg raakt en de laatste kete in een **niet-leeg** shimo wordt gedropt (dus pit-count > 1 na drop) **én** de capture-conditie níet vervuld is (geen mtaji-capture, geen safari-trigger), pak alle kete uit dat shimo en sow vanaf daar in dezelfde richting.

**Sources**: G regel 1214–1264, 1327–1396, 1481–1530 (drie verschillende sow-loops, allemaal hetzelfde patroon)
**Status**: DRAFT (G-only; klassiek bekende regel)

### 7.2 Terminatie

Endelea eindigt bij:
- Empty landing (`count == 1` na drop).
- Mtaji-capture-trigger (in capture-sow only).
- Functional-nyumba-encounter in capture-sow (safari-decision).
- Functional nyumba van speler-zelf in eigen takata-sow (stop, do not empty).

**Hard cap**: G heeft 12 full rounds van het bord; daarna wordt de speler "zombie" (verklaart automatisch verloren). Onze 256-hop cap (plan §2.4) is iets strakker.

**Sources**: G regel 1233–1244, 1351–1362
**Status**: DRAFT

## 8. Nyumba

### 8.1 Bestaan en positie

Geldt alleen voor Kiswahili. Vaste positie per speler (zie §1.2). In Kujifunza is geen nyumba-mechaniek (alle nyumba-checks zijn variant-conditioneel).

**Sources**: G regel 308–314, 850–875
**Status**: DRAFT (G-only)

### 8.2 Initiële vulling

| Veld | Waarde | Bron |
|------|--------|------|
| Initiële kete in nyumba | 6 | G regel 116, 126 |

**Status**: DRAFT (G-only)

### 8.3 Toestanden (functional / non-functional / destroyed)

Geziefer onderscheidt drie toestanden:
- **Functional** (state 0): nog in bezit ∧ ≥6 kete
- **Non-functional** (state 1): nog in bezit ∧ <6 kete (tijdelijk leeg geraakt of nooit gevoed tot 6)
- **Destroyed** (state 2): is door eigen sow geleegd; permanent

| Element | Specificatie | Bron |
|---------|--------------|------|
| Wanneer wordt nyumba destroyed | wanneer de eigen kete-source-pit nyumba is en ge-emptyd wordt voor een nieuwe sow-lap | G regel 699–709, 1186, 1261, 1394, 1465 |
| Wanneer wordt nyumba non-functional | wanneer de count <6 wordt door enige reden | G regel 606–613 |
| "Voeden" / fed-status | geen aparte fed-flag in geziefer; immuniteit komt uit functional/destroyed-onderscheid | G (geen feeding-flag) |
| Sow-pre-fed: stop, skip, of keuze | n.v.t. (geen feeding-onderscheid in geziefer) | G |
| Sow op functional nyumba | als landing met `count > 1` in capture-sow → safari; in takata-sow als endelea-stop | G regel 1252, 1518 |
| Sow op non-functional nyumba | normaal pit | G |

**Status**: DRAFT (G-only). Geziefer's functional/non-functional/destroyed model wijkt mogelijk af van klassieke "fed/unfed"-terminologie. Onderzoek vereist.

### 8.4 Tax-regel (geziefer-specifiek?)

Wanneer namu-takata vanaf eigen functional nyumba wordt gespeeld én de nyumba is de bron, worden **slechts 2 kete** uit de nyumba gehaald i.p.v. de hele inhoud. Vervolgens wordt normaal gesowd vanaf nyumba+1 met 2 kete.

```php
// G regel 1168–1177
if ($sourceField == $nyumba && $wasNyumbaFunctional) {
    $count = 2;
    $board[$player][$sourceField]["count"] -= $count;
    array_push($moves, "taxActive_" . $sourceField);
    ...
}
```

**Mogelijk huisregel**. Bevestigen tegen klassieke bron.

**Sources**: G regel 1168–1177
**Status**: DRAFT (G-only, mogelijk niet canon — flag voor secundaire bron)

### 8.5 Kunamua non-capture restricties

`getKunamuaPossibleNonCaptures` (G regel 507–567) heeft drie takken:

1. **Geen possession** (nyumba destroyed): mbele-pit met ≥2 kete heeft prioriteit; alleen als geen ≥2 bestaat, mag een ≥1-pit gebruikt worden.
2. **Functional nyumba**: kies elk niet-leeg mbele-veld dat NIET de nyumba is; alleen als geen ander gevuld mbele-veld bestaat, mag de nyumba zelf (dan triggert de tax-regel §8.4).
3. **Non-functional nyumba**: elk niet-leeg mbele-veld is toegestaan zonder restrictie.

**Sources**: G regel 507–567
**Status**: DRAFT (G-only)

## 9. Wincondities

### 9.1 Hamna

Speler verliest wanneer de score 0 wordt. Score = totaal kete als (canMove ∧ !mbele-leeg), anders 0. Mbele-leeg → verlies (= hamna in klassieke termen).

**Conditie-checks**: G regel 877–893.

**Sources**: G regel 843–894
**Status**: DRAFT (G-only)

### 9.2 Mkononi

Score = 0 in 1st phase wanneer mbele leeg is — gebeurt automatisch via dezelfde getScore-logica. Geen aparte mkononi-detectie in code; functioneel hetzelfde als hamna in namu-fase.

**Sources**: G regel 851–859
**Status**: DRAFT (G-only)

### 9.3 Stalemate

In geziefer: een speler die geen vichwa met ≥2 kete heeft (in mtaji) krijgt score 0 → verlies. Dus stalemate = verlies, niet remise.

```php
// G regel 866–874 (variant != Kiswahili 1st phase)
if ($count >= 2) {
    $canMove = true;
}
// regel 888–893
if ($canMove && !$isEmpty) {
    return $sum;
} else {
    return 0;  // lost
}
```

| Bron | Verlies / remise / anders |
|------|---------------------------|
| R | TBD |
| dV | TBD |
| BS | TBD |
| G | **VERLIES** |

**Status**: DRAFT (G-only — Open Question #5 voorlopig beantwoord met G; bevestiging tegen R/dV/BS gewenst voor `CONFIRMED`)

### 9.4 12-rounds-zombie

Geziefer-specifieke safeguard: als een sow ≥12 keer rond het bord gaat (impossible volgens normaal play, maar potentieel bij edit-mode), wordt de speler "zombie" en verliest. Niet een klassieke regel, maar een implementatie-detail.

**Sources**: G regel 1233–1244, 1351–1362
**Status**: DRAFT (G-only, niet canoniek)

## 10. Bao la Kujifunza — verschillen met Kiswahili

> Per user-keuze: minimaal — Kujifunza is een feature-flag op de engine, geen aparte UI of ELO-pool. CLAUDE.md is overeenkomstig bijgewerkt.

| Element | Kiswahili | Kujifunza | Bron |
|---------|-----------|-----------|------|
| Bestaat namu-fase | ja | nee — start direct in mtaji-state | G regel 135–145, 1843–1846 |
| Beginpositie | nyumba 6 + 2,2 + ghala 22 (zie §2.1) | alle 16 vichwa = 2 kete; ghala = 0 | G |
| Nyumba-mechaniek | aanwezig (functional/non-functional/destroyed) | n.v.t. (variant-conditioneel) | G regel 850–875 |
| Tax-regel | actief | n.v.t. | G |
| Safari | actief in mtaji-capture-sow op eigen functional nyumba | n.v.t. | G |
| Kutakatia | actief in 2nd phase | n.v.t. | G |
| Mandatory kula | actief | actief | G |
| canMove-conditie | ≥1 in mbele (1st phase) of ≥2 in elk vichwa (2nd) | ≥2 in elk vichwa | G regel 851–875 |
| Wincondities | hamna, mkononi, stalemate, zombie | hamna, stalemate, zombie | G |

**Sources**: G (variant-conditional code branches throughout)
**Status**: DRAFT (G-only)

## 11. Kutakatia (Kiswahili 2nd phase only)

Een blokkeermechanisme dat optreedt na bepaalde takata-zetten in mtaji. Geziefer's interpretatie:

**Trigger** (`checkAndMarkKutakatia`, G regel 630–697): na een mtaji-takata-zet wordt gekeken naar opponent's volgende-beurt-mogelijkheden:
- Exact één veld is voor opponent te kapen (door speler-zelf, in volgende beurt) — `count($possibleCapturedFields) == 1`
- Opponent kan zelf geen capture maken — `count($possibleOpponentsCaptures) == 0`

**Exclusies** (regel 678–684) — kutakatia activeert NIET als het te-blokkeren-opponent-veld:
- De functional nyumba van opponent is, OR
- Opponent's enige niet-lege mbele-veld is, OR
- Opponent's enige mbele-veld met ≥2 kete is

**Effect**: het veld wordt 3 zetten lang (`kutakatiaMoves = 3`, decrementing per beurt) "geblokkeerd":
- Geblokkeerde speler mag het niet legen via een takata
- Blokkerende speler MOET het kapen wanneer mogelijk (mtaji-captures worden gefilterd)

In notation krijgt zo'n zet een `*`.

**Implicatie voor onze engine**: `BoardState` heeft minimaal een `kutakatia: Option<KutakatiaState>`-veld nodig met `{blocked_field, blocked_player, moves_remaining}`.

**Sources**: G regel 630–697, 343–354, 405–416, 1771–1786, 1896–1913
**Status**: DRAFT (G-only — bevestigen tegen klassieke bronnen)

## A. Conflict-log

Geen conflicten yet (we hebben nog maar één geconsulteerde bron). Verwachte conflicten:
- Tax-regel (§8.4): mogelijk huisregel in geziefer
- Functional/non-functional/destroyed model (§8.3) vs klassiek "fed/unfed"
- Stalemate-uitkomst (§9.3) bij confrontatie met de Voogt
- Mandatory kula in namu (§3.1) — verschillende auteurs hebben verschillende strengheid
- "No suicide"-regel (geziefer-skipped, line 1005–1008)

| ID | Conflict-onderwerp | Bron-uitspraken | Gekozen interpretatie | Reden | User-bevestigd |
|----|---------------------|------------------|------------------------|-------|----------------|
| C1 | TBD | TBD | TBD | TBD | nee |

(Eerste daadwerkelijke conflict-rij komt na 2e bron-consultatie.)

## B. Open Questions die dit document moet oplossen vóór fase 1

(Spiegeling van Open Questions uit het plan.)

| # | Vraag | Geziefer's antwoord | Status |
|---|-------|---------------------|--------|
| 2 | Initiële bordpositie per variant | §2.1 + §2.2 ingevuld | DRAFT (G-only) |
| 3 | Russ-editie bevestigen | (geziefer geeft geen antwoord) | TBD |
| 4 | Namu-kula richtingskeuze (forced of player choice) | §6.2: indirect player-keuze via kichwa-selectie; meestal forced bij rand-captures, keuze bij middle | DRAFT (G-only) |
| 5 | Stalemate-uitkomst (verlies of remise) | §9.3: verlies | DRAFT (G-only) |
| 9 | Wat is "safari"? | §6.4: capture-sow in eigen functional nyumba — keuze ja/nee | DRAFT (G-only) |
| 10 | Kichwa-selectie als sub-state? | §6.3: ja, sub-state `AwaitKichwa` nodig (player-keuze in 1 van 5 condities) | DRAFT (G-only) |
| 11 | Hus uitsluiten? | §10 + introductie: ja, Hus blijft out-of-scope | CONFIRMED (user-keuze) |

Resterende open vraagstukken die geziefer NIET dekt (alleen relevant in latere fases):
- Welke specifieke editie van Russ
- De echte feeding/immuniteit-mechaniek (geziefer heeft alleen functional/non-functional/destroyed)
- Kutakatia-precisie (blokkering-duur, exclusies)
- "No suicide"-regel
- Tax-regel als canon of huisregel

Geen van bovenstaande is een blokker voor fase-1-engine-werk MITS we accepteren dat fase 1 op DRAFT (G-only) regels begint en de engine deze regels achteraf kan bijgewerken zodra een tweede bron beschikbaar is. Voor fase 1 zonder secundaire bronraadpleging stelt dat de regels zoals geziefer ze interpreteert als baseline vast.
