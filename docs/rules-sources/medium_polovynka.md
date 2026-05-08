# Medium — Dmytro Polovynka, "Learn how to play Bao la Kiswahili step-by-step"

> Bron: [`medium.com/@navpil/learn-how-to-play-bao-la-kiswahili-step-by-step-an-evolutionary-approach-7f1131bc3e0c`](https://medium.com/@navpil/learn-how-to-play-bao-la-kiswahili-step-by-step-an-evolutionary-approach-7f1131bc3e0c) — bezocht via WebFetch op 2026-05-08.
>
> **Status**: secundaire bron, blog-tutorial. Datum 2026-03-09. Geen attributie naar primaire bron in fetch-resultaat. Behandelt Kujifunza expliciet.

## 1. Initiële opstelling

> "Planting Phase (Namua): 6 seeds into the fifth hole, 2 seeds into the sixth and 2 seeds into the seventh hole of their inner rows" met 22 in reserve.

Bevestigt G+W+A+K op alle punten.

## 2. Wincondities

Verlies bij:
- "he can't move (same as in Hus)"
- "his inner row is empty"

Geen draw mentioned.

## 3. Capture-condities

**Namu**: "planted his seed into a hole and the opposing hole is non-empty" → capture.

**Mtaji**: zelfde trigger; last seed in non-empty opponent inner-row hole.

**Mandatory**: "if a capturing move can be made, then a player must make it."

> "only seeds from the inner opponent hole are captured. The seeds in the back (outer) row are always safe."

**Belangrijk**: bevestigt dat **alleen mbele-kete worden gepakt**, geen nyuma. Dat beantwoordt een nuancevraag in onze §6.1.

## 4. Niet-capture mtaji-keuze

> "a player must pick the seeds from the front row and sow those. If all the front row holes are empty or contain singletons only, then a player picks the seeds from the back row as sows those."

**Bevestigt geziefer's prioriteit-orde voor mtaji-takata**: eerst mbele met ≥2, alleen daarna nyuma.

## 5. Kichwa-selectie

> Forced (flanks): "If a capture happened in one of the flanks, then a player is forced to choose a nearest head for sowing."
>
> Free choice (central): "if a capture happened in one of the 4 central holes, so not in the flanks, then a player is free to choose which head to sow from."
>
> "if the head is empty, the move ends" / "if the head is not empty, then relay sowing continues."

Het laatste is interessant: sowing vanaf een kichwa met 0 kete bestaat niet (er is geen kete om te sowen?). Vermoedelijk bedoeld: na de capture-deposit in kichwa, wordt vanaf daar gesowt; als de kichwa LEEG was vóór deposit, eindigt de zet na 1 kete-drop (geen verdere kete om mee te sowen). Vergelijkbaar met geziefer's loop.

## 6. Nyumba-toestanden — DRIE STATES EXPLICIET

Polovynka maakt het 3-state model **expliciet**:

### Valid (functional)
> "Valid Nyumba (6+ seeds, never destroyed)"

Gedrag:
- **Blank move (takata)**: "If a last seed of a blank move falls into the house, the move forcefully ends." (= geforceerde stop, geen safari)
- **Capturing move**: "If a last seed of a capturing move falls into the house, a player may decide whether he wants to end the move now, or to continue." (= safari-keuze)

### Invalid (tijdelijk onklaar)
> "Invalid Nyumba (< 6 seeds, temporarily disabled): House becomes invalid and loses all its special abilities — namely if a last sown seed falls there, then the move continues and the house is destroyed."

**Trigger naar destroyed**: een sowing-stap-met-laatste-kete in een invalid house → endelea continueert → house wordt geleegd → destroyed.

### Recovery
> "If a house is refilled with 6 seeds it becomes a valid house again."

Een invalid house kan terug naar valid door bijvoeden.

### Destroyed (permanent)
> "a destroyed house loses its abilities forever" en "can never become valid again."

### Taxation (kunamua only)
> "after the seed is planted there, two seeds are taken from it and sown in a chosen direction" (only valid move when "all other holes in the inner row are empty").

Bevestigt G+A+W: tax-regel als enige niet-lege bowl is functional nyumba.

## 7. Safari

> "when a player decides to relay sow his house it's called safari or go safari."

Bevestigt G+W: safari = de keuze om door te sowen vanuit eigen functional nyumba tijdens een capture-zet.

## 8. Endelea (relay sowing)

> "if a last seed falls into a non-empty hole, then all the seeds from this hole are picked up and are sown starting from the next hole. And so it continues until the seeds falls into an empty hole, which ends the move."

Bevestigt G+A+K.

## 9. Takasia / kutakatia

> "during harvest — a second stage of the game, if a player just made a non-capturing move and all his opponents moves are also non-capturing (blank), and if a player attacked a single opponent hole, then this hole is blocked."

Effect:
> "Opponent can't sow seeds from the blocked hole and if a last seed falls into that blocked hole, sowing immediately stops."

Exclusies (cannot be blocked):
- "non-destroyed valid house"
- "only hole with seeds in the inner row"
- "only hole with more than one seeds in the inner row"

**Bevestigt G+A** drie-exclusies-model. **Conflicteert met K** twee-exclusies — maar K's "the only hole which allows the opponent to play" kan worden gelezen als unie van M's en A's "only-occupied" + "only-with-≥2"; semantisch dus vermoedelijk equivalent.

## 10. No-singleton-regel

> "it's forbidden to play singletons — holes with a single seed in it."

Bevestigt K. **Exception**: "a seed can be planted in any non-empty hole, regardless of how many seeds are there" — dat slaat op planten van ghala-seed in mbele (kunamua), niet op sowing-source. Dus de no-singleton-regel betreft sow-source.

> "if a house is destroyed and there are no capturing moves, then a player has to prefer holes with more than one seed."

Een verfijning: prioriteer ≥2 boven singletons als geen captures mogelijk.

## 11. Kimbi vs kichwa

> Kichwa: "Leftmost and rightmost holes in inner row (positions 1 and 8)."
>
> Kimbi: "Two leftmost and two rightmost holes in the inner rows of each player (so the holes 1, 2, 7 and 8 in the inner row) are called Kimbi."

**Bevestigt definitief**: kichwa = {1, 8}; kimbi = {1, 2, 7, 8}. **Kimbi is bovenset van kichwa.**

## 12. Bao la Kujifunza

Polovynka behandelt Kujifunza expliciet:

**Heeft**:
- Captures alleen uit opponent inner row
- Kichwa selection rules
- Twee zet-types (blank/mtaji)
- Verlies bij lege inner row

**Heeft NIET**:
- Twee-fase namu/mtaji structuur
- Nyumba-mechaniek
- Takasia/blocking
- Taxation

**Bevestigt**: Kujifunza is een vereenvoudiging zonder namu-fase, zonder nyumba, zonder kutakatia. Match onze §10 in RULES.md.

## 13. Author

Auteur: Dmytro Polovynka. Geen verdere attributie. Niet duidelijk of zelf-onderzoek of synthese van bestaande bronnen — gegeven de overlap met kibao.org / abstractstrategygames-blog vermoedelijk geconsulteerd uit één van die bronnen.
