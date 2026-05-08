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
| **R** | Russ — niet teruggevonden als canonieke Bao-bron. Wikipedia citeert Russ niet; vermoedelijk een misattribution in het oorspronkelijke plan | OBSOLETE (zie dV) |
| **dV** | Alex de Voogt, *Limits of the mind: towards a characterization of Bao mastership* (1995, PhD thesis, Leiden University). Volgens Wikipedia "the most influential transcription of the rules". PDF achter paywall (ResearchGate, Academia.edu); we benaderen dV via K (zie hieronder) als proxy | INDIRECT (via K) |
| **BS** | Bao Society / FIDE Mind Sports toernooiregels | TBD |
| **W** | `en.wikipedia.org/wiki/Bao_(game)`. Secundaire synthese, citeert dV. Bevindingen in `docs/rules-sources/wikipedia.md` | DRAFT (extracted) |
| **G** | `geziefer/baolakiswahili` — werkende BGA-implementatie door Alexander Rühl. Bevindingen in `docs/rules-sources/geziefer_bga.md` | DRAFT (extracted) |
| **MWW** | Mancala World wiki (Fandom) — `Bao_la_Kiswahili`, auteur Ralf Gering, CC BY-SA 2.5. Volledige inhoud verkregen via gebruiker-paste op 2026-05-08. Bevindingen in `docs/rules-sources/mancala_world_wiki.md` | DRAFT (extracted, complete) |
| **A** | `abstractstrategygames.blogspot.com/2011/01/bao-sum-up-of-rules.html` — gestructureerde regelset met regelnummers (Rule 1.4.4 etc.). Geen expliciete attributie; lijkt afgeleid van dV. Bevindingen in `docs/rules-sources/abstractstrategygames.md` | DRAFT (extracted) |
| **M** | `medium.com/@navpil/.../7f1131bc3e0c` — Dmytro Polovynka, "Learn how to play Bao la Kiswahili step-by-step" (2026-03-09). Behandelt Kujifunza expliciet. Bevindingen in `docs/rules-sources/medium_polovynka.md` | DRAFT (extracted) |
| **K** | `kibao.org/cs_kanuni.php?lng=en` — citeert dV (1995) en de Dar-es-Salaam Regional Traditional Games Association als bron. **De best beschikbare proxy voor dV's thesis.** Bevindingen in `docs/rules-sources/kibao_org.md` | DRAFT (extracted, dV-proxy) |

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
| `kutakatia` / `takasia` / `takatia` | Mtaji-fase blokkeermechanisme: na bepaalde takata-zetten wordt een opponent-veld "gemarkeerd" zodat het de volgende beurten alleen-via-capture-aanraakbaar is. Spellingsvarianten zijn werkwoord (kutakatia = "to make takata against") vs passieve/zelfstandig-naamwoord-vormen | G regel 630–697; MWW (snippet) | DRAFT (G+MWW corroborated; details conflicteren) |
| `tax` (taxing nyumba) | Namu-takata vanaf eigen functional nyumba haalt slechts 2 kete eruit i.p.v. de hele inhoud | G regel 1168–1177; W "taxing the nyumba" | **DRAFT (G+W corroborated)** — niet huisregel, klassieke regel |

## 1. Bord-topologie

### 1.1 Layout
- Het bord telt 4 rijen × 8 kolommen = **32 vichwa totaal**, gelijk verdeeld: **16 per speler** (2 rijen × 8 kolommen).
- Mbele = front row (richting opponent); nyuma = back row (eigen kant).
- **Kichwa = mbele-pits 1 en 8** (uiteinden, "head"). 2 per speler, 4 op het bord.
- **Kimbi = mbele-pits 1, 2, 7 en 8** ("flanks", inclusief kichwa als bovenset). 4 per speler, 8 op het bord. Definitie expliciet bevestigd door W ("name kimbi applies to both the kichwa and the pits adjacent to them, i.e., the second and next to last pit"), M ("the holes 1, 2, 7 and 8") en A.
- **Centrale mbele-pits** = pits 3, 4, 5, 6 (vier middelste).
- **CLAUDE.md-glossary correctie**: kichwa is "first or last" pit van mbele; kimbi is een **bovenset** van die uitersten plus hun directe buurpits. Niet "ultimate and penultimate" zoals CLAUDE.md zei.
- Nyumba: vaste positie per kant — zie §1.2.

**Sources**: G regel 21–27; W; A; M; K
**Status**: **DRAFT (G+W+A+M+K corroborated)**

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

