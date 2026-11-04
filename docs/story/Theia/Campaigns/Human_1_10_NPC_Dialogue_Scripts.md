# Theia — Human Levels 1–10 NPC Dialogue Scripts, Pass 1

**Status: Needs Review — authored dialogue, not recorded or implemented.** Based on [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md) and [Q001–Q017 working authoring review](Human_Q001_Q017_Rust_Authoring_Review.md). This expands the earlier [vertical slice dialogue brief](Human_1_10_Vertical_Slice_Implementation.md) into reusable quest-offer, progress, choice and turn-in dialogue. All lines are original first-pass text.

**Coverage:** This first pass now contains Q001–Q017, including WORKING drafts for optional Q014–Q017 and Elara's Q013 completion handoff. Their canonical quests and outcomes remain authoritative in the campaign; draft dialogue and completion-evidence detail do not independently approve extra objectives or mechanics.

## Dialogue design constraints

The player is a new Human resident/traveler, not a chosen hero. Early speakers know nothing of Nhal, the Seven's Binding, the Conclave, the Guardian gate, or the Kel'vari. The stolen goods are a local mystery. NPCs retain individual viewpoints. Dialogue choices change tone and later acknowledgement, never the canonical outcome. Dialogue is text-first, concise enough for an MMO dialogue panel. No player voice or gender assumption.

**Placeholder tokens:** `{player_name}` optional; avoid spoken player name by default. `{item_name}` and `{location_name}` only when verified by content implementation. Script IDs are editorial aliases, not engine localization keys.

## Q001 — Before the Next Caravan

**Elara, offer:** “Looking for work? Start by learning the place. Darian keeps the road watch, Anwen runs the healing house, and Brenna handles the caravans. Speak to all three. Then come back and tell me what you found.”

**Player choice A:** “I'll speak to them.” → **Elara:** “Good. Ask questions before you make promises.”

**Player choice B:** “Why does the village need another pair of hands?” → **Elara:** “Because a small place notices every absence. And we've had one too many lately.”

**Darian, introduction:** “Elara sent you? The patrol goes as far as the western bend. Most days it's nothing but tracks and dust. That's how trouble gets close.”

**Anwen, introduction:** “The healing house isn't glamorous. We need clean water, supplies and people who can carry them without dropping everything.”

**Brenna, introduction:** “My next caravan is overdue. I don't expect you to find it by asking three questions, but at least you'll understand what's missing.”

**Elara, progress:** “Darian, Anwen and Brenna. All three. I want their accounts, not guesses.”

**Elara, completion:** “Good. Start with Darian's patrol. If the road is unsafe, every other problem gets worse.”

## Q002 — The Road Patrol

**Darian, offer:** “Come with me to the western bend. Two places had fresh predator signs yesterday. We'll check them, then find a worker who hasn't come back from the road.”

**Player choice A:** “I'll keep watch.” → **Darian:** “Watch where you're standing, too. A beast won't wait for you to turn around.”

**Player choice B:** “Why not send more guards?” → **Darian:** “Because I can't leave the village gate empty. We work with what we've got.”

**Darian, first sign:** “Fresh. See how the ground's pressed down? Something crossed here recently.”

**Darian, second sign:** “Another trail. Same direction. We should find that worker before the animals do.”

**Worker, discovered:** “I heard them in the brush and stayed off the road. Can you get me past the bend?”

**Worker, after rescue:** “Thank you. I'll tell the others to avoid that path until the watch clears it.”

**Darian, progress:** “We need both trail reports and the worker safely back.”

**Darian, completion:** “You kept your head. I'll tell Elara the road needs another patrol. Anwen's parcel is still waiting, if you want to help.”

## Q003 — What the Healing House Needs

**Anwen, offer:** “The medical parcel was left at the roadside store. Please bring it back sealed. I've got patients waiting and no spare hands.”

**Player choice A:** “I'll collect it.” → **Anwen:** “Thank you. Ask the storekeeper before opening anything.”

