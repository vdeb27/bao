# abstractstrategygames blog — "Bao: sum-up of rules"

> Bron: [`abstractstrategygames.blogspot.com/2011/01/bao-sum-up-of-rules.html`](http://abstractstrategygames.blogspot.com/2011/01/bao-sum-up-of-rules.html) — bezocht via WebFetch op 2026-05-08.
>
> **Status**: derde onafhankelijke bron met **genummerde regels** (Rule 1.4.4, Rule 1.5.3.1, Rule 4.1, etc.). Auteur niet uit fetch-resultaat te halen; lijkt een geconsolideerde regelset te zijn (mogelijk gebaseerd op de Voogt). Blog uit 2011.

## 1. Initiële opstelling (kunamua)

> "Six seeds in South's nyumba and two seeds in the pit to the right of the nyumba and again two seeds in the next pit to the right. The same number of seeds are in North's nyumba and in the consecutive pits to the right."

Bevestigt G en W: 6 in nyumba, 2 in elk van twee buurkolommen, 22 in reserve.

## 2. Wincondities

> "The goal of Bao la Kiswahili is to empty the front row of the opponent or to make it impossible for the opponent to move."
>
> Game ends when: "(1) the front row of a player is empty (even before his/her move ends) or (2) a player cannot move."

**Bevestigt** G+W: hamna én stalemate beide → verlies, geen remise.

## 3. Capture-condities en mandatory kula

> "Harvesting only allowed if a sowing ends in a non-empty pit... at the front row that has an opposing non-empty pit at the front row of the opponent."
>
> "If a player can harvest, he must do so."

Bevestigt mandatory-kula in beide fases.

## 4. Kichwa-selectie

> "If the harvesting pit is not a kichwa or kimbi, the player may choose which kichwa to sow from."
>
> **Rule 1.4.4**: "If the sowing starts at the left kichwa, the sowing direction is clockwise, if the sowing starts at the right kichwa, the sowing direction is anti-clockwise."

Bevestigt geziefer's logic én Wikipedia: keuze als capture-veld in middle (cols 3..6); forced als capture in kimbi (cols 1, 2, 7, 8).

## 5. Mtaji-fase regels

> "A move in mtaji stage must start from a pit on the front row or back row that contains more than one seed, whose seeds allow a harvest."
>
> "It's not allowed to harvest starting a move from a pit with more than 15 seeds, even if it is the nyumba."

**Belangrijk**: bevestigt geziefer's `2 <= count <= 15` check (G regel 360). De ≥16-kete-restrictie die geziefer expliciet skipt (line 1007–1008) is hier dus **canonical**.

## 6. Kimbi vs kichwa — exacte definitie

Uit Wikipedia / gambiter (parallel): **kimbi includes kichwa als bovenset**.

> "The first and last pit of the inner row are called kichwa ('head'), while the name kimbi applies to both the kichwa and the pits adjacent to them (i.e., the second and next to last pit in the row)."

Dus:
- Kichwa = pits 1 en 8 (uiteinden)
- Kimbi = pits 1, 2, 7, 8 (kichwa + naburen)

Geziefer's "left kichwa or kimbi" = pits 1–2; "right kichwa or kimbi" = pits 7–8 — past **exact** in dit schema. Het is geen aparte non-overlapping set; "kimbi" is de containerterm.

## 7. Nyumba-mechaniek

### Ownership en functional/non-functional/destroyed

> "Players loose their house if it is emptied or when the first harvest."

Twee triggers voor verlies van "owned"-status: (a) eigen sow leegt nyumba, (b) eerste keer een capture-sow vanaf nyumba (= "first harvest" — interpretatie). Strikt genomen impliceert dit een **2-state-eigenschap (owned / not-owned)** bovenop een **functional-flag (≥6 of niet)**, wat samen 3 effectieve toestanden geeft:

1. **Functional**: owned ∧ ≥6 kete
2. **Tijdelijk-onklaar**: owned ∧ <6 kete (kan bijgevoed worden tot ≥6 → terug functional)
3. **Destroyed**: not-owned (permanent — emptied of harvested-from)

Dit komt overeen met **user-bevestiging** (zie RULES.md §A C1) én geziefer's interne model.

### Kunamua-restricties

> "It is not allowed to start a move without harvest (kutakata) start from the owned house with six or more seeds."

Een functional nyumba kan geen takata-bron zijn — tenzij hij de enige optie is:

> "If the house is the only filled pit: one seed from the store must be put in it, two seeds have to be taken out and spread to the left or to the right."

**Dit is de tax-regel maar genuanceerd**: tax wordt alleen geactiveerd als nyumba de enige niet-lege bowl is (compatibel met geziefer's `getKunamuaPossibleNonCaptures`-functional-tak, lines 539–554).

### Sowing eindigt in nyumba

> "If the player still owns the nyumba and a sowing ends in the nyumba then: the move ends during kutakata if the house contains six or more seeds; the move ends during a harvest move if no harvest is possible at the nyumba and if the player wishes to stop."

Dit is dus de **safari-conditie** geherformuleerd: tijdens een capture-sow die in eigen nyumba uitkomt zonder zelf een capture te triggeren, mag de speler stoppen — daar de keuze. Tijdens kutakata wordt automatisch gestopt zodra count ≥ 6 (vergelijkbaar met geziefer's nyumba-stop in takata).

## 8. Safari

**Niet expliciet als naam genoemd** in de blog, maar **functioneel beschreven** in §7 hierboven. De keuze "stop or continue" tijdens een capture-sow in eigen nyumba.

## 9. Endelea / kuendelea

> "If a sowing ends in a non-empty pit... and a harvest is not allowed, then the move continues in the same direction by taking all the seeds from that pit and sowing the seeds starting at the next pit in the same direction. This continuation of sowing is called kuendelea."

**Officiële naam: kuendelea** (werkwoordsvorm; "endelea" is de stam/imperatief). CLAUDE.md's "endelea" is acceptable.

### Kuendelea-termination

Kuendelea stopt bij:
- Harvest mogelijk (move continues with harvest)
- Sowing eindigt op owned house met 6+ seeds en speler wil niet doorspelen
- Sowing eindigt op een **kutakatia-ed pit**

## 10. Kutakatia (anti-starvation rule)

> "If a player must kutakata, after that the opponent kutakata-ing has left him/her with an exactly one of the opponent's pit that can be harvested, then the opponent is not allowed to empty it. This is a kutakatia-ed pit."

**Exclusies (pits die niet kutakatia-ed kunnen worden)**:
- "the still owned house"
- "the only occupied pit in the front row"
- "the only pit containing more than one seed in the front row"

**Effect**: "If kuendelea ends in a kutakatia-ed pit, the move ends."

**Vergelijking**:

| Aspect | A (blog) | G (geziefer) | MWW (snippet) |
|--------|----------|--------------|---------------|
| Trigger | na takata, opponent heeft exact één veld kapen-mogelijk | id. + opponent kan zelf niet kapen | id. (compact) |
| Exclusies | nyumba + only-occupied + only-with-≥2 | identiek | alleen nyumba |
| Effect | endelea stopt op blocked pit; zet eindigt | opponent mag niet legen + blokkeerder MOET kapen | endelea stopt; opponent mag niet vandaar starten |
| Duur | niet expliciet | 3 zetten | niet expliciet |

A en G overeenstemmen op exclusies (3-tuple); MWW noemt alleen nyumba. Effect-beschrijving in A is compatibel met G; MWW geeft een aanvullende formulering. Duur blijft open.

## 11. No-suicide regel

Rule 1.5.3.1: "If the only filled pit on the front row is one of the kichwa-s, then kutakata cannot be done in the direction of the back row (because the front row will be empty and the game is a loss)."

**Gerichte no-suicide-regel**: van een kichwa als enige niet-lege mbele-pit mag de takata niet richting nyuma. Geziefer skipt deze (line 1006: "no prevention of a move which causes loss of the player").

**Beslissing voor onze engine**: implementeren als blocking move-validation (correct conform A) of skip (zoals G)? **Open Question.**

## 12. Infinite-moves-regel

Rule 1.5.6 vermeld als illegaal in de blog (geen exacte tekst gevangen in extract). Ondersteunt onze 256-hop hard cap + geziefer's 12-rounds-zombie als sanity-check.

## 13. Bao la Kujifunza

**Niet behandeld** in deze blog.

## Niet in deze bron

- Specifieke initial position van Kujifunza
- Welke speler begint
- Bestaat nyumba in Kujifunza