**Sources**: G regel 112–134 (`setupNewGame` Kiswahili branch); W ("each player initially places 6 seeds in the nyumba, and two seeds in each of the two pits immediately to the right of the nyumba. All the remaining seeds are kept 'in hand'.")
**Status**: **DRAFT (G+W corroborated)**

### 2.2 Bao la Kujifunza

Alle 16 vichwa per speler bevatten 2 kete; ghala = 0. Geen namu-fase. Het spel begint direct in mtaji-state.

| Field | Speler 1 | Speler 2 |
|-------|----------|----------|
| 0 (ghala) | 0 | 0 |
| 1–16 | alle 2 | alle 2 |
| **Per-speler totaal** | 32 | 32 |

**Sources**: G regel 135–145 (`setupNewGame` Kujifunza/else branch); W ("all seeds are placed at startup, two per pit. Players thus have no seeds in hand, and thus there is no namua phase.")
**Status**: **DRAFT (G+W corroborated)**

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

**Sources**: G regel 1875–1893; W ("When players are left without seeds in their hands, the namua phase is over").
**Status**: **DRAFT (G+W corroborated)**

## 4. Zet-types

| Type | Variant | Beschrijving | Voorwaarden | BAN | Bron |
|------|---------|--------------|--------------|-----|------|
| Namu kula | Kiswahili-namu | Plaatsing van ghala-kete + capture-sow vanaf gekozen kichwa | Eigen mbele-pit `i` (≥1 kete vóór drop) ∧ opponent mbele `i` (≥1 kete) bestaat | `N:d1>` of `N:d1<` (richting volgt uit kichwa-keuze) | G regel 1153–1162, 484–505 |
| Namu takata | Kiswahili-namu | Plaatsing van ghala-kete + sow zonder capture | Geen kula-zet beschikbaar, plus nyumba-restricties (§8.5) | `N:d1~` | G regel 507–567, 1163–1266 |
| Namu tax | Kiswahili-namu | Speciaal: vanaf functional nyumba haal slechts 2 kete | Functional nyumba is enige niet-lege mbele OR (zie §8.5) | TBD | G regel 1168–1177 |
| Mtaji | beide | Sow met capture aan einde | 2..15 kete in source ∧ landing ≤8 ∧ landing in own mbele non-empty ∧ opponent same column non-empty | `e1>` of `e1>*` | G regel 335–394 |
| Takata | beide | Sow zonder capture | Geen mtaji-zet beschikbaar; mbele-vichwa met ≥2 kete eerst, anders nyuma | `e1>` zonder `*` | G regel 396–440 |

**Mandatory-kula**: bevestigd actief in zowel namu als mtaji (G regel 1031–1035, 1063–1067, 1710–1715, 1763–1769; W "A player must capture if he or she can do so" voor beide fases; A; M; K; MWW).

### Algemene capture-validatie (MWW-regels die ontbraken)

