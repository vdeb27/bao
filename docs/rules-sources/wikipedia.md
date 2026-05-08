# Wikipedia — Bao (game)

> Bron: [`en.wikipedia.org/wiki/Bao_(game)`](https://en.wikipedia.org/wiki/Bao_(game)) — bezocht via WebFetch op 2026-05-08.
>
> **Status**: secundaire bron, in essentie een synthese van de Voogt's transcriptie. Wikipedia citeert "Alex de Voogt, who wrote it between 1991 and 1995 based on the teachings of Zanzibari Bao masters" als de meest invloedrijke bron. **Russ wordt niet genoemd**; mogelijk dat het oorspronkelijke plan een misattribution had.

## 1. Initiële opstelling

> "each player initially places 6 seeds in the nyumba, and two seeds in each of the two pits immediately to the right of the nyumba. All the remaining seeds are kept 'in hand'."

Dit laat 22 zaden in de hand per speler. **Bevestigt geziefer §2.1 exact.**

> "all seeds are placed at startup, two per pit. Players thus have no seeds in hand, and thus there is no namua phase." (Kujifunza)

**Bevestigt geziefer §2.2 exact.**

## 2. Spelfases en overgang

> "When players are left without seeds in their hands, the namua phase is over, and a new phase of the game begins, which is called the 'mtaji' phase."

**Bevestigt geziefer §3.3** (beide ghala leeg).

## 3. Capture-condities

**Namua**:
> "A 'marker' pit is a pit of the inner row that faces a non-empty opponent's pit. If the first seed is placed in a marker pit, a capture occurs... A player must capture if he or she can do so."

**Mtaji**:
> "if the last seed of this first sowing is dropped in a marker, a mtaji turn begins" with capture. "Again... the player must capture if he or she can do so."

**Bevestigt geziefer**: mandatory kula in beide fases.

## 4. Kichwa-selectie

> "When a capture occurs, the player takes all of the seeds from the opponent's captured pit, and relay sows them in his or her rows. The first seed must be sown in a kichwa; if it is sown in the right kichwa, sowing will proceed counterclockwise, while if it is sown in the left kichwa, sowing will be clockwise."

> "The choice of the kichwa to sow from is initially left to the player, with a few exceptions. If capture has occurred in any kimbi, sowing must start from the closest kichwa."

> "On relay captures... it is never up to the player to choose which kichwa to sow from... the player must preserve the current clockwise or counterclockwise direction of sowing."

**Bevestigt geziefer §6.3**: deterministisch in 4 van 5 condities, player-keuze alleen bij middle-capture-with-no-prior-direction.

## 5. Safari

> "if sowing in a mtaji turn ends up in the nyumba, and the nyumba is not a marker, the player may freely choose whether to relay-sow the contents of the nyumba or end his or her turn; choosing to continue the sowing from the nyumba is called 'safari.'"

**Bevestigt geziefer §6.4 op alle hoofdpunten**:
- Mtaji-only (niet namu)
- Tijdens een capture-sow (relay-sow)
- Nyumba is non-marker (geen capture-trigger)
- Speler-keuze ja/nee

## 6. Nyumba-mechaniek

> "The nyumba loses its special features the first time its contents are sown (taxation excluded), i.e., the first time the player chooses to relay-sow from the nyumba in a mtaji turn, or if it is captured by the opponent."

> "if, during the namua phase, the player begins his turn sowing from the nyumba, he will only sow two seeds from the nyumba rather than its whole content; this is called 'taxing' the nyumba."

> "if sowing in a takata turn ends up in the nyumba, the turn is over (there is no 'relay-sowing' of the seeds in the nyumba)."

**Bevestigt geziefer §8.4 (tax-regel) als CANONIEK**, niet huisregel. **Bevestigt geziefer §5.2 (takata-stop in nyumba).**

**Conflict met geziefer §8.3**: Wikipedia heeft een **binair model** (special / not-special), waarbij "special" verloren wordt bij eerste sow van content óf bij capture. Geziefer heeft een **drie-toestanden-model** (functional / non-functional / destroyed) waarbij "non-functional" optreedt als count <6 maar nog in bezit. Wikipedia kent dit "non-functional"-tussengebied niet — als nyumba kete heeft, is hij special; als hij geleegd is, is hij gewoon. **Behoeft beslissing.**

## 7. Wincondities

> "The game ends when a player is left without seeds in his or her inner row, or when he or she cannot move anymore. In both cases, this player loses the game."

**Bevestigt geziefer §9.3**: stalemate = verlies, geen remise.

## 8. Wat NIET in Wikipedia staat

- **Endelea** als term komt niet voor (relay-sows wel)
- **Kutakatia / takasia / takatia** — geen vermelding
- **No-suicide** regel — geen vermelding
- **Hard cap** op sowing-rounds — geen vermelding (geziefer's 12-rounds is een implementatie-detail)
- **Niet-canonieke geziefer-skips** (kichwa-with-≥16-kete-restriction etc.) — niet bevestigd of ontkracht

## 9. Bronvermelding

Wikipedia noemt:
- **De Voogt (1995), *Limits of the mind: towards a characterization of Bao mastership*** — PhD thesis, "the most influential transcription"
- **De Voogt (2003), *Muyaka's poetry in the history of Bao***
- **T. Hyde (1694), *De Ludis Orientalibus*** — historisch interessant, niet operationeel

**Russ ontbreekt**. Open Question #3 is mogelijk gebaseerd op een misattribution; we zouden de Voogt 1995 als primary citation moeten aanmerken in plaats van Russ.
