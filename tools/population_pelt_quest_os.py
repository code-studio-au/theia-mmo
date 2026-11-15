"""Native populated-valley pelt quest using ordinary input and read-only receipts."""
import json
import math
import time
from qualify_world_input_os import wait
from qualify_quest_journal_os import LifeGuard, TravelLifeChanged

QUEST = 'theia.dev.quest-wolf-pelts'
PELT = 'theia.dev.wolf-pelt'
OBJECTIVE = QUEST + '.objective.pelts'


def flow(desktop, window, state, folder, quest, journal, feet, face, approach,
         close_panels, speak, action, capture, travel, reach_enemy, await_respawn,
         anchor, audit):
    scale = state()['window'][2]
    records = []
    starting_kills = state()['combat']['deaths']
    starting_revision = journal()['revision']
    def count():
        return sum(s['count'] for s in state()['inventory_bag']['slots'] if s and s['item'] == PELT)
    def progress():
        return quest(QUEST)['counts'][OBJECTIVE]
    def corpses():
        return state()['loot']['corpses']
    def loot_ui():
        return state()['loot']['ui']
    def save():
        (folder/'population-pelt-progress.json').write_text(json.dumps(records, indent=2)+'\n')
    def pickup(corpse):
        assert state()['loot']['particle_count'] == 8 * len(corpses()), 'Loot effect missing or duplicated'
        particles_before = state()['loot']['particle_count']
        close_panels()
        approach(lambda: corpse['position'], tolerance=1.8)
        face(corpse['position'])
        guard = LifeGuard(state)
        before = count()
        selected = None
        for dx,dy in [(0,0),(0,8),(0,-8),(8,0),(-8,0),(16,0),(-16,0),(0,16),(0,-16)]:
            guard.check()
            observed = wait(lambda: next((c for c in guard.check()['loot']['corpses'] if c['life']==corpse['life'] and c['screen']), None))
            x,y = observed['screen']
            width,height,_ = state()['window']
            if not (0 < x+dx < width and 0 < y+dy < height):
                continue
            desktop.click(window, [x+dx,y+dy], scale, 3)
            time.sleep(.3)
            if loot_ui()['open']:
                selected = state()['loot']['selected_life']
                assert any(c['life']==selected for c in corpses())
                break
        assert selected is not None, 'Normal right-click did not open nearby corpse'
        assert count()==before and progress()==before, 'Viewing a corpse granted quest credit'
        revision = state()['inventory_bag']['revision']
        submitted = loot_ui()['submitted']
        capture('loot-available.png')
        guard.check()
        desktop.click(window, wait(lambda: guard.check()['loot']['layout']['all']), scale, 1)
        wait(lambda: guard.check()['loot']['ui']['submitted'] > submitted)
        wait(lambda: guard.check()['inventory_bag']['revision'] > revision and not loot_ui()['open'])
        wait(lambda: progress()==count())
        wait(lambda: state()['loot']['particle_count'] == 8 * len(corpses()))
        capture('loot-taken.png')
        assert not any(c['life'] == selected for c in corpses()), 'Fully looted corpse still offers loot'
        records.append(dict(pickup=selected, before=before, after=count(),
                            particles_before=particles_before, particles_after=state()['loot']['particle_count'],
                            bag_revision=state()['inventory_bag']['revision']))
        save()
        close_panels()
        # Stay at the ordinary pickup location. Walking back into the centre of
        # a living pack is unnecessary before selecting the next ranged target.

    assert quest(QUEST) is None and count()==0
    speak()
    action(QUEST, 'Active')
    assert progress()==0
    capture('pelt-accepted.png')
    close_panels()
    travel()
    for attempt in range(18):
        try:
            guard = LifeGuard(state)
            # Previously denied lives may offer only tokens. Collect through the
            # same ordinary UI; acceptance never creates retroactive pelt rights.
            while corpses() and count() < 3:
                pickup(min(corpses(), key=lambda c: math.dist(c['position'], feet())))
            if count() >= 3:
                break
            wait(lambda: guard.check()['combat']['last_mana'] >= 95, 30)
            generation = reach_enemy()
            victim = next(t for t in state()['target_entities'] if t['entity']==state()['selected_hostile'])
            life = list(victim['epoch'].to_bytes(8, 'little') + victim['entity'].to_bytes(8, 'little'))
            deaths = state()['combat']['deaths']
            deadline = time.monotonic()+75
            while state()['combat']['deaths'] == deaths:
                guard.check()
                assert state()['combat']['player_generation']==generation
                assert time.monotonic() < deadline, 'Bounded pelt fight timed out'
                target = next((t for t in state()['target_entities'] if t['entity']==victim['entity']), None)
                if target is None:
                    # Target invalidation and combat events use distinct bounded
                    # channels. A fatal target can disappear a frame before its
                    # death event reaches telemetry; wait for that event.
                    wait(lambda: state()['combat']['deaths'] > deaths, 5)
                    break
                assert target['epoch']==victim['epoch']
                face(target['position'])
                desktop.key('1')
                time.sleep(.25)
            receipt = wait(lambda: audit(life), 15)
            pelt_roll = next(r for r in receipt['rolls'] if r['key']=='entry:theia.dev.wolf-pelt-loot')
            assert pelt_roll['eligible'] and pelt_roll['journal'] >= 15
            assert count()==progress(), 'A kill changed item credit without pickup'
            records.append(dict(kill=life, receipt=receipt, health=state()['combat']['player_health'], pelts=count()))
            save()
            if receipt['pending']:
                wait(lambda: any(c['life']==life for c in corpses()), 10)
        except TravelLifeChanged:
            await_respawn()
            records.append(dict(respawn=state()['combat']['player_generation'], player_deaths=state()['combat']['player_deaths']))
            save()
            close_panels()
            travel()
    else:
        raise AssertionError('Normal conditional pelt loop exceeded bounded attempts')
    assert count()==3 and progress()==3 and quest(QUEST)['status']=='Ready'
    capture('pelt-ready.png')
    close_panels()
    try:
        travel(reverse=True)
    except TravelLifeChanged:
        await_respawn()
    speak()
    action(QUEST, 'Completed')
    wait(lambda: count()==0 and journal()['xp']==250)
    assert progress()==3
    kills = state()['combat']['deaths']-starting_kills
    # Every new kill has its own durable journal receipt, even when it changes
    # no objective. Add acceptance, three possession updates and turn-in.
    assert journal()['revision'] == starting_revision + kills + 5
    capture('pelt-completed.png')
    result = dict(passed=True, records=records, kills=kills, revision=journal()['revision'], xp=journal()['xp'],
                  bag=state()['inventory_bag'], player_deaths=state()['combat']['player_deaths'])
    (folder/'population-pelt-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
