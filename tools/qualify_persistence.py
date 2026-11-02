#!/usr/bin/env python3
"""Store-only real PostgreSQL qualification in an owned isolated loopback cluster."""
import argparse
import json
from pathlib import Path
import subprocess
from private_postgres import Cluster


def run(output, sdk):
    output = output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    commands = []
    with Cluster(output/'cluster', sdk) as cluster:
        env = cluster.test_environment()
        flags = cluster.query('SHOW fsync; SHOW synchronous_commit; SHOW full_page_writes;')
        assert flags.returncode == 0 and flags.stdout.split() == ['on', 'on', 'on']
        wrong = cluster.query('SELECT 1;', wrong_password=True)
        assert wrong.returncode != 0 and 'password authentication failed' in wrong.stderr

        def cargo(name, args):
            with (output/(name+'.log')).open('w') as log:
                subprocess.run(['cargo', *args], env=env, stdout=log, stderr=subprocess.STDOUT,
                               timeout=300, check=True)
            commands.append(name)

        cargo('unit', ['test', '--locked', '-p', 'theia-persistence', '--lib', '-j', '2'])
        cargo('postgres-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'postgres_contract',
                                    '-j', '2', '--', '--ignored'])
        cargo('worker-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'worker_contract',
                                  '-j', '2', '--', '--ignored'])
        cargo('characters-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'characters_contract',
                                      '-j', '2', '--', '--ignored'])
        cargo('inventory-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'inventory_contract',
                                     '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('equipment-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'equipment_contract',
                                     '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('rewards-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'rewards_contract',
                                   '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('quests-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'quests_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('quest-lifecycle-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'quest_lifecycle_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('gameplay-profiles-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'gameplay_profiles_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('quest-turn-in-xp-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'quest_turn_in_xp_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('vendors-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'vendors_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('progression-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'progression_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        cargo('quest-journal-contract', ['test', '--locked', '-p', 'theia-persistence', '--test', 'quest_journal_contract', '-j', '2', '--', '--ignored', '--test-threads=1'])
        journal_query = """
            SELECT n.display_name,h.revision,p.experience,
              (SELECT count(*) FROM theia.quest_journal_entries e WHERE e.realm_id=n.realm_id AND e.character_id=n.character_id),
              (SELECT count(*) FROM theia.quest_journal_events e WHERE e.realm_id=n.realm_id AND e.character_id=n.character_id),
              (SELECT count(*) FROM theia.quest_journal_actions a WHERE a.realm_id=n.realm_id AND a.character_id=n.character_id),
              (SELECT coalesce(sum(xp_granted),0) FROM theia.quest_journal_actions a WHERE a.realm_id=n.realm_id AND a.character_id=n.character_id),
              (SELECT md5(string_agg(encode(state,'hex'),'|' ORDER BY quest_id)) FROM theia.quest_journal_entries e WHERE e.realm_id=n.realm_id AND e.character_id=n.character_id)
            FROM theia.character_names n JOIN theia.quest_journal_heads h USING(realm_id,character_id)
            JOIN theia.progression p USING(realm_id,character_id)
            WHERE n.display_name LIKE 'Journal%' ORDER BY n.display_name;
        """
        journal_before = cluster.query(journal_query)
        assert journal_before.returncode == 0
        journal_rows = [row.split('|') for row in journal_before.stdout.splitlines()]
        assert {row[0]: int(row[2]) for row in journal_rows} == {'Journalone':75,'Journaltwo':25,'Journalatomic':25,'Journalguard':0,'Journalzero':0,'Journalcap':25}
        (output/'quest-journal-before-restart.txt').write_text(journal_before.stdout)
        cargo('write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'store_qualification', '--', 'write'])
        cargo('character-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'character_qualification', '--', 'write'])
        cargo('inventory-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'inventory_qualification', '--', 'write'])
        cargo('equipment-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'equipment_qualification', '--', 'write'])
        cargo('rewards-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'rewards_qualification', '--', 'write'])
        cargo('progression-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'progression_qualification', '--', 'write'])
        cargo('quests-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quests_qualification', '--', 'write'])
        cargo('vendors-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'vendors_qualification', '--', 'write'])
        cargo('quest-lifecycle-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quest_lifecycle_qualification', '--', 'write'])
        cargo('quest-turn-in-xp-write', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quest_turn_in_xp_qualification', '--', 'write'])
        cluster.restart()
        journal_after = cluster.query(journal_query)
        assert journal_after.returncode == 0 and journal_after.stdout == journal_before.stdout, 'Multi-quest rows/receipts changed across database process restart'
        (output/'quest-journal-after-restart.txt').write_text(journal_after.stdout)
        cargo('reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'store_qualification', '--', 'read'])
        cargo('character-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'character_qualification', '--', 'read'])
        cargo('inventory-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'inventory_qualification', '--', 'read'])
        cargo('equipment-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'equipment_qualification', '--', 'read'])
        cargo('rewards-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'rewards_qualification', '--', 'read'])
        cargo('progression-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'progression_qualification', '--', 'read'])
        cargo('quests-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quests_qualification', '--', 'read'])
        cargo('quest-lifecycle-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quest_lifecycle_qualification', '--', 'read'])
        cargo('quest-turn-in-xp-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'quest_turn_in_xp_qualification', '--', 'read'])
        def completion_probe(name):
            return next(json.loads(line.removeprefix('quest_turn_in_xp_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('quest_turn_in_xp_probe '))
        assert completion_probe('quest-turn-in-xp-write') == completion_probe('quest-turn-in-xp-reload'), 'Completion XP state/receipts changed across actual process/database restart'
        def lifecycle_probe(name):
            return next(json.loads(line.removeprefix('quest_lifecycle_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('quest_lifecycle_probe '))
        assert lifecycle_probe('quest-lifecycle-write') == lifecycle_probe('quest-lifecycle-reload'), 'Explicit quest phase and pre-accept/action receipts changed across actual database restart'
        def quest_probe(name):
            return next(json.loads(line.removeprefix('quest_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('quest_probe '))
        assert quest_probe('quests-write') == quest_probe('quests-reload'), 'Objective/XP state and old receipts changed across actual database restart'
        def progression_probe(name):
            return next(json.loads(line.removeprefix('progression_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('progression_probe '))
        assert progression_probe('progression-write') == progression_probe('progression-reload'), 'Progression and old-event deduplication changed across actual database restart'
        def reward_probe(name):
            return next(json.loads(line.removeprefix('reward_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('reward_probe '))
        assert reward_probe('rewards-write') == reward_probe('rewards-reload'), 'Reward availability/consumption/items changed across actual database restart'
        def equipment_probe(name):
            return next(json.loads(line.removeprefix('equipment_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('equipment_probe '))
        assert equipment_probe('equipment-write') == equipment_probe('equipment-reload'), 'Equipment state/receipt changed across actual database restart'
        cargo('vendors-reload', ['run', '--locked', '-p', 'theia-persistence', '--example', 'vendors_qualification', '--', 'read'])
        def vendor_probe(name):
            return next(json.loads(line.removeprefix('vendor_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('vendor_probe '))
        assert vendor_probe('vendors-write')==vendor_probe('vendors-reload'), 'Purchase contents/receipt changed across actual database restart'
        def inventory_probe(name):
            return next(json.loads(line.removeprefix('inventory_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('inventory_probe '))
        assert inventory_probe('inventory-write') == inventory_probe('inventory-reload'), 'Backpack identity/state changed across actual database restart'
        def character_probe(name):
            return next(json.loads(line.removeprefix('character_probe ')) for line in (output/(name+'.log')).read_text().splitlines() if line.startswith('character_probe '))
        assert character_probe('character-write') == character_probe('character-reload'), 'Character identity/state changed across actual database restart'

        version = subprocess.check_output([str(cluster.bin/'postgres'), '--version'], env=cluster.env, text=True).strip()
    assert cluster.server is None
    assert not (output/'cluster'/'password').exists() and not (output/'cluster'/'pgpass').exists()
    result = dict(passed=True, scope='PostgreSQL store/account/character/item/reward/progression/objective/vendor/explicit-quest-journal-and-completion-XP boundaries; no live kill credit, NPC/range, networking or native purchase proof',
                  postgres=version, commands=commands, authenticated_loopback=True,
                  wrong_password_rejected=True, durability_flags=['fsync=on','synchronous_commit=on','full_page_writes=on'],
                  multi_quest_rows_event_receipts_and_exact_snapshots_survive_restart=True,
                  multi_quest_atomic_completion_rollback_and_lock_barrier=True,
                  quest_events_independent_of_kill_xp_recipe=True,
                  real_clean_restart_reload=True, named_character_identity_and_pose_survive_restart=True,
                  bounded_character_create_list_select=True, durable_backpack_restart_and_latest_receipt=True,
                  inventory_conflict_and_foreign_owner_rejection=True, inventory_dropped_response_recovery=True,
                  bounded_inventory_queue_and_backend_loss_fail_closed=True, owned_cluster_stopped=True,
                  durable_equipment_restart_and_latest_receipt=True,
                  equipment_backpack_race_conserves_items=True, equipment_dropped_response_recovery=True,
                  equipment_backend_loss_never_acknowledged_as_commit=True,
                  nonrepeatable_reward_claims_and_latest_receipt=True,
                  reward_full_bag_retains_claim_and_transaction_rollback=True,
                  reward_availability_consumption_and_items_survive_restart=True,
                  reward_cancelled_response_and_backend_loss_fail_closed=True,
                  progression_old_event_deduplication_and_cap=True,
                  progression_atomic_receipt_rollback_and_backend_loss=True,
                  progression_process_database_restart_and_old_receipts=True,
                  quest_atomic_xp_and_objective_receipts=True,
                  quest_completion_independent_of_xp_cap=True,
                  quest_old_event_queue_backend_and_restart=True,
                  explicit_quest_acceptance_turn_in_and_retained_action_receipts=True,
                  preaccept_capped_kill_receipts_prevent_retrospective_credit=True,
                  explicit_quest_process_database_restart=True,
                  journal_only_turn_in_grants_enabled=False,
                  completion_xp_atomic_positive_clipped_and_capped=True,
                  completion_xp_lost_answers_bounded_worker_and_backend_loss=True,
                  completion_xp_migration_legacy_rows_and_no_silent_adoption=True,
                  completion_xp_process_database_restart_and_receipts=True,
                  live_completion_xp_enabled=False,
                  gameplay_profile_preflight_read_only_and_expected_mismatch=True,
                  gameplay_profile_preflight_cancelled_answer_and_bounded_queue=True,
                  vendor_atomic_cost_output_and_latest_receipt=True,
                  vendor_full_bag_rollback_and_freed_slot=True,
                  vendor_dropped_answer_queue_and_backend_failure=True,
                  vendor_process_database_restart_receipt=True,
                  plaintext_password_files_removed=True)
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sdk', type=Path, default=Path('build/postgres-sdk'))
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.sdk.resolve()), indent=2))