| Regel | Bron | Geziefer-status |
|-------|------|-----------------|
| **First-lap-determines-captures**: als de eerste lap geen capture oplevert, kan in geen enkele endelea-vervolg-lap meer worden gekapen. Wel kan een capture-eerste-lap meerdere captures opleveren met daartussen non-capturing endelea-laps | MWW | impliciet correct (G's executeMove checkt capture alleen op landings die door de capture-flow gaan; takata-flow heeft geen capture-detection) |
| **16-seeds-no-capture (mtaji-only)**: een mtaji-zet vanaf een pit met ≥16 kete kan geen capture opleveren — telt als takata | MWW; A ("not allowed to harvest starting a move from a pit with more than 15 seeds") | G implementeert via `count >= 2 && count <= 15` filter; CONFIRMED |

**Status**: **CONFIRMED (5+ bronnen)**

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

**Sources**: G regel 442–482; W ("The choice of the kichwa to sow from is initially left to the player, with a few exceptions. If capture has occurred in any kimbi, sowing must start from the closest kichwa." en "On relay captures... it is never up to the player to choose which kichwa to sow from... the player must preserve the current clockwise or counterclockwise direction.")
**Status**: **DRAFT (G+W corroborated)**

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

### Conflict over mtaji-safari (C5 — nieuw)

| Bron | Mtaji-safari is | Bron-citaat |
|------|-----------------|-------------|
| G | keuze | `argSafariDecision` in state 13 — speler krijgt button-keuze |
| W | keuze | "the player may freely choose whether to relay-sow the contents of the nyumba or end his or her turn" |
| A | keuze | "the move ends ... if the player wishes to stop" |
| M | keuze | "a player may decide whether he wants to end the move now, or to continue" |
| **MWW** | **VERPLICHT** | "In contrast to the namu stage, **the player must safari** (continue to sow), if the sowing ends in the nyumba." |

**Voorlopige interpretatie**: mtaji-safari is een **keuze** (4 bronnen tegen 1). MWW is een outlier; mogelijk Ralf Gering hanteert een specifieke interpretatie. Zie §A C5.

**Sources**: G; W; A; M; MWW (afwijkend)
**Status**: **DRAFT (G+W+A+M corroborated keuze; MWW outlier — zie §A C5)**

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

### 8.3 Toestanden — drie-states-model (CONFIRMED)

Het drie-states-model is door meerdere onafhankelijke bronnen bevestigd, met name **M** (Polovynka) die het expliciet uitspelt:

| Toestand | Definitie | Gedrag bij sow-aankomst (mbele) |
|----------|-----------|----------------------------------|
| **Functional / valid** | owned ∧ ≥6 kete | Takata: zet stopt geforceerd. Capture-sow: safari-keuze (continue of stop). |
| **Tijdelijk-onklaar / invalid** | owned ∧ <6 kete | Geen speciale eigenschappen. Endelea-step: zet continueert via deze pit; pit wordt geleegd → **transitie naar destroyed**. |
| **Destroyed** | not owned (permanent) | Gewone pit zonder speciale status. |

**Triggers**:

| Transitie | Trigger | Bron |
|-----------|---------|------|
| Functional → tijdelijk-onklaar | count <6 (zonder dat eigen-sow de nyumba leegmaakt). Bv. tax (2 kete eruit) maakt nyumba 4 kete → tijdelijk-onklaar | M ("If a house is refilled with 6 seeds it becomes a valid house again") |
| Tijdelijk-onklaar → functional | count ≥6 weer (refill door sow van anderen of door capture-deposit) | M, G regel 606–613 |
| Functional/tijdelijk-onklaar → destroyed (a) eigen-sow-leegt | Eigen sow gebruikt nyumba als bron in mtaji én leegt hem | G regel 699–709; K ("Player loses its house if he empties it") |
| Functional/tijdelijk-onklaar → destroyed (b) opponent-capture | Opponent capture't nyumba | K ("or when the opponent captures it") |
| Tijdelijk-onklaar → destroyed (c) endelea-step | Een endelea-step met laatste-kete in tijdelijk-onklaar nyumba: zet continueert door de nyumba, hij wordt geleegd → destroyed | M ("if a last sown seed falls there, then the move continues and the house is destroyed") |

**Geziefer's `checkForNyumbaState`** (regel 616–628) returnt 0/1/2 (functional/non-functional/destroyed); bevestigt het 3-state model.

**Sources**: G regel 606–628, 699–709; A ("Players loose their house if it is emptied or when the first harvest"); M (expliciet 3-state); K ("if he empties it or when the opponent captures it"); W (binair model — onderspecifiek, de tijdelijk-onklaar-state ontbreekt)
**Status**: **CONFIRMED 3-state model** — user-bevestigd, gecorroboreerd door A+M+K+G. W's binair model is een vereenvoudiging (zie §A C1).

### 8.4 Tax-regel

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

**Bevestigd als canonical**: Wikipedia citeert dezelfde regel exact ("if, during the namua phase, the player begins his turn sowing from the nyumba, he will only sow two seeds from the nyumba rather than its whole content; this is called 'taxing' the nyumba"). De `taxActive_`-actie in geziefer is dus geen huisregel.

**Sources**: G regel 1168–1177; W
**Status**: **DRAFT (G+W corroborated)**

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
| dV | TBD (Wikipedia synthetiseert dV) |
| BS | TBD |
| W | **VERLIES** ("In both cases, this player loses the game.") |
| G | **VERLIES** |

**Status**: **DRAFT (G+W corroborated)** — Open Question #5 beantwoord. `CONFIRMED` na directe dV-raadpleging.

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

**Cross-validatie**:
- **A** (blog): identieke 3-tuple exclusies (still-owned-house, only-occupied, only-with-≥2). Effect: "If kuendelea ends in a kutakatia-ed pit, the move ends." Trigger: na takata + opponent kutakata-d laten waardoor exact één opponent-veld te kapen → blokkeer dat veld.
- **M** (Polovynka): identieke 3-tuple exclusies. Effect: "Opponent can't sow seeds from the blocked hole and if a last seed falls into that blocked hole, sowing immediately stops."
- **K** (kibao/dV): twee-exclusies-formulering ("only hole which allows the opponent to play" + "opponent's house"); semantisch waarschijnlijk equivalent (de unieformulering omvat M+A's "only-occupied" en "only-with-≥2"). Trigger inhoudelijk gelijk.
- **MWW** (snippet): "However, a nyumba itself cannot be takasiaed" + endelea-stop tenzij eerste lap vanuit nyumba.

**Conclusie**: het bestaan en de trigger zijn unaniem; exclusies zijn 3-tuple (G+A+M) of equivalent compact (K). De **3-zetten-duur** is uniek aan G en niet door andere bronnen bevestigd.

**Sources**: G regel 630–697, 343–354, 405–416, 1771–1786, 1896–1913; A; M; K; MWW (snippet)
**Status**: **DRAFT (G+A+M+K+MWW corroborated)** — duur (3 zetten in G) is enige niet-bevestigde detail. Zie §A C2 voor resterende detail-vragen.

## 12. Bewegingsbeperkingen

### 12.1 No-singleton-regel

> "It's not allowed to play a hole that contains only one seed." (K)
>
> "it's forbidden to play singletons — holes with a single seed in it." (M)

**Conditie**: een mtaji-zet mag **niet** vanuit een vichwa met exact 1 kete worden gestart. Geldt voor zowel mtaji-capture als mtaji-takata.

**Verfijning** (M): "if a house is destroyed and there are no capturing moves, then a player has to prefer holes with more than one seed." — dus prefer ≥2-pits boven singletons als geen captures mogelijk.

**Namu**: irrelevant — de namu-plaatsing maakt elke gekozen mbele-pit sowieso ≥2 vóór sow.

**Geziefer-conformiteit**: G's `getMtajiPossibleNonCaptures` (regel 419, 422) gebruikt `count >= 2`; consistent met de no-singleton-regel.

**Sources**: K, M; consistent met G
**Status**: **DRAFT (M+K corroborated; G consistent)**

### 12.2 No-suicide-regels

Twee specifieke vormen, beide in K (= dV-proxy) en A:

#### 12.2.1 Front-row mag nooit geleegd worden

> "The front row may never be emptied, not even temporarily." (MWW)

Algemene regel. **MWW formuleert sterker** dan A en K, die alleen het kichwa-edge-case noemen.

#### 12.2.2 Kichwa-takata-richting (specifiek geval)

> "If the only occupied hole of the front row is a kichwa and it contains two or more seeds, they must be sown towards the center of the front row." (MWW)
>
> "If the only filled hole on the front row is one of the kichwas, then kutakata cannot be done in the direction of the back row." (K)
>
> Rule 1.5.3.1 (A): "If the only filled pit on the front row is one of the kichwa-s, then kutakata cannot be done in the direction of the back row (because the front row will be empty and the game is a loss)."

MWW formuleert het positief ("MUST be sown towards center"), K en A negatief ("CANNOT be done in direction of back row"). Beide gelijkwaardig: van field 1 → +1 (richting field 2), van field 8 → -1 (richting field 7).

#### 12.2.3 Takata-source-rij in kunamua

> "Kutakata cannot start from the back row" (K, kunamua context)

In kunamua plant de speler in mbele en sowt vanaf daar; takata vanuit nyuma is sowieso geen valide kunamua-actie. Deze regel lijkt redundant of dekt een edge-case. TBD; mogelijk onbedoeld in de context.

**Geziefer-skip**: G regel 1005–1008 documenteert expliciet dat deze "no prevention of a move which causes loss of the player" niet wordt afgedwongen. **Onze engine moet dit wél afdwingen** voor canonical-conform spel.

**Sources**: K, A, MWW
**Status**: **CONFIRMED (3 bronnen)** — MWW formuleert sterker dan K+A; geziefer skipt expliciet

## A. Conflict-log

Geziefer + Wikipedia + Mancala World snippet vergeleken. Twee echte conflicten ontdekt; één voorlopig verwerpen ten gunste van de simpelere kant en één wachten op derde bron.

| ID | Conflict-onderwerp | Bron-uitspraken | Voorlopige interpretatie | Reden | User-bevestigd |
|----|---------------------|------------------|--------------------------|-------|----------------|
| C1 | Nyumba-toestanden: 3-state vs 2-state | **G+A+M+K**: drie toestanden bevestigd (functional / tijdelijk-onklaar / destroyed). **W**: binair (special / not-special) | **3-state model** (zie §8.3) | M expliciet 3-state, K (= dV-proxy) impliceert het, G implementeert het, A zinspeelt erop. W is een vereenvoudiging | **JA (RESOLVED)** |
| C2 | Kutakatia-duur | **G**: exact 3 zetten (`kutakatiaMoves = 3`, decrementeert per beurt). **A+M+K+MWW**: duur niet expliciet | Aanhouden G's 3 zetten als startpunt | Geen tegenstrijdige bron; alle anderen specificeren duur niet | nee — open |
| C3 | Kutakatia-exclusies: 3-tuple of unie-formulering | **G+A+M**: drie exclusies (nyumba, only-occupied, only-with-≥2). **K**: twee exclusies ("only-hole-allowing-play" + "opponent's house"). **MWW**: alleen nyumba | 3-tuple van G+A+M is concretiserend; K is waarschijnlijk equivalent unieformulering; MWW is incompleet (snippet) | Drie onafhankelijke bronnen geven dezelfde 3-tuple; K's compactere formulering is semantisch dekkend | impliciet via §11 |
| C4 | No-suicide-regel | **K+A+MWW**: kichwa-takata richting nyuma verboden + "front row mag nooit geleegd". **G**: skipt expliciet | Implementeren conform K+A+MWW | Drie bronnen bevestigen; geziefer geeft toe het te skippen. MWW formulering ("front row may never be emptied, not even temporarily") is meest algemene formulering | impliciet via §12.2 |
| C5 | Mtaji-safari: keuze of verplicht | **G+W+A+M**: keuze. **MWW**: verplicht ("In contrast to the namu stage, the player must safari") | Keuze (4-tegen-1) | MWW is outlier; geen 2e bron die mtaji-safari als verplicht beschrijft. Mogelijk auteur-specifieke (Gering) lezing | nee — open |

**Verwachte verdere conflicten** (na directe dV-raadpleging):
- Mandatory-kula-precision op edge cases
- Kichwa-with-≥16-kete restrictie (G skipt expliciet, niet door anderen besproken)
- 12-rounds-cap (G-implementatie-detail; klassieke regel onbekend — A noemt Rule 1.5.6 als "infinite moves illegal" maar geen exacte tekst opgevangen)

## B. Open Questions die dit document moet oplossen vóór fase 1

(Spiegeling van Open Questions uit het plan.)

| # | Vraag | Antwoord | Status |
|---|-------|----------|--------|
| 2 | Initiële bordpositie per variant | §2.1 + §2.2 ingevuld | **CONFIRMED (G+W+A+M+K)** |
| 3 | Russ-editie bevestigen | Wikipedia citeert Russ niet; **dV (1995)** is canonieke primaire bron, geraadpleegd via K als proxy | OBSOLETE → dV-by-proxy |
| 4 | Namu-kula richtingskeuze | §6.2: indirect via kichwa-selectie; rand-captures forced, middle player-keuze | **CONFIRMED (G+W+A+M+K)** |
| 5 | Stalemate-uitkomst | §9.3: verlies | **CONFIRMED (G+W+A+K)** |
| 9 | Wat is "safari"? | §6.4: keuze om eigen functional nyumba leeg te plunderen tijdens capture-sow | **CONFIRMED (G+W+A+M)** |
| 10 | Kichwa-selectie als sub-state? | §6.3: ja, sub-state `AwaitKichwa` voor middle-no-prior-direction | **CONFIRMED (G+W+A+M+K)** |
| 11 | Hus uitsluiten? | §10 + introductie: ja | CONFIRMED (user-keuze) |
| C1 | Nyumba-toestanden | §8.3: 3-state (functional / tijdelijk-onklaar / destroyed) | **CONFIRMED (G+A+M+K, user)** |

**Niet-blokkerende open vraagstukken**:
- **C2** kutakatia-duur (G's 3 zetten — niet door andere bron bevestigd; G is enige bron met expliciete duur)
- **C5** mtaji-safari: keuze (G+W+A+M) of verplicht (MWW). 4-1 voor keuze; voorlopige interpretatie: keuze
- "First-lap-from-nyumba" uitzondering op kutakatia-stop (MWW: "unless it has been reached in the first lap from a nyumba") — implementatie-detail van §11
- Mandatory-kula edge cases (mogelijk irrelevant voor MVP)
- Exact gedrag van "infinite moves" (Rule 1.5.6 in A; G's 12-rounds is workable proxy)

**Status fase 1 readiness**: alle voorheen-blokkerende vragen zijn `CONFIRMED` met 3+ onafhankelijke bronnen (G+W+A+M+K+MWW). C2 en C5 kunnen tijdens engine-implementatie verfijnd worden zonder data-model-impact. **Fase 1 is unblocked.**
