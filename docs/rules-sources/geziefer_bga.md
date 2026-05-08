# Geziefer BGA-implementatie als regel-oracle

> Bron: [`geziefer/baolakiswahili`](https://github.com/geziefer/baolakiswahili) — auteur Alexander Rühl (`alex@geziefer.de`). PHP-implementatie voor BoardGameArena. Centrale bestand: `baolakiswahili/baolakiswahili.game.php`, ~2100 regels. Alle regelverwijzingen hieronder zijn naar dit bestand tenzij anders vermeld.
>
> **Status**: dit document bevat de regelinterpretatie van geziefer; het is **één** bron en kan afwijken van canonieke literatuur. Alle uitspraken hieronder moeten als `(geziefer-only — needs corroboration)` worden behandeld in `RULES.md` totdat tegen Russ/de Voogt/Bao Society gevalideerd.

## 1. Field-encoding (kritiek)

Per-speler coördinaatsysteem. Het bord wordt voor speler 2 180° gespiegeld weergegeven, dus elke speler ziet zijn eigen field 1 linksonder.

```
geziefer comment, regels 21–27:
// 16 15 14 13 12 11 10 09 (opponent's 2nd row)
// 01 02 03 04 05 06 07 08 (opponent's 1st row)
// -----------------------
// 01 02 03 04 05 06 07 08 (player's 1st row)
// 16 15 14 13 12 11 10 09 (player's 2nd row)
```

| Field | Betekenis |
|-------|-----------|
| 0 | Ghala (storage / reservoir voor namu-kete) |
| 1–8 | Mbele (front row), van links→rechts vanuit eigen perspectief |
| 9–16 | Nyuma (back row), van rechts→links vanuit eigen perspectief — d.w.z. field 9 = direct boven field 8, field 16 = direct boven field 1 |

`getNextField` (regel 582–590) implementeert sow-topologie als simpele `+1`/`-1` met wrap (`0→16`, `17→1`):
```php
function getNextField($field, $direction)
{
    $destinationField = $field + $direction;
    $destinationField = ($destinationField == 0) ? 16 : $destinationField;
    $destinationField = ($destinationField == 17) ? 1 : $destinationField;
    return $destinationField;
}
```

**Implicatie voor onze TS/Rust-engine**: een lineaire 1..16 ringbuffer voldoet; geen aparte rij/kolom-omkering. De fysieke "kichwa-direction-reversal" emerges uit de wrap (`field 1` clockwise naar `field 16` van eigen nyuma; `field 8` counter-clockwise naar `field 9`). Geen aparte `next_pit`-regel voor kichwa nodig zoals in ons plan §2.4 verondersteld.

## 2. Initiële bordpositie

`setupNewGame` (regel 72–209). Variant-afhankelijk via `OPTION_VARIANT`.

### 2.1 Bao la Kiswahili (regel 112–134)

```php
// player 1 (South):
fields 1..4 = 0
field 5      = 6   // nyumba
field 6      = 2
field 7      = 2
fields 8..16 = 0
field 0      = 22  // ghala

// player 2 (North):
field 1      = 0
field 2      = 2
field 3      = 2
field 4      = 6   // nyumba
fields 5..16 = 0
field 0      = 22  // ghala
```

Totaal per speler: 6 + 2 + 2 + 22 = 32 (64 totaal).

**Nyumba-positie** (regel 308–314):
```php
function getNyumba($player_id) {
    return $this->isSouth($player_id) ? 5 : 4;
}
```
Field 5 voor South, field 4 voor North. **Mirroring**: niet beide field 5; per-speler-spiegelen.

### 2.2 Bao la Kujifunza (regel 135–145)

Alle 16 vichwa krijgen 2 kete; ghala = 0. Geen namu-fase (geen reserveseeds). Begint direct in mtaji-state 20.

```php
for ($i = 1; $i <= 16; $i++) {
    $values[] = "('$player1', '$i', '2')";
    $values[] = "('$player2', '$i', '2')";
}
$values[] = "('$player1', '0', '0')";
$values[] = "('$player2', '0', '0')";
```

Totaal per speler: 32 (64 totaal). Geen nyumba-mechaniek (zie §8 — alle nyumba-checks zijn conditional op `VARIANT_KISWAHILI`).

### 2.3 Beginspeler

BGA bepaalt automatisch wie begint (regel 95: "BGA determines start player"). Geen specifieke beperkingen op de eerste zet in deze code.

## 3. Mandatory kula

Bevestigd voor zowel namu als mtaji.

`argKunamuaMoveSelection` (regel 1701–1743):
```php
// assume capture move 
$capture = true;
$result = $this->getKunamuaPossibleCaptures($player);
if (empty($result)) {
    $capture = false;
    $result = $this->getKunamuaPossibleNonCaptures($player);
    ...
}
```

`argMtajiMoveSelection` (regel 1758–1788) volgt hetzelfde patroon. Een speler krijgt non-capture-zetten alléén als er geen capture-zetten zijn.

## 4. Capture-condities

### 4.1 Mtaji-capture (`getMtajiPossibleCaptures`, regel 335–394)

Voor elk eigen vichwa `i` met 2..15 kete, in beide richtingen, bereken landing field via `getDestinationField(i, dir, count)` (regel 571–579 — pure herhaling van `getNextField`). Capture is mogelijk als:
1. Landing field ≤ 8 (in mbele)
2. Eigen mbele bij landing has > 0 kete (zal ≥ 2 worden na drop — endelea-trigger)
3. Opponent mbele in zelfde kolom heeft > 0 kete

```php
$destinationField = $this->getDestinationField($i, -1, $count);
if ($destinationField <= 8 && $board[$player_id][$destinationField]["count"] > 0 &&
    $board[$opponent][$destinationField]["count"] > 0) {
    ...
}
```

**Conditie #2 is significant**: capture vereist dat de eigen landing-pit *al niet-leeg* was vóór de drop. Een sow die in eigen lege mbele eindigt is een eindigende takata, niet een capture.

### 4.2 Namu-capture (`getKunamuaPossibleCaptures`, regel 484–505)

```php
for ($i = 1; $i<= 8; $i++) {
    $countPlayer = $board[$player_id][$i]["count"];
    $countopponent = $board[$opponent][$i]["count"];
    if ($countPlayer >= 1 && $countopponent >= 1) {
        $result[$i] = array(0);
    }
}
```

Conditie: **eigen mbele-pit `i` moet vóór de namu-drop al ≥1 kete bevatten** AND opponent mbele-pit `i` heeft ≥1 kete. De namu-drop zelf voegt nog een kete toe; de capture-trigger is dan analoog aan mtaji (landing in non-empty own mbele met opposite stones).

`array(0)` als richting betekent: richting is nog niet vastgelegd; wordt bepaald door kichwa-selectie (zie §5).

### 4.3 Capture-uitvoering

Mtaji-capture (regel 1290–1302): kete worden gedistribueerd in de gekozen richting tot landing; daarna `stateAfterMove = 'continueCapture'` en `captureField` wordt opgeslagen — geen verdere sow yet, eerst kichwa-selectie.

Namu-capture (regel 1153–1162): de namu-kete uit ghala wordt geplaatst, daarna direct `stateAfterMove = 'continueCapture'`. **Geen sowing in deze stap** — de speler kiest eerst kichwa.

## 5. Kichwa-selectie (state 12)

`getPossibleKichwas` (regel 442–482) bepaalt welke kichwa-pits beschikbaar zijn. Logica:

| Conditie | Beschikbare kichwa(s) |
|----------|----------------------|
| `captureField <= 2` (capture in left kichwa/kimbi) | LEFT (field 1) |
| `captureField >= 7` (capture in right kichwa/kimbi) | RIGHT (field 8) |
| `moveDirection == +1` AND `captureField < 7` | LEFT |
| `moveDirection == -1` AND `captureField > 2` | RIGHT |
| `moveDirection == 0` AND `captureField in {3..6}` | BOTH (player chooses) |

Code:
```php
if ($captureField <= 2 || ($moveDirection == 1 && $captureField < 7)) {
    $result[1] = array(2, $opponent.'_'.$captureField);
} elseif ($captureField >= 7 || ($moveDirection == -1 && $captureField > 2)) {
    $result[8] = array(7, $opponent.'_'.$captureField);
} elseif ($moveDirection == 0 && $captureField > 2 && $captureField < 7) {
    $result[1] = array(2, $opponent.'_'.$captureField);
    $result[8] = array(7, $opponent.'_'.$captureField);
}
```

**Kichwa-selectie is dus deterministisch behalve in middle-capture-with-no-prior-direction** (de namu-kula edge case). In alle andere gevallen kan het in de Move-struct gepacked worden zonder een sub-state.

**Implicatie voor onze Move-enum**: voor namu-kula-with-direction-undecided is een sub-state nodig OF een tweede actie binnen dezelfde zet. Voor mtaji-kula is de richting al gekozen bij move-selection, dus kichwa-keuze is triviaal afleidbaar.

### 5.1 Capture-sow uitvoering (regel 1429–1530)

Na kichwa-selectie:
```php
// start with emptying opponent's bowl
$count = $captureCount;  // gestolen kete
// the move takes captured stones and starts with kichwa,
// as every move always goes to next field, we go back one field (inverted direction) 
// for start of move (sourceField) in order to have same behaviour later as for regular moves
$sourceField = $this->getNextField($sourceField, $moveDirection * (-1));
```

Daarna een `do { ... } while ($count > 1)` loop die door blijft sowen, met deze terminatie-condities:
1. **Hamna mid-sow**: `$scoreopponent == 0` → break. (regel 1494–1497)
2. **Volgende capture**: landing in mbele (`$sourceField <= 8`) met opponent stones same column → `stateAfterMove = 'continueCapture'`, break. (regel 1505–1511)
3. **Functional nyumba**: landing in eigen functional nyumba → `stateAfterMove = 'decideSafari'`, break. (regel 1514–1522)
4. **Empty landing**: count na drop = 1 → loop eindigt natuurlijk.

## 6. Safari (state 13)

`argSafariDecision` (regel 1745–1756):
```php
// no moves currently possible, but put nyumba in possible moves to allow for highlighting, 
// button selection will be presented to decide for continuing or stopping
```

Trigger: zie §5.1 punt 3 — wanneer een capture-sow uitkomt in de eigen functional nyumba (≥6 kete) wordt de speler gestopt en gevraagd om te beslissen.

Beslissing (regel 1045–1058):
- `direction == 0` → STOP. State wordt `nextPlayer`, zet eindigt; nyumba blijft intact.
- `direction == 1` → DOORGAAN ("go on safari"). Empty nyumba, mark als destroyed, sow continues.

Uitvoering bij safari = ja (regel 1408–1428):
```php
$sourceField = $this->getNyumba($player);
$count = $board[$player][$sourceField]["count"];
$board[$player][$sourceField]["count"] = 0;
$this->checkAndMarkDestroyedNyumba($player, $sourceField);
// ... continues into the same do-while sow loop
```

**Conclusie over safari**: safari = de keuze om je eigen functional nyumba leeg te plunderen om een capture-sow voort te zetten. Het is een speler-keuze, geen verplichting. Vereist een sub-state in de Phase-FSM en een `SafariChoice(bool)` actie in onze Move-enum.

## 7. Endelea (relay sowing)

In takata (regel 1208–1264 voor Kiswahili-namu, regel 1322–1396 voor Kujifunza/Kiswahili-2nd):

```php
while ($count > 1) {
    while ($count > 0) {
        $destinationField = $this->getNextField($sourceField, $moveDirection);
        if ($destinationField == $startField) {
            $rounds += 1;
        }
        $board[$player][$destinationField]["count"] += 1;
        $sourceField = $destinationField;
        $count -= 1;
    }
    if ($rounds >= 12) { /* zombie */ break; }
    $count = $board[$player][$sourceField]["count"];
    if ($count > 1) {
        // empty own bowl for next move (with nyumba-protection)
        $board[$player][$sourceField]["count"] = 0;
        ...
    }
}
```

Conditie: takata-sow gaat door als landing-pit `count > 1` na drop (i.e. niet-leeg vóór drop). Termination: empty landing (`count = 1` na drop) → loop eindigt.

**Hard cap = 12 full rounds van het bord** (regel 1235, 1353): bij ≥12 rondjes wordt de speler "zombie" — verklaart automatisch verloren. Dit is geen klassieke regel maar een implementatie-safeguard. Onze 256-hop cap in plan §2.4 is strakker (256 stappen ≈ 16 rondjes); we kunnen die houden als sanity-check.

**Belangrijk**: bij capture-sow geldt de endelea-loop óók (regel 1481–1530), maar daar checkt de inner loop óók op `continueCapture` (volgende capture mogelijk) en `decideSafari` (eigen functional nyumba) als terminerende condities.

## 8. Nyumba-mechaniek

### 8.1 Toestanden

Drie waarden van `checkForNyumbaState` (regel 616–628):
- `0` = **functional**: nog in bezit AND ≥6 kete
- `1` = **non-functional**: nog in bezit AND <6 kete (tijdelijk leeg geraakt of nooit ≥6, maar niet leeggehaald uit eigen sow)
- `2` = **destroyed**: niet meer in bezit (eigen sow heeft hem leeggemaakt)

KV-store flags: `nyumba5functional` voor player 1, `nyumba4functional` voor player 2 (boolean: in bezit ja/nee). De `functional`-status zelf wordt afgeleid uit possession + count (regel 606–613).

### 8.2 Initialisatie (Kiswahili-only)

```php
// regel 187–190
INSERT INTO kvstore VALUES ('nyumba5functional', true)  // player 1 owns
INSERT INTO kvstore VALUES ('nyumba4functional', true)  // player 2 owns
```

### 8.3 Vernietiging

`checkAndMarkDestroyedNyumba` (regel 699–709): wanneer een eigen sow het nyumba-veld emptyt, wordt de KV-flag op `false` gezet. Wordt aangeroepen op meerdere plekken — wanneer de eigen kete-source-pit nyumba is en ge-emptyd wordt voor een nieuwe sow-lap.

### 8.4 Tax-regel (uniek aan deze implementatie)

**Belangrijke afwijking van klassieke regels**: in deze code wordt namu-takata vanaf eigen functional nyumba *niet* een gewone sow van alle kete, maar een "tax" — alleen 2 kete worden uit nyumba gehaald (regel 1168–1177):

```php
if ($sourceField == $nyumba && $wasNyumbaFunctional) {
    $count = 2;
    $board[$player][$sourceField]["count"] -= $count;
    array_push($moves, "taxActive_" . $sourceField);
    ...
}
```

Daarna gaat de gewone sow-loop met deze 2 kete door. **Dit is een geziefer-specifieke regel** — bevestigen of dit canon is, of een huisregel.

### 8.5 Kunamua non-capture restricties

`getKunamuaPossibleNonCaptures` (regel 507–567) heeft drie takken op basis van nyumba-status:

1. **Geen possession** (regel 519–537): kies een mbele-pit met ≥2 kete; alleen als geen ≥2 bestaat, mogen ≥1-pits.
2. **Functional nyumba** (regel 539–554): kies elk niet-leeg mbele-veld dat NIET de nyumba is; alleen als geen ander gevuld mbele-veld bestaat, mag de nyumba zelf gebruikt worden.
3. **Non-functional nyumba** (regel 555–565): kies elk niet-leeg mbele-veld zonder restrictie.

Dit zijn allemaal namu-takata-restricties (alleen relevant als geen kula-zet bestaat).

## 9. Wincondities

`getScore` (regel 843–894). Score = totaal aantal kete als de speler nog kan bewegen én mbele niet-leeg, anders 0.

**`canMove`-check verschilt per variant**:
- Kiswahili 1st phase (namu): tenminste 1 kete in mbele
- Kiswahili 2nd phase (mtaji) of Kujifunza of Hus: tenminste 1 vichwa met ≥2 kete (in beide rijen)

**`isEmpty`-check** (regel 877–886): mbele-leeg = verloren. Dit is hamna.

`updateScores` (regel 898–940) returnt `true` als score van enige speler 0 is — eindigt het spel.

| Conditie | Resultaat in code | Klassieke term |
|----------|-------------------|----------------|
| Mbele leeg na opponent's zet | score = 0 → verlies | hamna |
| Geen vichwa met ≥2 kete (mtaji) | score = 0 → verlies | (stalemate als verlies) |
| Mbele leeg in 1st phase | score = 0 → verlies | mkononi |
| 12-rounds-cyclus in sow | zombie → verlies | (geen klassieke term) |

**Stalemate-behandeling (Open Question #5)**: in deze implementatie is stalemate altijd **verlies**, nooit remise.

## 10. Faseovergang namu→mtaji (Kiswahili)

`stNextPlayer` (regel 1855–1921):
```php
if ($this->getVariant() == VARIANT_KISWAHILI && 
    $board[$playerLast][0]["count"] == 0 && $board[$playerNext][0]["count"] == 0) {
    // store new phase and switch to it
    $sql = "UPDATE kvstore SET value_text = '2nd' WHERE `key` = 'phase'";
    ...
}
```

**Conditie**: BEIDE ghalas leeg. Niet "één ghala leeg" of een ply-cap.

Na overgang wordt `getScore` opnieuw uitgevoerd: een speler kan in 1st phase nog kunnen spelen (mbele leeg is OK in namu zolang er ghala-kete zijn) maar in 2nd phase verliezen door mbele-leeg.

## 11. Kutakatia (nieuw, niet in CLAUDE.md)

Een Kiswahili 2nd-phase-only mechanisme (regel 630–697). Werkt zo:

1. Na een takata-zet wordt gecheckt of opponent's volgende beurt **exact één** mogelijke capture toestaat (regel 671–674 zegt `count($possibleCapturedFields) == 1 && count($possibleOpponentsCaptures) == 0`).
2. Als opponent's enig-mogelijke-te-kapen-veld **niet** is:
   - Hun functional nyumba
   - Hun enige niet-lege mbele-veld
   - Hun enige mbele-veld met ≥2 kete
3. Dan wordt dat veld voor 3 zetten (één van elke speler ≈ "next 2 player changes plus current") **geblokkeerd**. De geblokkeerde speler mag het niet legen; de blokkerende speler MOET het kapen wanneer mogelijk.

`kutakatiaMoves` is een teller die elke beurt afneemt; bij 0 wordt `blockedField` en `blockedPlayer` gereset.

In de notation krijgt zo'n zet een `*`. **Conclusie over kutakatia**: dit is een geavanceerde regel die in andere bronnen ook bekend kan zijn (de term is Swahili). Toevoegen aan CLAUDE.md-glossary; bevestigen of canonical.

## 12. Variant-verschillen samengevat

| Element | Kiswahili (1st) | Kiswahili (2nd) | Kujifunza | Hus |
|---------|-----------------|------------------|-----------|-----|
| Begin | namu (1st phase) | mtaji-fase | direct mtaji | eigen variant |
| Ghala | start 22, leegt | n.v.t. | start 0 | start 0 |
| Initial board | nyumba 6 + 2,2 | (na phase-switch) | alle 2 | alle 2 |
| Nyumba | bestaat, ≥6 = functional | id. + tax/safari | n.v.t. | n.v.t. |
| Tax-regel | actief op functional nyumba | n.v.t. | n.v.t. | n.v.t. |
| Safari | n.v.t. tijdens namu-takata | trigger op functional nyumba mid-sow | n.v.t. | n.v.t. |
| Kutakatia | n.v.t. | actief | n.v.t. | n.v.t. |
| Mandatory kula | actief | actief | actief | n.v.t. (hus heeft eigen regels) |
| canMove-check | ≥1 kete in mbele | ≥2 kete in elk vichwa | ≥2 kete in elk vichwa | ≥2 kete in elk vichwa |

## 13. Geziefer-specifieke afwijkingen / edge cases

Lines 1005–1008 vermelden expliciet *deliberately ignored official rules*:
1. Geen preventie van een zet die direct leidt tot verlies van de speler.
2. Geen check op de regel "kichwa-naar-buitenste-rij is enige gevulde bowl en bevat 16+ kete".
3. Geen check op kutakatia-blocked bowl in een latere harvest-stap.

**Dit zijn signalen dat geziefer's regels niet 1-op-1 canonical zijn**; we moeten andere bronnen raadplegen voor (1) een "no suicide"-regel, (2) iets over kichwa met ≥16 kete, (3) kutakatia-strictness.

Daarnaast is **de tax-regel op functional nyumba** (§8.4) verdacht — niet zeker of klassiek of geziefer-eigen.

## 14. Concluderende observaties voor RULES.md

1. **Initial position is concreet**: §2.1 en §2.2 kunnen direct als DRAFT in RULES.md.
2. **Mandatory kula in zowel namu als mtaji is bevestigd**.
3. **Field encoding** als 0 (ghala), 1–8 (mbele), 9–16 (nyuma met kolom-omkering) is een werkbare conventie; we kunnen dezelfde aanhouden of vertalen naar (player, row, col).
4. **Safari is uitgelegd**: niet een namu-mechanisme zoals onze Open Question #9 suggereerde, maar een mtaji/capture-mechanisme.
5. **Kichwa-selectie is in 2 van 5 condities deterministisch**, in 1 conditie player-keuze; dit pleit voor een **sub-state** in `Phase`-FSM, niet een Move-veld.
6. **Stalemate = verlies** (Open Question #5 voorlopig beantwoord met geziefer als bron).
7. **Endelea hard-cap** (12 rounds) is geziefer's safeguard; onze 256-hop is een redelijke equivalent.
8. **Kutakatia** is een regel die we moeten toevoegen aan CLAUDE.md-glossary en RULES.md.
9. **Tax-regel** moet als geziefer-specifiek worden geflagd totdat tegenbron gecheckt.
10. **Hus** blijft out of scope (Open Question #11 bevestigd: ja, uitsluiten).

Wat geziefer NIET dekt en we elders moeten zoeken:
- Russ-editie identiteit (Open Question #3).
- "No suicide"-regel (officieel, maar geziefer skipt).
- Welke kichwa bij ≥16 kete (officieel, maar geziefer skipt).
- Bevestiging tax-regel.
- Bevestiging kutakatia-precieze-regels.
