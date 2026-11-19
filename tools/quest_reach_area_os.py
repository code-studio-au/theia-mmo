"""Normal OS movement / UI reach-area quest. No authoritative position injection."""
import json
import math
import time
from qualify_world_input_os import wait

QUEST = 'theia.dev.quest-overlook'


def flow(desktop, state, folder, quest, journal, feet, face, close_panels, speak, action, capture):
    # Select via the normal offered quest button, then use only camera/W movement.
    speak()
    action(QUEST, 'Active')
    capture('area-accepted.png')
    close_panels()
    start = dict(feet=feet(), authority_tick=state()['dodge']['tick'])
    target = [7864., 0., 8000.]
    face(target)
    began = time.monotonic()
    desktop.key_state('w', True)
    try:
        wait(lambda: quest(QUEST)['status'] == 'Ready', 45)
    finally:
        desktop.key_state('w', False)
    elapsed = time.monotonic()-began
    reached = feet()
    assert 7856. <= reached[0] <= 7872. and 7984. <= reached[2] <= 8016., reached
    assert -64. <= reached[1] <= 64., reached
    assert journal()['xp'] == 125, 'Area occupancy granted XP before turn-in'
    assert state()['combat']['player_deaths'] == 0
    capture('area-ready.png')
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
    proof = dict(passed=True, input='manual camera facing + normal W key',
                 outward_start=start, outward_reached_feet=reached,
                 outward_held_seconds=elapsed, return_feet=feet(),
                 area_ready_without_xp=True, completed_xp=150,
                 quest_revision=quest(QUEST)['revision'])
    (folder/'reach-area-proof.json').write_text(json.dumps(proof, indent=2)+'\n')
    return proof
