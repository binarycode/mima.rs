use std::path::PathBuf;
use std::time::Duration;

#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();

    let path = std::env::var("PATH").unwrap();
    let mut stubs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    stubs.push("tests/stubs");
    t.env("PATH", format!("{stubs}:{path}", stubs = stubs.display()));

    t.default_bin_name("mima");

    t.timeout(Duration::from_secs(5));

    t.insert_var("[VERSION]", env!("CARGO_PKG_VERSION"))
        .unwrap();

    t.case("tests/cmd/apply_snapshot/apply_snapshot_failure.toml");
    t.case("tests/cmd/apply_snapshot/common_snapshots_for_multiple_disks.toml");
    t.case("tests/cmd/apply_snapshot/happy_path.toml");
    t.case("tests/cmd/apply_snapshot/happy_path_with_apply_alias.toml");
    t.case("tests/cmd/apply_snapshot/happy_path_with_restore_alias.toml");
    t.case("tests/cmd/apply_snapshot/happy_path_with_revert_alias.toml");
    t.case("tests/cmd/apply_snapshot/happy_path_with_switch_alias.toml");
    t.case("tests/cmd/apply_snapshot/help.toml");
    t.case("tests/cmd/apply_snapshot/list_snapshots_failure.toml");
    t.case("tests/cmd/apply_snapshot/more_than_two_arguments.toml");
    t.case("tests/cmd/apply_snapshot/no_arguments.toml");
    t.case("tests/cmd/apply_snapshot/one_argument.toml");
    t.case("tests/cmd/apply_snapshot/uncommon_snapshot_failure.toml");
    t.case("tests/cmd/apply_snapshot/unknown_guest.toml");
    t.case("tests/cmd/apply_snapshot/unknown_snapshot_failure.toml");

    t.case("tests/cmd/check_snapshot/happy_negative_path.toml");
    t.case("tests/cmd/check_snapshot/happy_path.toml");
    t.case("tests/cmd/check_snapshot/help.toml");
    t.case("tests/cmd/check_snapshot/list_snapshots_failure.toml");
    t.case("tests/cmd/check_snapshot/more_than_two_arguments.toml");
    t.case("tests/cmd/check_snapshot/no_arguments.toml");
    t.case("tests/cmd/check_snapshot/one_argument.toml");
    t.case("tests/cmd/check_snapshot/unknown_guest.toml");

    t.case("tests/cmd/command_execution_failed_error/error.toml");
    t.case("tests/cmd/command_execution_failed_error/error_without_stderr.toml");
    t.case("tests/cmd/command_execution_failed_error/error_without_stdout.toml");
    t.case("tests/cmd/command_execution_failed_error/error_without_streams.toml");

    t.case("tests/cmd/copy_file_to_guest/failure_when_path_is_not_a_file.toml");
    t.case("tests/cmd/copy_file_to_guest/happy_path.toml");
    t.case("tests/cmd/copy_file_to_guest/happy_path_when_file_name_is_specified.toml");
    t.case("tests/cmd/copy_file_to_guest/happy_path_with_copy_alias.toml");
    t.case("tests/cmd/copy_file_to_guest/happy_path_with_upload_alias.toml");
    t.case("tests/cmd/copy_file_to_guest/help.toml");
    t.case("tests/cmd/copy_file_to_guest/more_than_three_arguments.toml");
    t.case("tests/cmd/copy_file_to_guest/no_arguments.toml");
    t.case("tests/cmd/copy_file_to_guest/one_argument.toml");
    t.case("tests/cmd/copy_file_to_guest/scp_failure.toml");
    t.case("tests/cmd/copy_file_to_guest/ssh_failure.toml");
    t.case("tests/cmd/copy_file_to_guest/unknown_guest.toml");

    t.case("tests/cmd/create_snapshot/common_snapshots_for_multiple_disks.toml");
    t.case("tests/cmd/create_snapshot/create_snapshot_failure.toml");
    t.case("tests/cmd/create_snapshot/happy_path.toml");
    t.case("tests/cmd/create_snapshot/happy_path_with_snapshot_alias.toml");
    t.case("tests/cmd/create_snapshot/help.toml");
    t.case("tests/cmd/create_snapshot/list_snapshots_failure.toml");
    t.case("tests/cmd/create_snapshot/more_than_two_arguments.toml");
    t.case("tests/cmd/create_snapshot/no_arguments.toml");
    t.case("tests/cmd/create_snapshot/one_argument.toml");
    t.case("tests/cmd/create_snapshot/snapshot_already_exists_failure.toml");
    t.case("tests/cmd/create_snapshot/unknown_guest.toml");

    t.case("tests/cmd/delete_snapshot/happy_path.toml");
    t.case("tests/cmd/delete_snapshot/help.toml");
    t.case("tests/cmd/delete_snapshot/more_than_two_arguments.toml");
    t.case("tests/cmd/delete_snapshot/multiple_disks.toml");
    t.case("tests/cmd/delete_snapshot/no_arguments.toml");
    t.case("tests/cmd/delete_snapshot/one_argument.toml");
    t.case("tests/cmd/delete_snapshot/snapshot_removal_failure.toml");
    t.case("tests/cmd/delete_snapshot/unknown_guest.toml");

    t.case("tests/cmd/dublicate_snapshot_error/error.toml");

    t.case("tests/cmd/execute_file_on_guest/failure_when_path_is_not_a_file.toml");
    t.case("tests/cmd/execute_file_on_guest/first_ssh_failure.toml");
    t.case("tests/cmd/execute_file_on_guest/happy_path.toml");
    t.case("tests/cmd/execute_file_on_guest/happy_path_with_execute_alias.toml");
    t.case("tests/cmd/execute_file_on_guest/happy_path_with_execute_script_on_guest_alias.toml");
    t.case("tests/cmd/execute_file_on_guest/happy_path_with_extra_arguments.toml");
    t.case("tests/cmd/execute_file_on_guest/happy_path_with_run_alias.toml");
    t.case("tests/cmd/execute_file_on_guest/help.toml");
    t.case("tests/cmd/execute_file_on_guest/more_than_two_arguments.toml");
    t.case("tests/cmd/execute_file_on_guest/no_arguments.toml");
    t.case("tests/cmd/execute_file_on_guest/one_argument.toml");
    t.case("tests/cmd/execute_file_on_guest/scp_failure.toml");
    t.case("tests/cmd/execute_file_on_guest/second_ssh_failure.toml");
    t.case("tests/cmd/execute_file_on_guest/third_ssh_failure.toml");
    t.case("tests/cmd/execute_file_on_guest/unknown_guest.toml");

    t.case("tests/cmd/help.toml");

    t.case("tests/cmd/initialize_guest/disk_creation_failure.toml");
    t.case("tests/cmd/initialize_guest/happy_path.toml");
    t.case("tests/cmd/initialize_guest/happy_path_with_init_alias.toml");
    t.case("tests/cmd/initialize_guest/happy_path_with_init_guest_alias.toml");
    t.case("tests/cmd/initialize_guest/happy_path_with_multiple_disks.toml");
    t.case("tests/cmd/initialize_guest/happy_path_with_multiple_disks_when_some_are_skipped.toml");
    t.case("tests/cmd/initialize_guest/help.toml");
    t.case("tests/cmd/initialize_guest/more_than_one_argument.toml");
    t.case("tests/cmd/initialize_guest/no_arguments.toml");
    t.case("tests/cmd/initialize_guest/noop_when_path_exists.toml");
    t.case("tests/cmd/initialize_guest/snapshot_creation_failure.toml");
    t.case("tests/cmd/initialize_guest/unknown_guest.toml");

    t.case("tests/cmd/invalid_file_error/error.toml");

    t.case("tests/cmd/list_guests/happy_path.toml");
    t.case("tests/cmd/list_guests/happy_path_with_guests_alias.toml");
    t.case("tests/cmd/list_guests/happy_path_with_list_alias.toml");
    t.case("tests/cmd/list_guests/help.toml");
    t.case("tests/cmd/list_guests/more_than_zero_arguments.toml");

    t.case("tests/cmd/list_snapshots/big_difference_in_snapshot_timestamp_for_multiple_disks.toml");
    t.case("tests/cmd/list_snapshots/common_snapshots_for_multiple_disks.toml");
    t.case("tests/cmd/list_snapshots/happy_path.toml");
    t.case("tests/cmd/list_snapshots/help.toml");
    t.case("tests/cmd/list_snapshots/list_snapshots_failure.toml");
    t.case("tests/cmd/list_snapshots/more_than_one_argument.toml");
    t.case("tests/cmd/list_snapshots/multiple_snapshots.toml");
    t.case("tests/cmd/list_snapshots/no_arguments.toml");
    t.case("tests/cmd/list_snapshots/no_snapshots.toml");
    t.case("tests/cmd/list_snapshots/unknown_guest.toml");

    t.case("tests/cmd/missing_configuration_error/error_when_config_path_is_not_specified.toml");
    t.case("tests/cmd/missing_configuration_error/error_when_config_path_is_specified.toml");

    t.case("tests/cmd/parse_command_output_error/error.toml");
    t.case("tests/cmd/parse_command_output_error/error_without_stdout.toml");

    t.case("tests/cmd/parse_configuration_error/error.toml");

    t.case("tests/cmd/print_version.toml");

    t.case("tests/cmd/read_configuration_error/error.toml");

    t.case("tests/cmd/show_guest_details/happy_path.toml");
    t.case("tests/cmd/show_guest_details/happy_path_with_guest_alias.toml");
    t.case("tests/cmd/show_guest_details/happy_path_with_show_alias.toml");
    t.case("tests/cmd/show_guest_details/help.toml");
    t.case("tests/cmd/show_guest_details/more_than_one_argument.toml");
    t.case("tests/cmd/show_guest_details/no_arguments.toml");
    t.case("tests/cmd/show_guest_details/unknown_guest.toml");

    t.case("tests/cmd/start_guest/boot_from_more_than_one_source_failure.toml");
    t.case("tests/cmd/start_guest/guest_start_failure.toml");
    t.case("tests/cmd/start_guest/happy_path_with_boot_from_cdrom.toml");
    t.case("tests/cmd/start_guest/happy_path_with_boot_from_network.toml");
    t.case("tests/cmd/start_guest/happy_path_with_complex_configuration.toml");
    t.case("tests/cmd/start_guest/happy_path_with_several_cdroms.toml");
    t.case("tests/cmd/start_guest/help.toml");
    t.case("tests/cmd/start_guest/iproute_failure.toml");
    t.case("tests/cmd/start_guest/more_than_one_argument.toml");
    t.case("tests/cmd/start_guest/no_arguments.toml");
    t.case("tests/cmd/start_guest/noop_when_guest_is_already_running.toml");
    t.case("tests/cmd/start_guest/simple_happy_path.toml");
    t.case("tests/cmd/start_guest/simple_happy_path_with_start_alias.toml");
    t.case("tests/cmd/start_guest/unknown_guest.toml");

    t.case("tests/cmd/stop_guest/first_pkill_failure.toml");
    t.case("tests/cmd/stop_guest/happy_path_when_the_guest_is_not_running.toml");
    t.case("tests/cmd/stop_guest/happy_path_with_force_flag.toml");
    t.case("tests/cmd/stop_guest/happy_path_with_force_flag_unresponsive.toml");
    t.case("tests/cmd/stop_guest/happy_path_with_soft_shutdown.toml");
    t.case("tests/cmd/stop_guest/happy_path_with_soft_shutdown_timeout.toml");
    t.case("tests/cmd/stop_guest/happy_path_with_soft_shutdown_timeout_and_unresponsive.toml");
    t.case("tests/cmd/stop_guest/help.toml");
    t.case("tests/cmd/stop_guest/more_than_one_argument.toml");
    t.case("tests/cmd/stop_guest/no_arguments.toml");
    t.case("tests/cmd/stop_guest/second_pkill_failure.toml");
    t.case("tests/cmd/stop_guest/simple_happy_path.toml");
    t.case("tests/cmd/stop_guest/simple_happy_path_with_stop_alias.toml");
    t.case("tests/cmd/stop_guest/unknown_guest.toml");

    t.case("tests/cmd/unknown_guest_error/error.toml");

    t.case("tests/cmd/unknown_network_error/error.toml");

    t.case("tests/cmd/unknown_snapshot_error/error.toml");

    t.case("tests/cmd/wait_for_guest_to_shutdown/happy_path_with_wait.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/help.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/more_than_one_argument.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/no_arguments.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/simple_happy_path.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/simple_happy_path_with_wait_alias.toml");
    t.case("tests/cmd/wait_for_guest_to_shutdown/unknown_guest.toml");
}
