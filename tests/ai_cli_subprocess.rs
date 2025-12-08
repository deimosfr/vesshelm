use vesshelm::clients::ai::cli_subprocess::{PromptDelivery, run_cli};
use vesshelm::error::VesshelmError;

#[tokio::test]
async fn run_cli_pipes_stdin_to_stdout_via_cat() {
    let out = run_cli("cat", &[], "hello world", PromptDelivery::Stdin)
        .await
        .expect("cat ok");
    assert_eq!(out, "hello world");
}

#[tokio::test]
async fn run_cli_passes_prompt_as_arg_via_echo() {
    let out = run_cli("echo", &["-n"], "hello-arg", PromptDelivery::AsArg("--"))
        .await
        .expect("echo ok");
    assert!(out.contains("hello-arg"));
}

#[tokio::test]
async fn run_cli_returns_missing_dependency_when_binary_absent() {
    let result = run_cli(
        "vesshelm-nonexistent-binary-xyz",
        &[],
        "x",
        PromptDelivery::Stdin,
    )
    .await;
    match result {
        Err(VesshelmError::MissingDependency(msg)) => {
            assert!(msg.contains("vesshelm-nonexistent-binary-xyz"));
        }
        other => panic!("expected MissingDependency, got {:?}", other),
    }
}

#[tokio::test]
async fn run_cli_reports_nonzero_exit_as_command_failed() {
    let result = run_cli(
        "sh",
        &["-c", "echo boom 1>&2; exit 1"],
        "",
        PromptDelivery::Stdin,
    )
    .await;
    match result {
        Err(VesshelmError::CommandFailed(msg)) => {
            assert!(msg.contains("exited with"));
            assert!(msg.contains("boom"));
        }
        other => panic!("expected CommandFailed, got {:?}", other),
    }
}