**Player choice B:** “Is anyone in danger?” → **Anwen:** “Not if the supplies arrive soon. That's why I asked.”

**Storekeeper:** “Anwen's parcel? Right here. I kept it away from the damp.”

**Anwen, parcel received:** “Good. Put it on the table. There's something I'd like you to see.”

**Patient:** “I thought I'd be laid up for weeks. It barely aches now.”

**Anwen, quietly:** “The wound's closing faster than I expected. I can't explain it yet, and I won't pretend I can.”

**Anwen, completion:** “You've helped more than you know. Brenna still needs someone to check the caravan route.”

## Q004 — The Missing Wheels

**Brenna, offer:** “The teamster should have checked in at the western marker. Follow the road and look for the wagon. If you find anyone alive, listen before you act.”

**Player choice A:** “I'll find out what happened.” → **Brenna:** “Bring back details. We can do something with details.”

**Player choice B:** “Was it bandits?” → **Brenna:** “Maybe. But I won't accuse anyone until we know.”

**Wagon inspection, player journal:** “Wheel damage, scattered ordinary goods and empty fittings where selected crates were secured.”

**Teamster, discovered:** “I hid when they came back. They took the crates with the marked seals. Left the rest.”

**Player response:** “Did you see who led them?” → **Teamster:** “Not clearly. One of them said the buyer wouldn't pay for substitutions.”

**Brenna, completion:** “So they knew what they wanted. Elara will want to hear this. For now, the road still needs people willing to help.”

## Q005 — The Fence at Dawn (optional)

**Farm steward, offer:** “The fence went down overnight. Animals got in and the livestock scattered. Could you clear the pasture while we repair it?”

**Player choice:** “I'll help.” → **Steward:** “Good. Keep the animals away from the workers.”

**Steward, progress:** “The pasture isn't safe until the fence stands and the predators are gone.”

**Steward, completion:** “That's a night's worry lifted. Tell Elara we can use the field again.”

## Q006 — Seed Before Winter (optional)

**Tomas, offer:** “The ledger says I owe two lots of seed. I've only got one storehouse and one field. Will you check the count before the collector comes?”

**Player choice A:** “I'll check the records.” → **Tomas:** “The sacks are all marked. Start there.”

**Player choice B:** “Maybe the collector is right.” → **Tomas:** “Then I'd like to know how. Count them first.”

**Tomas, discovery:** “Two entries for the same delivery? That explains why the numbers never worked.”

**Choice A, correction:** “We'll ask the clerk to strike the duplicate.” → **Tomas:** “Fair enough. I'd rather have it right than have someone blamed.”

**Choice B, correction:** “We'll attach a signed explanation.” → **Tomas:** “That'll do. Gives them less room to argue.”

**Tomas, completion:** “Now we can plant instead of fighting over ink.”

## Q007 — A Cart That Can Move (optional)

**Heddan, offer:** “See that cart? It's useful as a bench and not much else. I need ordinary iron for the axle and someone to hold it steady while I fit the wheel.”

**Player choice:** “Where can I find iron?” → **Heddan:** “There's scrap by the old worksite. Don't take anything still holding a roof up.”

**Heddan, at repair bench:** “Keep the pin straight. That's it. Now give the wheel a turn.”

**Heddan, completion:** “There. A cart that moves. Not every good deed needs a song written about it.”

## Q008 — The Healer's Ledger (optional)

**Anwen, offer:** “I'd like to record how quickly people are recovering. Not rumors—dates, symptoms, what we actually observed. Will you help?”

**Patient A:** “I was walking sooner than the healer expected. I don't know why.”

**Patient B:** “My fever broke overnight. That's all I can tell you.”

**Anwen, progress:** “Don't turn a few stories into a miracle. We need careful records.”

**Anwen, completion:** “Thank you. Maybe one day these notes will help someone understand what's changing.”

## Q009 — An Unusual Buyer

**Elara, offer:** “The teamster's account gives us a place to start. Iven Rusk buys stolen cargo from the hills. Find out what he took and who paid for it. Bring back evidence, not a body.”

