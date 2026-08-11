use super::*;

#[test]
fn each_campaign_reference_reaches_terminal_success() {
    for id in MissionId::ALL {
        let report = run_scenario(id, ScenarioKind::Reference);
        assert!(report.terminal_success, "{report:?}");
        assert!(report.continuation_matches);
        assert_eq!(report.mass_balance_error, 0);
        assert!(report.ticks >= 100);
        assert!(report.injected_vu > 0);
        assert!(report.admitted_commands > 0, "{report:?}");
        assert!(report.placed_devices > 0, "{report:?}");
    }
}

#[test]
fn campaign_reference_hashes_repeat() {
    for id in MissionId::ALL {
        assert_eq!(
            run_scenario(id, ScenarioKind::Reference),
            run_scenario(id, ScenarioKind::Reference)
        );
    }
}

#[test]
fn alternate_routes_reach_success_and_preserve_midpoint_continuation() {
    for id in MissionId::ALL {
        let first = run_scenario(id, ScenarioKind::Alternate);
        let second = run_scenario(id, ScenarioKind::Alternate);
        assert_eq!(first, second);
        assert!(first.terminal_success, "{first:?}");
        assert!(first.continuation_matches);
        assert_eq!(first.mass_balance_error, 0);
        assert_ne!(first.midpoint_hash, first.final_hash);
        assert!(first.admitted_commands > 0, "{first:?}");
    }
}

#[test]
fn authored_failure_scenarios_fail_deterministically() {
    for id in MissionId::ALL {
        let report = run_scenario(id, ScenarioKind::Failure);
        assert!(report.terminal_failure, "{report:?}");
        assert!(!report.terminal_success);
        assert!(report.continuation_matches);
        assert_eq!(report.mass_balance_error, 0);
    }
}

#[test]
fn l03_insufficient_water_stays_recoverable() {
    let report = run_scenario(
        MissionId::L03Firebreak,
        ScenarioKind::InsufficientWaterRecovery,
    );
    assert!(!report.terminal_success);
    assert!(!report.terminal_failure);
    assert!(report.objective_incomplete);
    assert!(report.continuation_matches);
    assert_eq!(report.mass_balance_error, 0);
}
