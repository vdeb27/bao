# kibao.org — `cs_kanuni.php` (Bao rules)

> Bron: [`kibao.org/cs_kanuni.php?lng=en`](https://kibao.org/cs_kanuni.php?lng=en) — bezocht via WebFetch op 2026-05-08.
>
> **Status**: ZEER WAARDEVOLLE bron. Bevat een directe attributie:
>
> > "The above rules are adapted from: **Alexander J. de Voogt (1995). Limits of the Mind. Towards a characterisation of Bao mastership. Ph.D. thesis, University of Leiden, The Netherlands** and from manuscripts of the 'Dar es Salaam Regional Traditional Games Association.'"
>
> Dit maakt kibao.org de beste beschikbare proxy voor dV's thesis (waarvan de PDF achter paywalls zit op ResearchGate en Academia.edu). De Tanzaniaanse Dar-es-Salaam manuscripten als secundair-aangevoerd geeft extra Bao-traditie-grond.

## 1. Initiële opstelling

> "At the start of the game each player has 22 seeds in store, 6 seeds in the nyumba and 2 seeds in the two consecutive holes to the right of the nyumba."

**South begint**. **Bevestigt G+W+A+M op alle punten.**

## 2. Wincondities

> "The winner is the player who firstly empties the front row of the opponent or makes it impossible for the opponent to move."

Bevestigt hamna én stalemate als verlies-condities.

## 3. Faseovergang

> "The mtaji (capital) stage starts if all 64 seeds are on the board and the stores are empty."

Bevestigt G en W: beide stores leeg = transitie. **Geheel** 64 op het bord is implicaite (komt automatisch uit beide stores leeg).

## 4. Capture-condities

**Kunamua**:
> "Capturing is only allowed if a sowing ends in a non-empty hole at the front row that has an opposing non-empty hole at the front row of the opponent (this is called mtaji, capital)."

**Mtaji**:
> "After spreading in a chosen direction, the last seed must allow capturing (capture is compulsory)."

> "Capture is always compulsory."

Bevestigt mandatory-kula in beide fases.

## 5. Kichwa-selectie

**Kunamua**:
> "If the capturing hole is not a kichwa or kimbi, the player may choose which kichwa to sow from."

**Mtaji**:
> "The sowing of the captured seeds from one of the four central holes starts from the right kichwa if the selected move direction was anti-clockwise, starts from the left kichwa if the selected move direction was clockwise."

Bevestigt geziefer's regel: forced bij kimbi (incl. kichwa), forced bij vorige direction in mtaji, keuze bij middle-no-prior-direction.

## 6. Nyumba-toestanden — 3 states bevestigd

**Tijdelijk-onklaar (kunamua)**:
> "If the player still owns the nyumba and a sowing ends in the nyumba then the move ends during takata."

**Permanent destroyed**:
> "Player loses its house if he empties it or when the opponent captures it."

Twee triggers voor destroyed:
1. Speler leegt zelf de nyumba in eigen sow
2. Opponent capture't de nyumba

**Bevestigt 3-state model**: functional (≥6 én owned) → tijdelijk-onklaar (<6 én owned) → destroyed (not owned). Niet expliciet, maar de combinatie van "owned" en "≥6" geeft drie zinnige toestanden.

## 7. Endelea (kuendelea)

> "If a sowing ends in a non-empty hole and a capture is not allowed, then the move continues in the same direction by taking all the seeds from that hole and sowing the seeds starting at the next hole in the same direction."

Bevestigt onze §7 (G+A).

## 8. Kutakatia

> "If a player must kutakata, but he can play such that: the opponent also must kutakata next move and exactly one of the opponent's hole can be captured after that, then the opponent is not allowed to empty this hole under kutakatia."

**Exclusies**:
> Cannot apply if that hole is "the only hole which allows the opponent to play" or is "the opponent's house."

**Slechts 2 exclusies hier** (vs. 3 in blog A en geziefer G). Het verschil: kibao formuleert "only hole which allows the opponent to play" als één omvattende exclusie, waar A en G drie subgevallen onderscheiden ("only occupied", "only with ≥2", "owned house"). Mogelijk semantisch equivalent (omvattende exclusie = unie van de subgevallen).

## 9. No-suicide-regels

> "Kutakata cannot start from the back row" (Kunamua context).

> "If the only filled hole on the front row is one of the kichwas, then kutakata cannot be done in the direction of the back row."

Twee specifieke no-suicide-regels:
- Tijdens kunamua mag een takata niet vanuit nyuma starten — onduidelijk relevant want namu plant in mbele.
- Een kichwa als enige niet-lege mbele-pit mag geen takata richting nyuma — verlies-preventie.

**Geziefer skipt deze expliciet** (line 1006). Voor de canonical engine: implementeren.

## 10. No-singleton regel

> "It's not allowed to play a hole that contains only one seed."

Een vichwa met exact 1 kete mag niet als sow-source gebruikt worden. Geldt voor mtaji-takata.

In namu is dit irrelevant: planten van een ghala-kete maakt de bron altijd ≥ 2 vóór de sow.

In geziefer's `getMtajiPossibleNonCaptures` (regel 419, 422) staat `count >= 2`, dus geziefer is consistent. Geziefer's `getKunamuaPossibleNonCaptures` (regel 528–537) staat singletons toe als laatste resort, maar dat gaat over het kiezen van het *plantings*-veld, niet de sow-source — dus eveneens consistent.

## 11. Notatie-symbolen

> Special holes include: **&** = kichwa (head); **%** = kimbi (side).

Niet gebruikt in onze BAN-spec (die gebruikt `>` en `<` voor richting).

## 12. Bao la Kujifunza

**Niet behandeld in dit document.**

## 13. Beoordeling

Kibao.org is dichtst-bij-dV. De regels matchen (en specificeren strikter dan) wat we uit Wikipedia en blog kennen. Op specifieke punten:
- Bevestigt 3-state nyumba (zonder ze als zodanig te benoemen)
- Geeft expliciete no-suicide en no-singleton regels — beide door geziefer expliciet over-geslagen
- Heeft compactere kutakatia-exclusie-formulering