**Player choice A:** “I'll recover the orders.” → **Elara:** “And the cargo if you can. Don't risk the whole village for a few crates.”

**Player choice B:** “What if Rusk attacks?” → **Elara:** “Defend yourself. But I want the name behind the theft.”

**Rusk, first confrontation:** “You came a long way for someone else's boxes. Whoever sent you should have stayed home.”

**Rusk, pressured:** “I moved what they asked me to move. I didn't ask why. The payment was good.”

**Buyer order, journal:** “Specific crates requested. Substitutions forbidden. Official-looking seal damaged.”

**Elara, completion:** “A buyer who knew the cargo, and a seal that looks official. Heddan can tell us something about the workmanship.”

## Q010 — A Seal Without a Name

**Heddan, offer:** “I can compare the metalwork, but don't mistake that for knowing who issued the order. Bring the seal to my bench.”

**Heddan, inspection:** “The fitting resembles stock used by officials. That's all I can prove. Someone could have copied it.”

**Player choice A:** “So the order is royal?” → **Heddan:** “I didn't say that. That's exactly the mistake I'm warning you against.”

**Player choice B:** “Who can verify it?” → **Heddan:** “A proper records office. Stoneford's the nearest place to start.”

**Heddan, completion:** “Take both the fitting and the shipment record. One without the other tells half a story.”

## Q011 — At the River Crossing

**Brenna, offer:** “A caravan leaves for Stoneford today. Go with it and check the warehouse. You'll learn more from a ledger than from another fight on the road.”

**Caravan leader, departure:** “We travel together and stop at the crossing. Stay close enough to hear the signal if trouble comes.”

**Caravan leader, arrival:** “Stoneford. Get your papers ready. The warehouse won't open its books for a stranger without a reason.”

**Warehouse contact:** “Brenna sent you? Then we can talk. But we'll need the original dates.”

## Q012 — Three Books

**Local correspondent, offer:** “Stock book, courier docket, warehouse tally. One shipment appears in all three, but the dates and quantities don't agree. Find the contradiction.”

**Player choice A:** “Could be a copying mistake.” → **Correspondent:** “Could be. That's why we compare all three.”

**Player choice B:** “Someone falsified them.” → **Correspondent:** “Maybe. Evidence first, accusation afterward.”

**Incorrect comparison:** “Look again. Two records agree on the dispatch date. Which one doesn't?”

**Correct comparison:** “There it is. The goods were signed out before they were supposedly received.”

**Correspondent, completion:** “Now we have something that can be independently checked. Perrin handled the transfer paperwork.”

## Q013 — The Office After Edric

**Correspondent, offer:** “Perrin kept the payment instructions from the office transfer. We need the original and his account, with the reeve's authority.”

**Perrin, questioned:** “I copied the order as it was given. The office had changed hands after Edric died. I didn't choose who signed it.”

**Player choice A:** “Who authorized the payment?” → **Perrin:** “The seal was on the instruction. The name wasn't mine to question.”

**Player choice B:** “Why keep a copy?” → **Perrin:** “Because I knew someone would ask later. I didn't expect it to be you.”

**Correspondent, referral handoff:** “This belongs in Valecrown. Elara can introduce you to Mara Thorne. She'll know what to do with a record like this.”

**Q013 completion boundary:** Obtain Elara's introduction to Mara after the payment evidence and authorized interview. The correspondent's referral is not final completion.

**Elara, completion (WORKING dialogue; after original evidence and authorised interview):** “You brought the instruction back, and Perrin answered under the reeve's authority. That's something Mara can examine, not just another rumour about Edric's office.”

**Player:** “Will she believe us?”

**Elara:** “She'll test the record. That's why I'm sending you to her. Take my introduction and the authenticated copy; keep the original with its lawful custodian.”

**Completion evidence:** Elara issues the introduction to Mara only after payment-copy recovery and authorised Perrin interview; no accusation against Cassian is treated as proved at Q013.


