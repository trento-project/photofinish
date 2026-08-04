// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use crate::config::Scenario;

pub fn show_list(scenarios: &[Scenario]) {
    for scenario in scenarios.iter() {
        print_scenario(scenario)
    }
}

fn print_scenario(scenario: &Scenario) {
    println!(
        "NAME: {}\nFILES:\n{}\nDIRECTORIES:\n{}\n",
        scenario.label,
        scenario.files.join("\n"),
        scenario.directories.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // println! output isn't captured/asserted here (would need a refactor to inject
    // a writer); these are smoke tests confirming the functions don't panic.

    #[test]
    fn show_list_handles_no_scenarios() {
        show_list(&[]);
    }

    #[test]
    fn show_list_handles_scenarios_with_files_and_directories() {
        show_list(&[Scenario {
            label: "my-scenario".to_string(),
            files: vec!["a.json".to_string(), "b.json".to_string()],
            directories: vec!["some/dir".to_string()],
        }]);
    }

    #[test]
    fn print_scenario_handles_empty_files_and_directories() {
        print_scenario(&Scenario {
            label: "empty-scenario".to_string(),
            files: vec![],
            directories: vec![],
        });
    }
}
