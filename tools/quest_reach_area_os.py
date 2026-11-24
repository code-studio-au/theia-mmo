"""Normal OS movement / UI reach-area quest. No authoritative position injection."""
import json
import math
import time
from qualify_world_input_os import wait

QUEST = 'theia.dev.quest-overlook'


def flow(desktop, state, folder, quest, journal, feet, face, close_panels, speak, action, capture, area=None, travel=None, travel_label=None, await_respawn=None):
    # Select via the normal offered quest button, then use only camera/W movement.
    speak()
    action(QUEST, 'Active')
    capture('area-accepted.png')
    close_panels()
    start = dict(feet=feet(), authority_tick=state()['dodge']['tick'])
    deaths_before = state()['combat']['player_deaths']
    began = time.monotonic()
    reached_before_respawn = None
    returned_via_respawn = False
    if area:
        assert travel is not None
        from qualify_quest_journal_os import TravelLifeChanged
        try:
            travel()
        except TravelLifeChanged:
            if await_respawn is None:
                raise
            # Entering the area can complete the objective before the combat
            # route's later endpoint. Preserve evidence at the actual death
            # position; a respawn must never stand in for reaching the area.
            wait(lambda: quest(QUEST)['status'] == 'Ready', 2)
            reached_before_respawn = feet()
            assert all(area['min'][i] <= reached_before_respawn[i] <= area['max'][i] for i in range(3))
            capture('area-reached-before-respawn.png')
            await_respawn()
            returned_via_respawn = True
        wait(lambda: quest(QUEST)['status'] == 'Ready')
    else:
        target = [7864., 0., 8000.]
        face(target)
        desktop.key_state('w', True)
        try:
            wait(lambda: quest(QUEST)['status'] == 'Ready', 45)
        finally:
            desktop.key_state('w', False)
    elapsed = time.monotonic()-began
    reached = reached_before_respawn if reached_before_respawn is not None else feet()
    if area:
        assert all(area['min'][i] <= reached[i] <= area['max'][i] for i in range(3)), reached
    else:
        assert 7856. <= reached[0] <= 7872. and 7984. <= reached[2] <= 8016., reached
        assert -64. <= reached[1] <= 64., reached
    assert journal()['xp'] == 125, 'Area occupancy granted XP before turn-in'
    assert state()['combat']['player_deaths'] == (deaths_before + int(returned_via_respawn) if area else 0)
    capture('area-ready.png')
    if area and not returned_via_respawn:
        from qualify_quest_journal_os import TravelLifeChanged
        try:
            travel(reverse=True)
        except TravelLifeChanged:
            if await_respawn is None:
                raise
            await_respawn()
            returned_via_respawn = True
    elif not area:
        face([8004., 0., 8000.])
        desktop.key_state('w', True)
        try:
            wait(lambda: math.hypot(feet()[0]-8004., feet()[2]-8000.) < 2.1, 45)
        finally:
            desktop.key_state('w', False)
    speak()
    action(QUEST, 'Completed')
    wait(lambda: journal()['xp'] == 150)
    capture('area-completed.png')
    proof = dict(passed=True, input=travel_label or 'manual camera facing + normal W key',
                 outward_start=start, outward_reached_feet=reached,
                 outward_elapsed_seconds=elapsed, return_feet=feet(),
                 returned_via_respawn=returned_via_respawn,
                 area_id=area['id'] if area else 'theia.dev.area.western-overlook',
                 area_ready_without_xp=True, completed_xp=150,
                 quest_revision=quest(QUEST)['revision'])
    (folder/'reach-area-proof.json').write_text(json.dumps(proof, indent=2)+'\n')
    return proof