## Q014 — The Quarry Bell (optional)

**Joss, offer:** “The Greyfall shift should have returned with the evening bell. The tally's missing too. I need the crew found before anyone starts calling this sabotage.”

**Player:** “Where did they last work?”

**Joss:** “The lower workface. The predators have been moving through the old cut. Follow the bell route and bring the people out first.”

**In progress:** “The crew and shift tally are both still unaccounted for. Don't leave anyone behind for a piece of paper.”

**Rescued worker:** “We took shelter when the predators came through. The tally is in the tool chest; it lists everyone who entered.”

**Joss, completion:** “They're alive, and the count matches. Thank you. The quarry needs a safer route, not a conspiracy invented to explain every broken shift.”

**Completion evidence:** Missing workers rescued, ordinary cave threat cleared or a separately approved safe alternative used, and shift tally returned. A proposed alternate route is not automatically executable.

## Q015 — One Bridge, Two Claims (optional)

**Stoneford reeve, offer:** “The crossing's damaged. The miller needs grain moved; the merchant needs supplies across. Each has a fair claim and neither can cross an unsafe bridge.”

**Miller:** “If we wait, the flour waits with us. Families won't thank us for a tidy ledger.”

**Merchant:** “And if the repair tools don't arrive, there won't be a bridge to argue over.”

**Player choice A:** “Prioritise the mill's immediate needs.”
**Player choice B:** “Prioritise delivery of repair supplies.”

**Reeve, response:** “I'll record the order you chose. We still owe both sides a safe crossing.”

**Reeve, completion:** “The supports have been checked and the crossing is open. Neither claim was forgotten. That's what a useful decision looks like.”

**Completion evidence:** Inspect damage, hear both parties, record local work order, secure repair needs and reopen a safe ordinary crossing. Choice changes acknowledgement, not permanent denial of either service.

## Q016 — The Warehouse Night Watch (optional)

**Brenna, offer:** “Tools have gone missing from the warehouse. The watch saw movement after closing, but I won't accuse our own workers without proof.”

**Player:** “What do you need?”

**Brenna:** “Check the loss record, walk the perimeter and stop whoever returns tonight. Bring the tools back if you can.”

**In progress:** “Keep to the agreed watch route. A missing crate isn't proof of a larger plot.”

**Brenna, completion:** “Ordinary thieves, then. The tools are back, and the losses can be accounted for. Not every wrong in Stoneford belongs to the same enemy.”

**Completion evidence:** Losses checked, patrol completed, thieves intercepted and stolen tools recovered. Retry after failed combat without resetting unrelated story evidence.

## Q017 — The Long Way Home (optional)

**Elara, offer:** “A section of the caravan has finally been recovered. Their usual road is still blocked. Can you help them bring the remaining supplies home?”

**Caravan driver:** “We'll take the longer route. I'd rather arrive late with everyone than gamble on the old crossing.”

**In progress:** “If the wagons stop, find the agreed safe point. Don't leave the drivers to face the obstruction alone.”

**Elara, completion:** “The survivors are home and the supplies are accounted for. The road changed, but the people still needed to make it back.”

**Player:** “There will be more journeys.”

**Elara:** “Then we'll make them as safely as we can. That's enough work for today.”

**Completion evidence:** Supplies prepared, route obstruction cleared, recovered caravan segment delivered to Millhaven. Rejoining at safe checkpoints remains a WORKING gameplay requirement, not an implemented escort system.

## Implementation requirements

Each quest requires dialogue nodes for offer, in-progress, turn-in, unavailable state and any conditional player choices. Every choice must be saved as a small reaction flag, not a separate canonical storyline. Scene and localization IDs remain to be assigned.

**Dialogue QA:** no spoilers; no player gender assumptions; all critical information readable without voice audio; optional quest dialogue cannot gate main-story evidence; wrong puzzle responses are retryable; NPC availability and death/reset states cannot softlock a quest. All lines need editorial review and performance direction before localization or voice casting.
