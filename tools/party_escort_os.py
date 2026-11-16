"""Two owned clients observe one escort and complete their own durable turn-in."""
import json
import math
import time
from party_os import form
from quest_travel_os import controls, FOLLOW, RANGER
from qualify_world_input_os import wait


def flow(desktop, windows, states, folder):
    membership = form(desktop, windows, states)
    ui = [controls(desktop, windows[i], states[i], folder/('owner' if i == 0 else 'guest')) for i in [0, 1]]
    for c in ui:
        c.focus(); c.close()
    home = ui[0].contact(RANGER)['global']
    ui[0].speak(RANGER); ui[0].action(FOLLOW, 'Active'); ui[0].close()
    time.sleep(2)
    assert math.dist(ui[1].contact(RANGER)['global'], home) < .02, 'Unaccepted party member lost private contact'
    ui[1].speak(RANGER); ui[1].action(FOLLOW, 'Active'); ui[1].close()
    samples = []

    def sample(driver):
        contacts = [c.contact(RANGER) for c in ui]
        npcs = [c['global'] for c in contacts]
        assert math.dist(*npcs) < 2., 'Party contact views diverged beyond snapshot timing'
        value = dict(driver=driver, npc=npcs, players=[c.feet() for c in ui],
                     rendered=[c['rendered_global'] for c in contacts],
                     statuses=[c.quest(FOLLOW)['status'] for c in ui])
        samples.append(value)
        return value

    def follow(index, stop):
        c = ui[index]
        c.focus(); c.close()
        deadline = time.monotonic()+120
        moving = False
        try:
            while not stop():
                value = sample(index+1)
                target = value['npc'][index]
                assert time.monotonic() < deadline, 'Shared escort approach timed out'
                c.face(target)
                distance = math.dist(c.feet(), target)
                if not moving and distance > 6.:
                    desktop.key_state('w', True); moving = True
                elif moving and distance < 3.:
                    desktop.key_state('w', False); moving = False
                time.sleep(.1)
        finally:
            if moving: desktop.key_state('w', False)

    # A carries the ranger beyond B's range, while B sees the same moving NPC.
    follow(0, lambda: math.dist(ui[0].feet(), home) > 28.)
    assert math.dist(ui[1].feet(), ui[1].contact(RANGER)['global']) > 10.
    # Retreat a short distance along the already travelled route. Neither
    # participant is now eligible to keep her moving.
    ui[0].face(home); desktop.hold_key('w', .7)
    time.sleep(2)
    paused = [c.contact(RANGER)['global'] for c in ui]
    assert all(math.dist(c.feet(), paused[i]) > 10. for i, c in enumerate(ui))
    time.sleep(1)
    assert all(math.dist(paused[i], c.contact(RANGER)['global']) < .02 for i, c in enumerate(ui))
    assert math.dist(*paused) < .02
    for c in ui:
        c.focus(); c.capture('party-ranger-waits.png')
    # B takes over and reaches the ancient tree; A stays behind with no credit.
    follow(1, lambda: ui[1].quest(FOLLOW)['status'] == 'Ready')
    assert ui[0].quest(FOLLOW)['status'] == 'Active'
    assert [c.journal()['xp'] for c in ui] == [50, 50]
    ui[1].capture('party-ranger-ready.png')
    ui[1].speak(RANGER); ui[1].action(FOLLOW, 'Completed'); ui[1].close()
    assert ui[0].quest(FOLLOW)['status'] == 'Active' and ui[0].journal()['xp'] == 50
    # Reuse the observed, already traversed NPC route to bring A to the tree.
    # These are ordinary keyboard/mouse controls, not a pose or credit write.
    c = ui[0]; c.focus(); c.close()
    route = [s['npc'][0] for s in samples if s['driver'] == 2]
    last = c.feet()
    route = [point for point in route if math.dist(point, home) > math.dist(last, home)]
    anchors = []
    for point in route:
        if not anchors or math.dist(point, anchors[-1]) > 4.:
            anchors.append(point)
    anchors.append(c.contact(RANGER)['global'])
    deadline = time.monotonic()+100
    for point in anchors:
        while math.dist(c.feet(), point) > 2.5:
            assert time.monotonic() < deadline, 'Returning participant did not reach the shared destination'
            c.walk(point); time.sleep(.1)
    wait(lambda: c.quest(FOLLOW)['status'] == 'Ready', 5)
    assert c.journal()['xp'] == 50
    c.speak(RANGER); c.action(FOLLOW, 'Completed'); c.close()
    assert all(c.journal()['revision'] == 9 and c.journal()['xp'] == 75 for c in ui)
    assert all(c.quest(FOLLOW)['status'] == 'Completed' for c in ui)
    assert any(math.dist(s['npc'][1], home) > 20. and math.dist(s['npc'][1], s['players'][1]) > 10. for s in samples if s['driver'] == 1)
    assert len({tuple(s['rendered'][1]) for s in samples if s['driver'] == 1}) > 10, 'Background member did not render the moving ranger'
    result = dict(passed=True, membership=membership, samples=samples, waiting=paused,
                  individual_arrival_and_turn_in=True, nonparticipant_private=True)
    (folder/'party-escort-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
