# Mancala World wiki — `Bao_la_Kiswahili`

> Bron: [`mancala.fandom.com/wiki/Bao_la_Kiswahili`](https://mancala.fandom.com/wiki/Bao_la_Kiswahili) — directe WebFetch HTTP 403; volledige inhoud verkregen via gebruiker-copy-paste van de wiki-source op 2026-05-08.
>
> Auteurschap rules-sectie: **Ralf Gering** (User:Mr Mancala). License: CC BY-SA 2.5. Het artikel is een synthese van Bao-literatuur; de rules-sectie is consistent met de Bao Society / dV-traditie maar niet expliciet één bron toegeschreven.
>
> **Status**: complete bron, nu integreerbaar. Citeert primaire literatuur (de Voogt 1995, Townshend 1986, Hyde 1694, etc.) in een uitgebreide referentie-lijst.

## 1. Bord-topologie

> "The Bao board consists of four rows, each one with eight holes. The holes are rounded except the fourth from the right in the central rows, which is square in shape and called nyumba ('house')."

Nyumba is fysiek gemarkeerd als vierkante hole. **Positie: vierde-vanaf-rechts in de inner row** = field 5 vanuit links voor South (geziefer-conventie). Bevestigt G+W+A+M+K.

> "The ultimate holes at either end of the inner rows are called kichwa ('head') and both, the ultimate and the penultimate holes are known as kimbi (according to P. Townshend this word could be derived from kimbia = 'very fast')."

**Kichwa = {1, 8} (ultimate); kimbi = {1, 2, 7, 8} (ultimate + penultimate)**. Bevestigt M+W+A definitief.

## 2. Nyumba-toestanden — letterlijke 3-state-bevestiging

> "A nyumba ceases temporarily to be a functional nyumba, when it has less than six seeds, and ultimately, when its contents have been captured or moved in a lap. In the rules given below, a nyumba is always meant to be a 'functional nyumba'."

Drie expliciete toestanden:
- **Functional**: ≥6 kete én niet-vernietigd
- **Tijdelijk-onklaar** ("temporarily ceases to be functional"): <6 kete maar niet vernietigd
- **Permanent destroyed** ("ultimately"): contents captured of moved-in-lap

Trigger naar destroyed:
- Captured by opponent
- "moved in a lap" — d.w.z. een eigen sow heeft de nyumba leeggemaakt of doorgespeeld

**Vierde bron die het 3-state-model expliciet bevestigt** (na G+M+K).

## 3. Initiële opstelling

> "each player has 22 seeds in reserve" + 6 in nyumba + 2 in elk van de twee buurkolommen rechts van nyumba.

Bevestigt G+W+A+M+K.

## 4. Algemene regels (BELANGRIJK — niet eerder geconsolideerd)

> "Bao la Kiswahili is a game with multilap sowing. Each player only sows around his own two rows."
>
> "Moves can be with or without capturing. Non-capturing moves are also known as takata. Captures are mandatory."
>
> "A prerequisite for making a capture is to have at least two occupied holes facing each other in the players' front rows."
>
> "Any such position results in a capture during the namu stage, but in the mtaji stage the last seed of the first lap must fell into an occupied hole in opposition to really effect a capture."
>
> "Only the contents of the opponent's front row can be captured while those in his back row are safe."

### Algemene regels die het plan/RULES.md nog mistte:

> **First-lap-determines-captures**: "If the first lap of a move doesn't capture, nothing will be captured in the full move. On the other hand, if the first lap captures, multiple captures can follow, even if they will be interrupted by non-capturing laps."
>
> **16-seeds-no-capture-mtaji**: "If 16 or more seeds are sown in the first lap, nothing will be captured. Note that this rule only applies to the second stage because a move always starts with a single seed in the first stage."

**Beide regels zijn essentieel voor capture-validatie**:
- First-lap rule: een takata-eerste-lap kan niet later in een capture-lap omslaan, ook al ontstaan er endelea-laps.
- 16-seeds rule: mtaji vanaf een pit met ≥16 kete telt als takata, geen capture mogelijk.

**Geziefer**: heeft de 16-seeds rule (regel 360 `count <= 15`) maar skipt expliciet andere edge cases. First-lap rule is impliciet (zijn capture-detection check zit alleen op landing van eerste sow + endelea-laps van capture-mode).

## 5. Namu-fase

### Niet-capturende zetten

> "If it is not possible to make a capture, the player takes a seed from his reserve and puts it into a non-empty hole in his front row:
> - If the player has a nyumba, he is not permitted to put the seed into it, unless it is the only occupied hole in his front row.
> - If the player has no nyumba, he can only add the seed to a hole, which contains at least two seeds, unless all non-empty holes in the front row are singletons."

Bevestigt G's `getKunamuaPossibleNonCaptures` regel 519–567 op alle drie de takken (no-possession, functional, geen-functional).

> "After that the player picks all the seeds from this hole and sows them into consecutive holes in either direction, clockwise or anticlockwise."

### Tax (namu vanaf nyumba)

> "If, however, the seed is put into a nyumba, he takes just two seeds from it and sows them in either direction."

**Tax is dus alleen mogelijk wanneer nyumba de enige niet-lege mbele-pit is** (per voorwaarde hierboven). Bevestigt G+W+A+M+K.

### Endelea + nyumba-stop

> "If the last seed is sown into a non-empty hole, but not a nyumba, its contents are taken and the sowing continues until the last seed falls in an empty hole, which also ends the turn."
>
> "If, however, the lap ends in the nyumba, the move is not continued and the turn is over without delay."

Bevestigt: in namu-takata wordt een sow geforceerd gestopt als hij in eigen functional nyumba eindigt.

### Capturing zetten

> "After the player has put a seed into a hole, which effects a capture, he takes the contents of the opponent's inner hole opposite to it and sows them towards the center of his inner row starting in a kichwa:
> - If he has captured from a kimbi, he must start in the kichwa of the same side (left or right).
> - If he has captured from the four central holes, he may choose the kichwa."

Match aan G+W+A+M+K.

### Multi-capture kichwa direction

> "He continues in laps as in takata unless the last seed is dropped into an occupied hole of his inner row and the opponent's hole opposite is not empty either, which results in another capture:
> - The captured seeds must now be sown towards the center from the kichwa, which is in the direction from where he arrived (so that the direction of sowing remains unaltered) unless he captured from a kimbi of the other end of the row. Then he starts from the kichwa of this side and the direction of sowing is reversed."

Match aan A+K.

### Safari (namu)

> "If, however, the player ends a lap in his nyumba, he can either choose to stop sowing or he may continue (called safari), which would destroy the nyumba forever."

In namu-capture-sow eindigend in eigen functional nyumba: keuze. Bevestigt G+W+A+M.

## 6. Mtaji-fase

### Niet-capturende zetten

> "If the player has no reserve seeds left and cannot capture, he may choose any hole of his front row (including the nyumba), which contains more than one seed, and then sows its contents in either direction."
>
> "If there are only singletons in the front row, he may take a hole in his back row, but no singletons."

Bevestigt no-singleton-regel (M+K). Bevestigt prioriteit van mbele over nyuma.

### No-suicide regel (sterk geformuleerd)

> "The front row may never be emptied, not even temporarily."
>
> "If the only occupied hole of the front row is a kichwa and it contains two or more seeds, they must be sown towards the center of the front row."

**Twee no-suicide-regels** in mtaji:
1. Algemene regel: front row mag NOOIT geleegd worden, ook niet tijdelijk.
2. Specifiek: kichwa-with-≥2 als enige niet-lege mbele → MOET towards center.

**Sterker dan A's "kutakata cannot be done in the direction of the back row"**: MWW zegt "must be sown towards the center" — niet alleen back-row verboden, maar specifiek center-richting verplicht.

### Capturing zetten

> "A capture can be effected starting from any hole in either row with at least two seeds. The captured seeds are sown in a new lap towards the center from the kichwa, which is in the direction from where he came (so that the direction of sowing remains unaltered) unless he captured from a kimbi of the other end of the row."

### Mtaji-safari — UITZONDERING t.o.v. andere bronnen

> "In contrast to the namu stage, the player must safari (continue to sow), if the sowing ends in the nyumba."

**MWW UNIEK**: in mtaji is safari **VERPLICHT**, geen keuze. Andere bronnen (G+W+A+M) zeggen het is een keuze in mtaji ook. **Nieuw conflict — zie §A C5 in RULES.md.**

## 7. Kutakatia / takasia

> "If after a takata move the contents of only one opponent's hole are under threat of being captured, but not one of the player's own holes is menaced (that is, the opponent must also takata), this hole is 'takasiaed'. The opponent cannot start his turn from it nor would a move be continued, if a lap ends in it unless it has been reached in the first lap from a nyumba. However, a nyumba itself cannot be takasiaed. Nor can a hole that is the only occupied one or the only one containing more than one seed in the player's front row."

**Drie exclusies match G+A+M**: nyumba zelf, only-occupied, only-with-≥2. **Plus extra detail**: lap-stop "unless it has been reached in the first lap from a nyumba" — een sow die direct vanaf nyumba in een takasiaed pit eindigt, kan wél doorgaan. Voor onze engine een nuance om te implementeren.

Duur: niet expliciet (G's 3 zetten blijft enige bron).

## 8. Wincondities

> "The player wins either by 'Bao hamna', that is capturing all seeds of the opponent's front row, or by leaving his opponent just singletons, so that he isn't able to move."

Hamna = mbele-leeg-na-eigen-zet. Stalemate = "just singletons" — opponent kan nergens vandaan een legale ≥2-zet starten. Match no-singleton-rule + verlies.

## 9. Etymologie & geschiedenis

Niet voor regels relevant, maar het wiki-artikel geeft veel achtergrond:
- Eerst beschreven door Étienne de Flacourt (1658) in Madagascar onder de naam "Fifangha"
- Bao poem "Bao Naligwa" door Muyaka bin Haji ca. 1820s
- Chama Cha Bao opgericht 1966 in Tanzania
- de Voogt's PhD (1995) is de hoofdacademische bron
- "The rules of Bao Kiswahili are considered to be the most difficult and complex to learn of all mancala games."

## 10. Bronnenlijst (voor toekomstige raadpleging)

Het wiki-artikel citeert ~80 bronnen. Belangrijkste primaire / academische voor onze doeleinden:

- **Voogt, A. J. de. (1995)**. *Limits of the Mind*. CNWS Publications, Leiden.
- **Townshend, P.** (meerdere papers, 1977–1986; PhD-thesis Cambridge 1986)
- **Donkers, H. H. L. M. (2001)**. *Zanzibar Bao Rules for the Computer*. Universiteit Maastricht. ([gamecabinet-link](http://www.fdg.unimaas.nl/educ/donkers/games/Bao/rules.html))
- **Russ, L. (1999)**. *The Complete Mancala Games Book*. Marlowe & Company. Pages 122–127. (Bao behandeld; mogelijk overlap met onze oorspronkelijke verwarring)
- **Vessella, N. (2010)**. *Il Libro Quasi Completo del Gioco del Bao*. Italiaanse uitgave.

Donkers' Maastricht-tekst en Russ' boek (122–127) zouden directe waarde toevoegen voor verdere validatie.
